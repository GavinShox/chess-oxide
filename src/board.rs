use core::fmt;
use std::rc::Rc;

use ahash;
use log;

use crate::engine;
use crate::errors::BoardStateError;
use crate::errors::FenParseError;
use crate::movegen::*;
use crate::position::*;
use crate::util;

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
    pub legal_moves: Vec<Move>,
    pub position_hash: u64,
    position: Position,
    move_count: u32,
    halfmove_count: u32,
    position_occurences: ahash::AHashMap<PositionHash, u8>,
}

impl BoardState {
    pub fn new_starting() -> Self {
        let position = Position::new_starting();
        log::info!("New starting Position created");
        let position_hash: PositionHash = position.pos_hash();
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().copied().collect();
        log::info!("Legal moves generated: {legal_moves:?}");
        let mut position_occurences = ahash::AHashMap::default();
        *position_occurences.entry(position_hash).or_insert(0) += 1;
        log::info!("New starting BoardState created");
        BoardState {
            position,
            move_count: 1, // movecount starts at 1
            halfmove_count: 0,
            position_hash,
            side_to_move,
            last_move: NULL_MOVE,
            legal_moves,
            position_occurences,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let (position, fen_vec) = Position::from_fen_partial_impl(fen)?;
        log::info!("New Position created from FEN");
        log::debug!("FEN: {fen}, Position: {position:?}");
        let position_hash: PositionHash = position.pos_hash();
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().copied().collect();
        let mut position_occurences = ahash::AHashMap::default();
        *position_occurences.entry(position_hash).or_insert(0) += 1;

        // default values for move count and halfmove count if not provided see <https://www.talkchess.com/forum3/viewtopic.php?f=7&t=79627>
        let mut halfmove_count: u32 = 0;
        let mut move_count: u32 = 1;

        if fen_vec.len() >= 5 {
            halfmove_count = match fen_vec[4].parse::<u32>() {
                Ok(halfmove_count) => halfmove_count,
                Err(_) => {
                    log::error!("Error parsing halfmove count: {}", fen_vec[4]);
                    return Err(FenParseError(format!(
                        "Error parsing halfmove count: {}",
                        fen_vec[4]
                    )));
                }
            };

            if fen_vec.len() == 6 {
                move_count = match fen_vec[5].parse::<u32>() {
                    Ok(move_count) => move_count,
                    Err(_) => {
                        log::error!("Error parsing move count: {}", fen_vec[5]);
                        return Err(FenParseError(format!(
                            "Error parsing move count: {}",
                            fen_vec[5]
                        )));
                    }
                }
            }
        }
        log::info!("New BoardState created from FEN");
        Ok(BoardState {
            side_to_move,
            last_move: NULL_MOVE,
            legal_moves,
            position,
            move_count,
            halfmove_count,
            position_hash,
            position_occurences,
        })
    }

    pub fn to_fen(&self) -> String {
        // final two fields of the FEN string, halfmove count and move count
        let mut fen_str = self.position.to_fen_partial_impl();
        fen_str.push_str(&format!("{} {}", self.halfmove_count, self.move_count));
        log::info!("Converted BoardState to FEN: {}", fen_str);

        fen_str
    }

    pub fn get_move_count(&self) -> u32 {
        self.move_count
    }

    pub fn last_move_as_notation(&self) -> Result<String, BoardStateError> {
        if self.last_move == NULL_MOVE {
            return Err(BoardStateError::NullMove(
                "last_move is NULL_MOVE, has a move been made yet?".to_string(),
            ));
        }

        let notation_from = util::index_to_notation(self.last_move.from);
        let notation_to = util::index_to_notation(self.last_move.to);

        let get_piece_str = |ptype: PieceType| -> String {
            match ptype {
                PieceType::Pawn => notation_from.chars().next().unwrap().to_string(), // get pawns rank
                PieceType::Knight => "N".to_string(),
                PieceType::Bishop => "B".to_string(),
                PieceType::Rook => "R".to_string(),
                PieceType::Queen => "Q".to_string(),
                PieceType::King => "K".to_string(),
                PieceType::None => "".to_string(),
            }
        };

        let piece_str = get_piece_str(self.last_move.piece.ptype);

        let notation = match self.last_move.move_type {
            MoveType::EnPassant(ep) => format!("{}x{}", piece_str, util::index_to_notation(ep)),
            MoveType::Promotion(promotion_type) => {
                format!("{}={}", notation_to, get_piece_str(promotion_type))
            }
            MoveType::Castle(castle_move) => {
                if castle_move.rook_from.abs_diff(castle_move.rook_to) == 3 {
                    "O-O-O".to_string()
                } else {
                    "O-O".to_string()
                }
            }
            MoveType::DoublePawnPush => notation_to,
            MoveType::PawnPush => notation_to,
            MoveType::Capture(_) => format!("{}x{}", piece_str, notation_to),
            MoveType::Normal => format!("{}{}", piece_str, notation_to),
            MoveType::None => "".to_string(),
        };
        return if self.get_gamestate() == GameState::Checkmate {
            Ok(format!("{}#", notation))
        } else if self.get_gamestate() == GameState::Check {
            Ok(format!("{}+", notation))
        } else {
            Ok(notation)
        };
    }

    pub fn next_state(&self, mv: &Move) -> Result<Self, BoardStateError> {
        if mv == &NULL_MOVE {
            log::error!("&NULL_MOVE was passed as an argument to BoardState::next_state()");
            return Err(BoardStateError::NullMove(
                "&NULL_MOVE was passed as an argument to BoardState::next_state()".to_string(),
            ));
        }
        if !self.legal_moves.contains(mv) {
            log::error!("{:?} is not a legal move", mv);
            return Err(BoardStateError::IllegalMove(format!(
                "{:?} is not a legal move",
                mv
            )));
        }

        let current_game_state = self.get_gamestate();

        if current_game_state == GameState::Checkmate
            || current_game_state == GameState::Stalemate
            || current_game_state == GameState::FiftyMove
            || current_game_state == GameState::Repetition
        {
            log::error!(
                "No legal moves in current game state: {:?}",
                current_game_state
            );
            return Err(BoardStateError::NoLegalMoves(current_game_state));
        }

        let position = self.position.new_position(mv);
        log::debug!("New Position created from move: {:?}", mv);
        let position_hash = position.pos_hash();
        let side_to_move = position.side;
        let last_move = *mv;
        // deref all legal moves
        let legal_moves = position.get_legal_moves().into_iter().copied().collect();
        log::debug!("Legal moves generated: {legal_moves:?}");

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
        *position_occurences.entry(position_hash).or_insert(0) += 1;
        log::info!("New BoardState created from move: {:?}", mv);
        Ok(Self {
            side_to_move,
            last_move,
            legal_moves,
            position,
            position_hash,
            move_count,
            halfmove_count,
            position_occurences,
        })
    }

    pub fn get_occurences_of_current_position(&self) -> u8 {
        *self
            .position_occurences
            .get(&self.position_hash)
            .unwrap_or(&1)
    }
    // TODO add check for insufficient material
    // TODO improve performance for use in engine.rs
    pub fn get_gamestate(&self) -> GameState {
        let legal_move_len = self.legal_moves.len();
        let is_in_check = self.position.is_in_check();
        let occurence_of_current_pos = self.get_occurences_of_current_position();

        // checkmate has to be checked for first, as it supercedes other states like the 50 move rule
        if is_in_check && legal_move_len == 0 {
            GameState::Checkmate
        } else if !is_in_check && legal_move_len == 0 {
            GameState::Stalemate
        } else if self.halfmove_count >= 100 {
            GameState::FiftyMove
        } else if occurence_of_current_pos >= 3 {
            GameState::Repetition
        } else if is_in_check {
            GameState::Check
        } else if false {
            //placeholder
            // check for insufficient material TODO
            GameState::InsufficientMaterial
        } else {
            GameState::Active
        }
    }

    pub fn is_in_check(&self) -> bool {
        self.position.is_in_check()
    }

    pub fn is_checkmate(&self) -> bool {
        self.legal_moves.is_empty() && self.position.is_in_check()
    }

    pub fn is_draw(&self) -> bool {
        (self.legal_moves.is_empty() && !self.position.is_in_check())
            || self.halfmove_count >= 100
            || self.get_occurences_of_current_position() >= 3
    }

    // gamestates that are draws
    pub fn gamestate_is_draw(&self, gamestate: GameState) -> bool {
        matches!(
            gamestate,
            GameState::Stalemate
                | GameState::FiftyMove
                | GameState::Repetition
                | GameState::InsufficientMaterial
        )
    }

    pub fn get_pos64(&self) -> &Pos64 {
        &self.position.pos64
    }
}

#[derive(Debug)]
pub struct Board {
    pub current_state: BoardState,
    pub state_history: Vec<BoardState>,
    transposition_table: engine::TranspositionTable,
}

impl Board {
    pub fn new() -> Self {
        let current_state = BoardState::new_starting();
        log::info!("New starting BoardState created");

        let mut state_history: Vec<BoardState> = Vec::new();
        log::info!("State history created");
        state_history.push(current_state.clone());

        let transposition_table = engine::TranspositionTable::new();
        log::info!("Transposition table created");
        log::info!("New Board created");
        Board {
            current_state,
            state_history,
            transposition_table,
        }
    }
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let current_state = BoardState::from_fen(fen)?;
        let mut state_history: Vec<BoardState> = Vec::new();
        state_history.push(current_state.clone());

        let transposition_table = engine::TranspositionTable::new();
        log::info!("New Board created from FEN: {}", fen);
        Ok(Board {
            current_state,
            state_history,
            transposition_table,
        })
    }

    pub fn to_fen(&self) -> String {
        self.current_state.to_fen()
    }

    pub fn branch(&self, _branch_state: Rc<BoardState>) -> Self {
        // TODO, clone from specific state in state_history. Will probably need to store data differently like position_occurences
        // probably will have to go through all position hashes after the branch node, and remove occurences one by one
        todo!()
    }

    pub fn make_move(&mut self, mv: &Move) -> Result<GameState, BoardStateError> {
        let next_state = self.current_state.next_state(mv)?;
        self.current_state = next_state;
        self.state_history.push(self.current_state.clone());

        let game_state = self.current_state.get_gamestate();
        //println!("FEN: {}", self.to_fen());

        Ok(game_state)
    }

    pub fn make_engine_move(&mut self, depth: i32) -> Result<GameState, BoardStateError> {
        let (eval, engine_move) =
            engine::choose_move(&self.current_state, depth, &mut self.transposition_table);
        let mv = *engine_move;
        log::info!("Engine move chosen: {:?} @ eval: {}", engine_move, eval);

        self.make_move(&mv)
    }

    pub fn unmake_move(&mut self) -> Result<Rc<BoardState>, BoardStateError> {
        todo!()
    }

    pub fn get_gamestate(&self) -> GameState {
        self.current_state.get_gamestate()
    }
}
