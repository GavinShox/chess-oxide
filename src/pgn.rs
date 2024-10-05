// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
mod notation;
mod tag;
mod token;

use std::fmt;

use crate::board;
use crate::errors::PGNParseError;
use notation::*;
use tag::*;
use token::*;

struct PGN {
    pgn_string: String,
    tokens: Tokens,
    tags: Vec<Tag>,
    moves: Vec<Notation>,
    termination_marker: TerminationMarker,
}
impl PGN {
    fn new(pgn: &str) -> Result<Self, PGNParseError> {
        let mut new = Self {
            pgn_string: pgn.to_string(),
            tokens: Tokens::new(),
            tags: Vec::new(),
            moves: Vec::new(),
            termination_marker: TerminationMarker::InProgress,
        };
        new.parse()?;
        Ok(new)
    }

    fn parse(&mut self) -> Result<(), PGNParseError> {
        self.tags = self.tokens.get_tags()?;
        //self.moves = self.tokens.get_moves()?;
        //self.termination_marker = self.tokens.get_termination_marker()?;
        Ok(())
    }

    fn from_board(board: &board::Board) -> Self {
        let mut new = Self {
            pgn_string: String::new(),
            tokens: Tokens::new(),
            tags: Vec::new(),
            moves: Vec::new(),
            termination_marker: TerminationMarker::InProgress,
        };

        new
    }

    fn to_board(&self) -> board::Board {
        board::Board::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
}
