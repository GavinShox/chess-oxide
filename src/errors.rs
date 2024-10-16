use std::error;
use std::fmt;

use crate::{GameOverState, GameState};

#[derive(Debug)]
pub enum BoardStateError {
    IllegalMove(String),
    NullMove(String),
    NoLegalMoves(GameState),
    LazyIncompatiblity(String),
    GameOver(GameOverState),
}

impl fmt::Display for BoardStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardStateError::IllegalMove(s) => write!(f, "Illegal move: {}", s),
            BoardStateError::NullMove(s) => write!(f, "Null move: {}", s),
            BoardStateError::NoLegalMoves(gs) => write!(f, "No legal moves in GameState: {}", gs),
            BoardStateError::LazyIncompatiblity(s) => {
                write!(f, "Lazy legal move generation incompatibility: {}", s)
            }
            BoardStateError::GameOver(gos) => write!(f, "Game over: {:?}", gos),
        }
    }
}

impl error::Error for BoardStateError {}

#[derive(Debug)]
pub struct FenParseError(pub String);

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing FEN: {}", self.0)
    }
}

impl error::Error for FenParseError {}

#[derive(Debug)]
pub enum PGNParseError {
    InvalidTag(String),
    NotationParseError(String),
    FileError(String),
    MoveNotFound(String),
}

impl fmt::Display for PGNParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PGNParseError::InvalidTag(s) => write!(f, "Invalid tag: {}", s),
            PGNParseError::NotationParseError(s) => write!(f, "Error parsing notation: {}", s),
            PGNParseError::FileError(s) => write!(f, "Error reading file: {}", s),
            PGNParseError::MoveNotFound(s) => write!(f, "Move not found: {}", s),
        }
    }
}

impl error::Error for PGNParseError {}
