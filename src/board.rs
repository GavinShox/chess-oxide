use core::fmt;
use std::{ rc::Rc, collections::HashMap };

use crate::engine;
use crate::position::*;
use crate::movegen::*;
use crate::util;
use crate::errors::BoardStateError;
use crate::errors::FenParseError;

pub trait Player {
    fn get_move(&self, _: &BoardState) -> Move;
}

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
    position: Position,
    position_hash: u64,
    move_count: u32,
    halfmove_count: u32,
    position_occurences: HashMap<PositionHash, u8>,
}

impl BoardState {
    pub fn new_starting() -> Self {
        let position = Position::new_starting();
        let position_hash: PositionHash = position.pos_hash();
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().copied().collect();
        let mut position_occurences = HashMap::new();
        *position_occurences.entry(position_hash).or_insert(0) += 1;
        BoardState {
            position,
            move_count: 0,
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
        let position_hash: PositionHash = position.pos_hash();
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().copied().collect();
        let mut position_occurences = HashMap::new();
        *position_occurences.entry(position_hash).or_insert(0) += 1;
        
        // handle the final two fields of the FEN vector
        let Ok(halfmove_count) = fen_vec[4].parse::<u32>() else { return Err(FenParseError(format!("Error parsing halfmove count: {}", fen_vec[4]))) };
        let Ok(move_count) = fen_vec[5].parse::<u32>() else { return Err(FenParseError(format!("Error parsing move count: {}", fen_vec[5]))) };

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
    
        fen_str
    }

    pub fn last_move_as_notation(&self) -> String {
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
            MoveType::Promotion(promotion_type) => format!("{}={}", notation_to, get_piece_str(promotion_type)),
            MoveType::Castle(castle_move) => if castle_move.rook_from.abs_diff(castle_move.rook_to) == 3 {
                "O-O-O".to_string()
            } else {
                "O-O".to_string()
            },
            MoveType::DoublePawnPush => notation_to,
            MoveType::PawnPush => notation_to,
            MoveType::Capture(_) => format!("{}x{}", piece_str, notation_to),
            MoveType::Normal => format!("{}{}", piece_str, notation_to),
            MoveType::None => "".to_string(),
        };
        return if self.get_gamestate() == GameState::Checkmate {
            format!("{}#", notation)
        } else if self.get_gamestate() == GameState::Check {
            format!("{}+", notation)
        } else {
            notation
        };
    }

    pub fn next_state(&self, mv: &Move) -> Result<Self, BoardStateError> {
        if mv == &NULL_MOVE {
            return Err(BoardStateError::NullMove("&NULL_MOVE was passed as an argument to BoardState::next_state()".to_string()));
        }
        if !self.legal_moves.contains(mv) {
            return Err(BoardStateError::IllegalMove(format!("{:?} is not a legal move", mv)));
        }

        let current_game_state = self.get_gamestate();

        if
            current_game_state == GameState::Checkmate ||
            current_game_state == GameState::Stalemate ||
            current_game_state == GameState::FiftyMove ||
            current_game_state == GameState::Repetition
        {
            return Err(BoardStateError::NoLegalMoves(current_game_state));
        }

        let position = self.position.new_position(mv);
        let position_hash = position.pos_hash();
        let side_to_move = position.side;
        let last_move = *mv;
        // deref all legal moves
        let legal_moves = position.get_legal_moves().into_iter().copied().collect();

        let move_count = if side_to_move == PieceColour::White {
            self.move_count + 1
        } else {
            self.move_count
        };

        let halfmove_reset = matches!(mv.move_type, MoveType::PawnPush | MoveType::DoublePawnPush | MoveType::Capture(_));
        let halfmove_count = if halfmove_reset { 0 } else { self.halfmove_count + 1 };

        let mut position_occurences = self.position_occurences.clone();
        *position_occurences.entry(position_hash).or_insert(0) += 1;

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
        *self.position_occurences.get(&self.position_hash).unwrap_or(&1)
    }
    // TODO add check for insufficient material
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
        } else if false { //placeholder
            // check for insufficient material TODO
            GameState::InsufficientMaterial
        } else {
            GameState::Active
        }
    }

    // gamestates that are draws
    pub fn gamestate_is_draw(&self, gamestate: GameState) -> bool {
        matches!(gamestate, GameState::Stalemate | GameState::FiftyMove | GameState::Repetition | GameState::InsufficientMaterial)
    }

    pub fn get_pos64(&self) -> &Pos64 {
        &self.position.pos64
    }

    pub fn is_in_check(&self) -> bool {
        self.position.is_in_check()
    }
}


#[derive(Debug)]
pub struct Board {
    pub current_state: BoardState,
    pub state_history: Vec<BoardState>,
}


impl Board {
    pub fn new() -> Self {
        let current_state = BoardState::new_starting();

        let mut state_history: Vec<BoardState> = Vec::new();
        state_history.push(current_state.clone());

        Board {
            current_state,
            state_history,
        }
    }
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let current_state = BoardState::from_fen(fen)?;
        let mut state_history: Vec<BoardState> = Vec::new();
        state_history.push(current_state.clone());

        Ok(Board {
            current_state,
            state_history,
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

        Ok(game_state)
    }

    pub fn make_engine_move(&mut self, depth: i32) -> Result<GameState, BoardStateError> {
        let engine_move = engine::choose_move(&self.current_state, depth);
        let mv = *engine_move.1;

        self.make_move(&mv)
    }

    pub fn unmake_move(&mut self) -> Result<Rc<BoardState>, BoardStateError> {
        todo!()
    }

    pub fn get_gamestate(&self) -> GameState {
        self.current_state.get_gamestate()
    }
}