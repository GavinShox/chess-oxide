// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
mod notation;
mod tag;
mod token;

use std::fmt;
use std::fs;
use std::path::Path;

use crate::board;
use crate::errors::PGNParseError;
use crate::log_and_return_error;
use notation::*;
use tag::*;
use token::*;

#[derive(Debug)]
struct PGN {
    tags: Vec<Tag>,
    moves: Vec<Notation>,
}
impl PGN {
    fn from_file(file_path: &Path) -> Result<Self, PGNParseError> {
        let pgn = match fs::read_to_string(file_path) {
            Ok(pgn) => pgn,
            Err(e) => log_and_return_error!(PGNParseError::FileError(e.to_string())),
        };
        Self::from_str(&pgn)
    }

    fn from_str(pgn: &str) -> Result<Self, PGNParseError> {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
        };
        let tokens = Tokens::from_pgn_str(pgn);
        new.tags = tokens.get_tags()?;
        new.moves = tokens.get_move_notations()?;
        // new.termination_marker = tokens.get_termination_marker()?;
        Ok(new)
    }

    fn is_valid(&self) -> bool {
        todo!()
    }

    fn from_board(board: &board::Board) -> Self {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
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
    use super::*;

    #[test]
    fn test_pgn_from_file() {
        let pgn = PGN::from_file(Path::new("test.pgn")).unwrap();
        println!("{:#?}", pgn);

        assert_eq!(pgn.tags.len(), 10);
        assert_eq!(pgn.moves.len(), 115);
    }
}
