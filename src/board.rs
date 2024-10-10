use core::fmt;
use std::rc::Rc;

use ahash;
use log;

use crate::engine;
use crate::errors::BoardStateError;
use crate::errors::FenParseError;
use crate::errors::PGNParseError;
use crate::log_and_return_error;
use crate::movegen::*;
use crate::pgn::notation::Notation;
use crate::position::*;
use crate::transposition;
use crate::util;
use crate::zobrist;
use crate::zobrist::PositionHash;

#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    Check,
    Checkmate,
    Stalemate,
    Repetition,
    FiftyMove,
    InsufficientMaterial,
    Active,
}
impl GameState {
    // gamestates that are draws
    #[inline]
    pub fn is_draw(&self) -> bool {
        matches!(
            self,
            GameState::Stalemate
                | GameState::FiftyMove
                | GameState::Repetition
                | GameState::InsufficientMaterial
        )
    }
    // gamestates that end game
    #[inline]
    pub fn is_game_over(&self) -> bool {
        matches!(self, GameState::Checkmate | GameState::Stalemate) || self.is_draw()
    }
}
// String representation of GameState
impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state_str = match self {
            GameState::Check => "Check",
            GameState::Checkmate => "Checkmate",
            GameState::Stalemate => "Stalemate",
            GameState::Repetition => "Repetition",
            GameState::FiftyMove => "Fifty Move Draw",
            GameState::InsufficientMaterial => "Insufficient Material",
            GameState::Active => "",
        };
        write!(f, "{}", state_str)
    }
}

#[derive(Debug, Clone)]
pub struct BoardState {
    pub side_to_move: PieceColour,
    pub last_move: Move,
    legal_moves: Vec<Move>,
    pub board_hash: u64,
    pub position_hash: u64,
    position: Position,
    move_count: u32,
    halfmove_count: u32,
    position_occurences: ahash::AHashMap<PositionHash, u8>,
    lazy_legal_moves: bool,
}

impl PartialEq for BoardState {
    fn eq(&self, other: &Self) -> bool {
        self.board_hash == other.board_hash && self.position_hash == other.position_hash
    }
}

impl BoardState {
    pub fn new_starting() -> Self {
        let position = Position::new_starting();
        log::info!("New starting Position created");
        let position_hash: PositionHash = position.pos_hash();
        let board_hash = zobrist::board_state_hash(position_hash, 1, 0);
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().cloned().collect();
        log::trace!("Legal moves generated: {legal_moves:?}");
        let mut position_occurences = ahash::AHashMap::default();
        position_occurences.insert(position_hash, 1);
        log::info!("New starting BoardState created");
        BoardState {
            position,
            move_count: 1, // movecount starts at 1
            halfmove_count: 0,
            position_hash,
            board_hash,
            side_to_move,
            last_move: NULL_MOVE,
            legal_moves,
            position_occurences,
            lazy_legal_moves: false,
        }
    }

    // TODO check for overflows
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let (position, fen_vec) = Position::from_fen_partial_impl(fen)?;

        // check for multiple kings, should be the only issue in terms of pieces on the board
        let mut wking_num = 0;
        let mut bking_num = 0;
        for s in position.pos64 {
            match s {
                Square::Piece(p) => {
                    if p.ptype == PieceType::King {
                        match p.pcolour {
                            PieceColour::White => {
                                wking_num += 1;
                            }
                            PieceColour::Black => {
                                bking_num += 1;
                            }
                        }
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
            if wking_num > 1 || bking_num > 1 {
                let err = FenParseError(format!(
                    "Multiple kings (white: {}, black: {}) in FEN: {}",
                    wking_num, bking_num, fen
                ));
                log_and_return_error!(err)
            }
        }

        log::debug!("New Position created from FEN");
        log::trace!("FEN: {fen}, Position: {position:?}");
        let position_hash: PositionHash = position.pos_hash();
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().cloned().collect();
        let mut position_occurences = ahash::AHashMap::default();
        position_occurences.insert(position_hash, 1);

        // default values for move count and halfmove count if not provided see <https://www.talkchess.com/forum3/viewtopic.php?f=7&t=79627>
        let mut halfmove_count: u32 = 0;
        let mut move_count: u32 = 1;

        if fen_vec.len() >= 5 {
            halfmove_count = match fen_vec[4].parse::<u32>() {
                Ok(halfmove_count) => halfmove_count,
                Err(_) => {
                    let err =
                        FenParseError(format!("Error parsing halfmove count: {}", fen_vec[4]));
                    log_and_return_error!(err)
                }
            };

            if fen_vec.len() == 6 {
                move_count = match fen_vec[5].parse::<u32>() {
                    Ok(move_count) => move_count,
                    Err(_) => {
                        let err =
                            FenParseError(format!("Error parsing move count: {}", fen_vec[5]));
                        log_and_return_error!(err)
                    }
                };
            }
        }

        let board_hash = zobrist::board_state_hash(position_hash, 1, halfmove_count); // FEN doesnt store position occurrence info, so set to 1

        log::info!("New BoardState created from FEN");
        Ok(BoardState {
            side_to_move,
            last_move: NULL_MOVE,
            legal_moves,
            position,
            move_count,
            halfmove_count,
            position_hash,
            board_hash,
            position_occurences,
            lazy_legal_moves: false,
        })
    }

    pub fn to_fen(&self) -> String {
        // final two fields of the FEN string, halfmove count and move count
        let mut fen_str = self.position.to_fen_partial_impl();
        fen_str.push_str(&format!("{} {}", self.halfmove_count, self.move_count));
        log::info!("Converted BoardState to FEN: {}", fen_str);

        fen_str
    }

    pub fn get_pseudo_legal_moves(&self) -> &Vec<Move> {
        self.position.get_pseudo_legal_moves()
    }

    // checks if a move would create a legal position, does not check for boardstate legality
    pub fn is_move_legal_position(&self, mv: &Move) -> bool {
        self.position.is_move_legal(mv)
    }

    // lazily do legality check on pseudo legal moves as the iterator is used
    pub fn lazy_get_legal_moves(&self) -> impl Iterator<Item = &Move> {
        self.position
            .get_pseudo_legal_moves()
            .iter()
            .filter(|mv| self.position.is_move_legal(mv))
    }

    // next state without legality and gamestate checks done (legal_moves is empty), may panic if unreachable code is hit e.g. in zobrist hash generation if position occurrences ever gets above 3
    // USERS MUST CHECK IF GAMESTATE IS VALID (E.G THREEFOLD REPETITION, 50 MOVE RULE) AS THIS FUNCTION DOES NOT
    pub fn lazy_next_state_unchecked(&self, mv: &Move) -> Self {
        let position = self.position.new_position(mv);
        log::trace!("New Position created from move: {:?}", mv);
        let position_hash = zobrist::pos_next_hash(
            &self.position.movegen_flags,
            &position.movegen_flags,
            self.position_hash,
            mv,
        );
        log::trace!(
            "New position hash generated: {}",
            util::hash_to_string(position_hash)
        );
        let side_to_move = position.side;
        let last_move = *mv;
        // deref all legal moves
        let legal_moves = Vec::with_capacity(0); // empty vec as we don't need to generate legal moves ahead of time

        let move_count = if side_to_move == PieceColour::White {
            self.move_count + 1
        } else {
            self.move_count
        };

        let halfmove_reset = matches!(
            mv.move_type,
            MoveType::PawnPush | MoveType::DoublePawnPush | MoveType::Capture(_)
        );
        let halfmove_count = if halfmove_reset {
            0
        } else {
            self.halfmove_count + 1
        };

        let mut position_occurences = self.position_occurences.clone();
        let po = position_occurences.entry(position_hash).or_insert(0);
        *po += 1;

        let board_hash = zobrist::board_state_hash(position_hash, *po, halfmove_count);
        //let board_hash = position_hash ^ (*po as u64) ^ (halfmove_count as u64);
        log::trace!("Board hash: {}", util::hash_to_string(board_hash));

        log::trace!("New BoardState created from move: {:?}", mv);
        Self {
            side_to_move,
            last_move,
            legal_moves,
            position,
            board_hash,
            position_hash,
            move_count,
            halfmove_count,
            position_occurences,
            lazy_legal_moves: true,
        }
    }

    pub fn lazy_next_state(&self, _mv: &Move) -> Result<Self, BoardStateError> {
        // TODO maybe just gen_legal_moves and call next_state?
        todo!()
    }

    pub fn next_state_unchecked(&self, _mv: &Move) -> Self {
        todo!()
    }

    pub fn next_state(&self, mv: &Move) -> Result<Self, BoardStateError> {
        if mv == &NULL_MOVE {
            let err = BoardStateError::NullMove(
                "&NULL_MOVE was passed as an argument to BoardState::next_state()".to_string(),
            );
            log_and_return_error!(err)
        }
        if self.lazy_legal_moves {
            let err = BoardStateError::LazyIncompatiblity("next_state called on BoardState with lazy_legal_moves flag set, cannot generate next state without all legal moves being generated.".to_string());
            log_and_return_error!(err)
        }
        if !self.legal_moves.contains(mv) {
            let err = BoardStateError::IllegalMove(format!("{:?} is not a legal move", mv));
            log_and_return_error!(err)
        }

        let current_game_state = self.get_gamestate();

        if current_game_state == GameState::Checkmate
            || current_game_state == GameState::Stalemate
            || current_game_state == GameState::FiftyMove
            || current_game_state == GameState::Repetition
        {
            let err = BoardStateError::NoLegalMoves(current_game_state);
            log_and_return_error!(err)
        }

        let position = self.position.new_position(mv);
        log::trace!("New Position created from move: {:?}", mv);
        let position_hash = zobrist::pos_next_hash(
            &self.position.movegen_flags,
            &position.movegen_flags,
            self.position_hash,
            mv,
        );
        log::trace!(
            "New position hash generated: {}",
            util::hash_to_string(position_hash)
        );
        let side_to_move = position.side;
        let last_move = *mv;
        // deref all legal moves
        let legal_moves = position.get_legal_moves().into_iter().cloned().collect();
        log::trace!("Legal moves generated: {legal_moves:?}");

        let move_count = if side_to_move == PieceColour::White {
            self.move_count + 1
        } else {
            self.move_count
        };

        let halfmove_reset = matches!(
            mv.move_type,
            MoveType::PawnPush | MoveType::DoublePawnPush | MoveType::Capture(_)
        );
        let halfmove_count = if halfmove_reset {
            0
        } else {
            self.halfmove_count + 1
        };

        let mut position_occurences = self.position_occurences.clone();
        let po = position_occurences.entry(position_hash).or_insert(0);
        *po += 1;

        let board_hash = zobrist::board_state_hash(position_hash, *po, halfmove_count);
        //let board_hash = position_hash ^ (*po as u64) ^ (halfmove_count as u64);
        log::trace!("Board hash: {}", util::hash_to_string(board_hash));

        log::trace!("New BoardState created from move: {:?}", mv);
        Ok(Self {
            side_to_move,
            last_move,
            legal_moves,
            position,
            board_hash,
            position_hash,
            move_count,
            halfmove_count,
            position_occurences,
            lazy_legal_moves: false,
        })
    }

    fn gen_legal_moves(&mut self) {
        self.legal_moves = self
            .position
            .get_legal_moves()
            .into_iter()
            .cloned()
            .collect();
    }

    pub fn get_legal_moves(&self) -> Result<&[Move], BoardStateError> {
        if self.lazy_legal_moves {
            let err = BoardStateError::LazyIncompatiblity("get_legal_moves called on BoardState with lazy_legal_moves flag set, legal_moves vec is empty".to_string());
            log_and_return_error!(err)
        }
        Ok(&self.legal_moves)
    }

    pub fn get_occurences_of_current_position(&self) -> u8 {
        *self
            .position_occurences
            .get(&self.position_hash)
            .unwrap_or(&1)
    }
    // TODO add check for insufficient material
    pub fn get_gamestate(&self) -> GameState {
        let legal_moves_empty = if self.lazy_legal_moves {
            self.lazy_get_legal_moves().peekable().peek().is_none()
        } else {
            self.legal_moves.is_empty()
        };
        let is_in_check = self.position.is_in_check();

        // checkmate has to be checked for first, as it supercedes other states like the 50 move rule
        if is_in_check && legal_moves_empty {
            GameState::Checkmate
        } else if !is_in_check && legal_moves_empty {
            GameState::Stalemate
        } else if self.halfmove_count >= 100 {
            GameState::FiftyMove
        } else if self.get_occurences_of_current_position() >= 3 {
            GameState::Repetition
        } else if is_in_check {
            GameState::Check
        } else if false {
            //placeholder
            GameState::InsufficientMaterial
        } else {
            GameState::Active
        }
    }

    // fn is_in_check(&self) -> bool {
    //     self.position.is_in_check()
    // }

    // fn is_checkmate(&self) -> bool {
    //     return if self.lazy_legal_moves {
    //         self.lazy_is_checkmate()
    //     } else {
    //         self.legal_moves.is_empty() && self.position.is_in_check()
    //     };
    // }

    // fn is_draw(&self) -> bool {
    //     return if self.lazy_legal_moves {
    //         self.lazy_is_draw()
    //     } else {
    //         (self.legal_moves.is_empty() && !self.position.is_in_check())
    //             || self.halfmove_count >= 100
    //             || self.get_occurences_of_current_position() >= 3
    //     };
    // }

    // // is_checkmate only checking if the lazy legal moves iterator returns None on peek
    // fn lazy_is_checkmate(&self) -> bool {
    //     self.lazy_get_legal_moves().peekable().peek().is_none() && self.position.is_in_check()
    // }

    // // is draw only checking if the lazy legal moves iterator returns None on peek
    // fn lazy_is_draw(&self) -> bool {
    //     self.halfmove_count >= 100
    //         || self.get_occurences_of_current_position() >= 3
    //         || (self.lazy_get_legal_moves().peekable().peek().is_none()
    //             && !self.position.is_in_check())
    // }

    pub fn get_pos64(&self) -> &Pos64 {
        &self.position.pos64
    }
}

#[derive(Debug)]
pub struct Board {
    pub current_state: BoardState,
    pub detached_idx: Option<usize>,
    pub state_history: Vec<BoardState>,
    pub move_history: Vec<Move>,
    transposition_table: transposition::TranspositionTable,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        let current_state = BoardState::new_starting();
        let mut state_history: Vec<BoardState> = Vec::new();
        log::info!("State history created");
        state_history.push(current_state.clone());

        let transposition_table = transposition::TranspositionTable::new();
        log::info!("Transposition table created");
        log::info!("New Board created");
        Board {
            current_state,
            detached_idx: None,
            state_history,
            move_history: Vec::new(),
            transposition_table,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let current_state = BoardState::from_fen(fen)?;
        let state_history: Vec<BoardState> = vec![current_state.clone()];

        let transposition_table = transposition::TranspositionTable::new();
        log::info!("New Board created from FEN: {}", fen);
        Ok(Board {
            current_state,
            detached_idx: None,
            state_history,
            move_history: Vec::new(),
            transposition_table,
        })
    }

    pub fn to_fen(&self) -> String {
        self.current_state.to_fen()
    }

    pub fn get_starting(&self) -> &BoardState {
        // first element in state_history is guarenteed to be initialised as starting BoardState
        &self.state_history[0]
    }

    pub fn make_move(&mut self, mv: &Move) -> Result<GameState, BoardStateError> {
        let next_state = self.current_state.next_state(mv)?;
        self.current_state = next_state;
        self.state_history.push(self.current_state.clone());
        self.move_history.push(*mv);

        let game_state = self.current_state.get_gamestate();
        //println!("FEN: {}", self.to_fen());

        Ok(game_state)
    }

    pub fn make_engine_move(&mut self, depth: u8) -> Result<GameState, BoardStateError> {
        let (eval, engine_move) =
            engine::choose_move(&self.current_state, depth, &mut self.transposition_table);
        let mv = *engine_move;
        log::info!("Engine move chosen: {:?} @ eval: {}", engine_move, eval);

        self.make_move(&mv)
    }

    pub fn move_history_string_notation(&self) -> Vec<String> {
        let mut notations_string = Vec::new();
        let notations = self.move_history_notation();
        for n in notations {
            notations_string.push(n.to_string());
        }
        notations_string
    }

    pub fn move_history_notation(&self) -> Vec<Notation> {
        let mut notations = Vec::new();
        for (state, mv) in self.state_history.iter().zip(self.move_history.iter()) {
            // move will all be legal, so unwrap is safe
            let notation = Notation::from_mv_with_context(state, mv).unwrap();
            notations.push(notation);
        }
        notations
    }

    pub fn unmake_move(&mut self) -> Result<Rc<BoardState>, BoardStateError> {
        todo!()
    }

    pub fn get_current_gamestate(&self) -> GameState {
        self.current_state.get_gamestate()
    }
}
