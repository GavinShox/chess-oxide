// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
mod notation;
mod tag;
mod token;

use std::fmt;
use std::fs;

use crate::board;
use crate::errors::PGNParseError;
use notation::*;
use tag::*;
use token::*;

struct PGN {
    tags: Vec<Tag>,
    moves: Vec<Notation>,
    termination_marker: TerminationMarker,
}
impl PGN {
    fn from_str(pgn: &str) -> Result<Self, PGNParseError> {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
            termination_marker: TerminationMarker::InProgress,
        };
        let tokens = Tokens::from_pgn_str(pgn);
        new.tags = tokens.get_tags()?;
        // new.moves = tokens.get_moves()?;
        // new.termination_marker = tokens.get_termination_marker()?;
        Ok(new)
    }

    fn from_board(board: &board::Board) -> Self {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
            termination_marker: TerminationMarker::InProgress,
        };

        new
    }

    fn to_string(&self) -> String {
        todo!()
    }

    fn to_board(&self) -> board::Board {
        board::Board::new()
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::fs;

    use super::*;

}
