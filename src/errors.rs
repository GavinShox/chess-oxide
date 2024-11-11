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
    InvalidInput(String),
}

impl fmt::Display for BoardStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IllegalMove(s) => write!(f, "Illegal move: {}", s),
            Self::NullMove(s) => write!(f, "Null move: {}", s),
            Self::NoLegalMoves(gs) => write!(f, "No legal moves in GameState: {}", gs),
            Self::LazyIncompatiblity(s) => {
                write!(f, "Lazy legal move generation incompatibility: {}", s)
            }
            Self::GameOver(gos) => write!(f, "Game over: {:?}", gos),
            Self::InvalidInput(s) => write!(f, "Invalid input: {}", s),
        }
    }
}

impl error::Error for BoardStateError {}

#[derive(Debug)]
pub enum FenParseError {
    InvalidFen(String),
    VariantIncompatible(String),
}

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidFen(s) => write!(f, "Invalid FEN: {}", s),
            Self::VariantIncompatible(s) => write!(f, "Variant incompatibility: {}", s),
        }
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
            Self::InvalidTag(s) => write!(f, "Invalid tag: {}", s),
            Self::NotationParseError(s) => write!(f, "Error parsing notation: {}", s),
            Self::FileError(s) => write!(f, "Error reading file: {}", s),
            Self::MoveNotFound(s) => write!(f, "Move not found: {}", s),
        }
    }
}

impl error::Error for PGNParseError {}
