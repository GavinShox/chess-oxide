use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BoardStateError {
    IllegalMove(String),
    NullMove(String),
    NoLegalMoves,
}

impl fmt::Display for BoardStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardStateError::IllegalMove(s) => write!(f, "Illegal move: {}", s),
            BoardStateError::NullMove(s) => write!(f, "Null move: {}", s),
            BoardStateError::NoLegalMoves => write!(f, "No legal moves"),
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