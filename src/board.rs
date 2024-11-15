use core::fmt;

use ahash;
use log;

use crate::engine;
use crate::errors::BoardStateError;
use crate::errors::PGNParseError;
use crate::fen::FEN;
use crate::log_and_return_error;
use crate::movegen::*;
use crate::pgn;
use crate::pgn::notation::Notation;
use crate::pgn::tag::Tag;
use crate::position::*;
use crate::transposition;
use crate::util;
use crate::zobrist;
use crate::zobrist::PositionHash;

const DEFAULT_HALFMOVE_COUNT: u32 = 0;
const DEFAULT_MOVE_COUNT: u32 = 1; // movecount starts at 1

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Self::Stalemate | Self::FiftyMove | Self::Repetition | Self::InsufficientMaterial
        )
    }
    // gamestates that are wins
    #[inline]
    pub fn is_win(&self) -> bool {
        matches!(self, Self::Checkmate)
    }
    // gamestates that end game
    #[inline]
    pub fn is_game_over(&self) -> bool {
        self.is_win() || self.is_draw()
    }
}
// String representation of GameState
impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state_str = match self {
            Self::Check => "Check",
            Self::Checkmate => "Checkmate",
            Self::Stalemate => "Stalemate",
            Self::Repetition => "Repetition",
            Self::FiftyMove => "Fifty Move Draw",
            Self::InsufficientMaterial => "Insufficient Material",
            Self::Active => "",
        };
        write!(f, "{}", state_str)
    }
}

#[derive(Debug, Clone)]
pub struct BoardState {
    pub side_to_move: PieceColour,
    pub last_move: Option<Move>,
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

impl From<FEN> for BoardState {
    fn from(fen: FEN) -> Self {
        let pos = Position::from(fen);
        Self::from_parts(pos, fen.halfmove_count(), fen.move_count())
    }
}

impl BoardState {
    pub fn new_starting() -> Self {
        let position = Position::new_starting();
        log::info!("New starting Position created");
        Self::from_parts(position, DEFAULT_HALFMOVE_COUNT, DEFAULT_MOVE_COUNT)
    }

    pub fn new_chess960() -> Self {
        let position = Position::new_chess960_random();
        log::info!("New random Chess960 Position created");
        Self::from_parts(position, DEFAULT_HALFMOVE_COUNT, DEFAULT_MOVE_COUNT)
    }

    pub fn new_chess960_from_num(position_number: usize) -> Result<Self, BoardStateError> {
        if position_number > 959 {
            let err = BoardStateError::InvalidInput(format!(
                "Chess960 position number {} is out of range. Must be between 0 and 959",
                position_number
            ));
            log_and_return_error!(err)
        }
        let position = Position::new_chess960_number_derive(position_number);
        log::info!(
            "New Chess960 Position created from position number: {}",
            position_number
        );

        Ok(Self::from_parts(
            position,
            DEFAULT_HALFMOVE_COUNT,
            DEFAULT_MOVE_COUNT,
        ))
    }

    pub(crate) fn from_parts(position: Position, halfmove_count: u32, move_count: u32) -> Self {
        let position_hash: PositionHash = position.pos_hash();
        let board_hash = zobrist::board_state_hash(position_hash, 1, halfmove_count);
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().cloned().collect();
        let mut position_occurences = ahash::AHashMap::default();
        position_occurences.insert(position_hash, 1);
        log::info!(
            "New BoardState created from position: {} halfmove_count: {} move_count: {}",
            util::hash_to_string(position_hash),
            halfmove_count,
            move_count
        );
        BoardState {
            position,
            move_count,
            halfmove_count,
            position_hash,
            board_hash,
            side_to_move,
            last_move: None,
            legal_moves,
            position_occurences,
            lazy_legal_moves: false,
        }
    }

    pub(crate) fn position(&self) -> &Position {
        &self.position
    }

    pub fn halfmove_count(&self) -> u32 {
        self.halfmove_count
    }

    pub fn move_count(&self) -> u32 {
        self.move_count
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
    pub fn next_state_unchecked(&self, mv: &Move) -> Self {
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
        let last_move = Some(*mv);
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
        let last_move = Some(*mv);
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

    // fn gen_legal_moves(&mut self) {
    //     self.legal_moves = self
    //         .position
    //         .get_legal_moves()
    //         .into_iter()
    //         .cloned()
    //         .collect();
    // }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverState {
    WhiteResign,
    BlackResign,
    AgreedDraw,
    Forced(GameState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    #[default]
    Standard,
    Chess960,
}

impl Variant {
    #[inline]
    pub fn is_standard(&self) -> bool {
        matches!(self, Self::Standard)
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    variant: Variant,
    current_state: BoardState,
    state_history: Vec<BoardState>,
    move_history: Vec<Move>,
    game_over_state: Option<GameOverState>,
    transposition_table: transposition::TranspositionTable,
    detatched_idx: Option<usize>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl From<FEN> for Board {
    fn from(fen: FEN) -> Self {
        let current_state = BoardState::from(fen);
        let state_history: Vec<BoardState> = vec![current_state.clone()];
        let transposition_table = transposition::TranspositionTable::new();
        // TODO gos
        log::info!("New Board created from FEN: {}", fen.to_string());
        Board {
            variant: Variant::Standard,
            current_state,
            state_history,
            move_history: Vec::new(),
            game_over_state: None,
            transposition_table,
            detatched_idx: None,
        }
    }
}

impl TryFrom<pgn::PGN> for Board {
    type Error = PGNParseError;

    fn try_from(pgn: pgn::PGN) -> Result<Self, PGNParseError> {
        let fen_tag = pgn.tags().iter().find(|tag| matches!(tag, Tag::FEN(_)));

        let mut board = match fen_tag {
            Some(Tag::FEN(fen_str)) => {
                let fen = fen_str.parse::<FEN>();
                match fen {
                    Ok(fen) => Board::from(fen),
                    Err(e) => {
                        log_and_return_error!(PGNParseError::NotationParseError(e.to_string()))
                    }
                }
            }
            _ => Board::new(),
        };

        for notation in pgn.moves() {
            let mv = notation.to_move_with_context(board.get_current_state())?;
            match board.make_move(&mv) {
                Ok(_) => {}
                Err(e) => log_and_return_error!(PGNParseError::NotationParseError(e.to_string())),
            }
        }
        //TODO when board can store more info set it here
        for tag in pgn.tags() {
            if let Tag::Result(result) = tag {
                match result.as_str() {
                    // these will be ignored if game over state is already set in Board, priority is given to Forced(GameState) FIXME this needs to be clearer
                    "1-0" => board.set_resign(PieceColour::Black),
                    "0-1" => board.set_resign(PieceColour::White),
                    "1/2-1/2" => board.set_draw(),
                    _ => {}
                }
            }
        }
        Ok(board)
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
            variant: Variant::Standard,
            current_state,
            state_history,
            move_history: Vec::new(),
            game_over_state: None,
            transposition_table,
            detatched_idx: None,
        }
    }

    pub fn new_chess960() -> Self {
        let current_state = BoardState::new_chess960();
        let mut state_history: Vec<BoardState> = Vec::new();
        log::info!("State history created");
        state_history.push(current_state.clone());

        let transposition_table = transposition::TranspositionTable::new();
        log::info!("Transposition table created");
        log::info!("New Chess960 variant Board created");
        Board {
            variant: Variant::Chess960,
            current_state,
            state_history,
            move_history: Vec::new(),
            game_over_state: None,
            transposition_table,
            detatched_idx: None,
        }
    }

    pub fn new_chess960_from_num(position_number: usize) -> Result<Self, BoardStateError> {
        let current_state = BoardState::new_chess960_from_num(position_number)?;
        let mut state_history: Vec<BoardState> = Vec::new();
        log::info!("State history created");
        state_history.push(current_state.clone());

        let transposition_table = transposition::TranspositionTable::new();
        log::info!("Transposition table created");
        log::info!(
            "New Chess960 variant Board created from position number: {}",
            position_number
        );
        Ok(Board {
            variant: Variant::Chess960,
            current_state,
            state_history,
            move_history: Vec::new(),
            game_over_state: None,
            transposition_table,
            detatched_idx: None,
        })
    }

    pub fn set_resign(&mut self, side: PieceColour) {
        let gos = match side {
            PieceColour::White => GameOverState::WhiteResign,
            PieceColour::Black => GameOverState::BlackResign,
        };
        if self.game_over_state.is_none() {
            self.game_over_state = Some(gos);
        } else {
            log::warn!("Game over state already set, ignoring set_resign");
        }
    }

    pub fn set_draw(&mut self) {
        if self.game_over_state.is_none() {
            self.game_over_state = Some(GameOverState::AgreedDraw);
        } else {
            log::warn!("Game over state already set, ignoring set_draw");
        }
    }

    pub fn get_starting_state(&self) -> &BoardState {
        // first element in state_history is guarenteed to be initialised as starting BoardState
        &self.state_history[0]
    }

    pub fn get_side_to_move(&self) -> PieceColour {
        self.current_state.side_to_move
    }

    pub fn get_current_state(&self) -> &BoardState {
        &self.current_state
    }

    pub fn get_current_move_count(&self) -> u32 {
        self.current_state.move_count
    }

    pub fn get_current_halfmove_count(&self) -> u32 {
        self.current_state.halfmove_count
    }

    pub fn get_state_history(&self) -> &Vec<BoardState> {
        &self.state_history
    }

    pub fn get_game_over_state(&self) -> Option<GameOverState> {
        if self.is_detatched() {
            return None;
        } else {
            self.game_over_state
        }
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn is_detatched(&self) -> bool {
        self.detatched_idx.is_some()
    }

    pub fn detatched_idx(&self) -> Option<usize> {
        self.detatched_idx
    }

    pub fn make_move(&mut self, mv: &Move) -> Result<GameState, BoardStateError> {
        if let Some(idx) = self.detatched_idx {
            let err = BoardStateError::Detatched(format!(
                "Detatched from current boardstate at index {}. Cannot make move",
                idx
            ));
            log_and_return_error!(err)
        }
        if let Some(gos) = self.game_over_state {
            let err = BoardStateError::GameOver(gos);
            log_and_return_error!(err)
        }
        let next_state = self.current_state.next_state(mv)?;
        self.current_state = next_state;
        self.state_history.push(self.current_state.clone());
        self.move_history.push(*mv);

        let game_state = self.current_state.get_gamestate();
        if game_state.is_game_over() {
            self.game_over_state = Some(GameOverState::Forced(game_state));
        }
        log::info!("Move made: {:?}", mv);
        Ok(game_state)
    }

    pub fn make_engine_move(&mut self, depth: u8) -> Result<(GameState, i32), BoardStateError> {
        if let Some(idx) = self.detatched_idx {
            let err = BoardStateError::Detatched(format!(
                "Detatched from current boardstate at index {}. Cannot make engine move",
                idx
            ));
            log_and_return_error!(err)
        }
        if let Some(gos) = self.game_over_state {
            let err = BoardStateError::GameOver(gos);
            log_and_return_error!(err)
        }
        let (eval, engine_move) =
            engine::choose_move(&self.current_state, depth, &mut self.transposition_table);
        let mv = *engine_move;
        match self.make_move(&mv) {
            Ok(gs) => Ok((gs, eval)),
            Err(e) => Err(e),
        }
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

    pub fn last_move_notation(&self) -> Option<Notation> {
        if let Some(idx) = self.detatched_idx {
            if idx == 0 {
                return None;
            } else {
                return Some(self.move_history_notation()[idx - 1].clone());
            }
        } else if self.move_history_notation().is_empty() {
            return None;
        }
        Some(self.move_history_notation().last().unwrap().clone())
    }

    pub fn last_move_string_notation(&self) -> String {
        if let Some(n) = self.last_move_notation() {
            n.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn checkout_state(&mut self, bs: &BoardState) -> Result<(), BoardStateError> {
        if self.state_history.contains(bs) {
            let index = self.state_history.iter().position(|x| *x == *bs).unwrap();
            self.current_state = self.state_history[index].clone();
            // not detatched if index is the latest state
            self.detatched_idx = if index + 1 == self.state_history.len() {
                None
            } else {
                Some(index)
            };
            Ok(())
        } else {
            let err = BoardStateError::NotFound(format!(
                "BoardState ({}) not found in state history",
                bs.board_hash
            ));
            log_and_return_error!(err)
        }
    }

    // returns true if board is still detatched, otherwise false
    pub fn checkout_next(&mut self) -> bool {
        if let Some(idx) = self.detatched_idx {
            if idx + 1 == self.state_history.len() - 1 {
                self.checkout_latest_state();
                false
            } else {
                self.current_state = self.state_history[idx + 1].clone();
                self.detatched_idx = Some(idx + 1);
                true
            }
        } else {
            false
        }
    }

    // returns true if board is still detatched, otherwise false
    pub fn checkout_prev(&mut self) -> bool {
        if let Some(idx) = self.detatched_idx {
            if idx > 0 {
                self.current_state = self.state_history[idx - 1].clone();
                self.detatched_idx = Some(idx - 1);
            }
            true
        } else if self.state_history.len() > 1 {
            let idx = self.state_history.len() - 2; // -2 as we want the second last state not the last (current) state
            self.detatched_idx = Some(idx);
            self.current_state = self.state_history[idx].clone();
            true
        } else {
            false // starting state is only state
        }
    }

    pub fn checkout_latest_state(&mut self) {
        self.detatched_idx = None;
        self.current_state = self.state_history.last().unwrap().clone();
    }

    pub fn checkout_starting_state(&mut self) {
        self.detatched_idx = Some(0);
        self.current_state = self.state_history[0].clone();
    }

    pub fn find_states_by_notation(&self, notation: &str) -> Vec<&BoardState> {
        let mut state_iter = self.state_history.iter();
        state_iter.next(); // skip starting state
        let mut states = Vec::new();
        for (state, n) in state_iter.zip(self.move_history_notation()) {
            if n.to_string() == notation {
                states.push(state);
            }
        }
        states
    }

    pub fn get_current_gamestate(&self) -> GameState {
        self.current_state.get_gamestate()
    }
}
