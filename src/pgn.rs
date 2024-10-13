// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
pub mod notation;
mod tag;
mod token;

use std::fmt;
use std::fs;
use std::path::Path;

use chrono::prelude::*;

use crate::board;
use crate::errors::PGNParseError;
use crate::log_and_return_error;
use crate::PieceColour;
use notation::*;
use tag::*;
use token::*;

#[derive(Debug)]
pub struct PGN {
    tags: Vec<Tag>,
    moves: Vec<Notation>,
}
impl fmt::Display for PGN {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl PGN {
    pub fn from_file(file_path: &Path) -> Result<Self, PGNParseError> {
        let pgn = match fs::read_to_string(file_path) {
            Ok(pgn) => pgn,
            Err(e) => log_and_return_error!(PGNParseError::FileError(e.to_string())),
        };
        Self::from_str(&pgn)
    }

    pub fn from_str(pgn: &str) -> Result<Self, PGNParseError> {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
        };
        let tokens = Tokens::from_pgn_str(pgn);
        new.tags = tokens.get_tags()?;
        new.moves = tokens.get_move_notations()?;
        if !new.check_required_tags() {
            log_and_return_error!(PGNParseError::InvalidTag(
                "PGN is missing required tags".to_string()
            ));
        } else {
            Ok(new)
        }
    }

    pub fn from_board(board: &board::Board) -> Self {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
        };

        new.tags.push(Tag::Event("Chess Oxide Export".to_string()));
        new.tags.push(Tag::Site("chess-oxide".to_string()));

        // set date tag
        let date_time = Local::now();
        let date = date_time.format("%Y.%m.%d").to_string();
        new.tags.push(Tag::Date(date));

        new.tags.push(Tag::Round("?".to_string()));
        new.tags.push(Tag::White("?".to_string()));
        new.tags.push(Tag::Black("?".to_string()));

        // set result tag based on game state
        let gs = board.get_current_gamestate();
        if gs.is_draw() {
            new.tags.push(Tag::Result("1/2-1/2".to_string()));
        } else if gs.is_game_over() {
            if board.get_current_state().side_to_move == PieceColour::White {
                new.tags.push(Tag::Result("0-1".to_string()));
            } else {
                new.tags.push(Tag::Result("1-0".to_string()));
            }
        } else {
            new.tags.push(Tag::Result("*".to_string()));
        }
        // set a custom field for the FEN of starting position, state_history[0] is guaranteed to be initialised
        new.tags.push(Tag::CustomTag(CustomTag::new(
            "FEN",
            &board.get_starting_state().to_fen().to_string(),
        )));

        new.moves = board.move_history_notation();

        new
    }

    pub fn to_string(&self) -> String {
        let mut sorted_tags = self.tags.to_vec();
        sorted_tags.sort();

        let mut pgn = String::new();
        for tag in &sorted_tags {
            pgn.push_str(&format!("{}\n", tag));
        }
        pgn.push('\n');
        // wrap lines at 80 characters
        let mut chars_since_newline = 0;
        for (i, mv) in self.moves.iter().enumerate() {
            if chars_since_newline >= 80 {
                pgn.push('\n');
                chars_since_newline = 0;
            }
            if i % 2 == 0 {
                let str = format!("{}.", i / 2 + 1);
                pgn.push_str(&str);
                chars_since_newline += str.len();
            }
            let mv_str = mv.to_string();
            pgn.push_str(&format!("{} ", mv_str));
            chars_since_newline += mv_str.len() + 1;
        }
        let termination_indicator = match self.tags.iter().find(|tag| match tag {
            Tag::Result(_) => true,
            _ => false,
        }) {
            Some(Tag::Result(result)) => result,
            _ => "*",
        };
        pgn.push_str(&format!(" {}\n", termination_indicator));

        pgn
    }

    pub fn to_board(&self) -> Result<board::Board, PGNParseError> {
        let mut board = board::Board::new();
        for notation in &self.moves {
            let mv = notation.to_move_with_context(&board.get_current_state())?;
            match board.make_move(&mv) {
                Ok(_) => {}
                Err(e) => log_and_return_error!(PGNParseError::NotationParseError(e.to_string())),
            }
        }
        Ok(board)
    }

    fn check_required_tags(&self) -> bool {
        let mut has_event = false;
        let mut has_site = false;
        let mut has_date = false;
        let mut has_round = false;
        let mut has_white = false;
        let mut has_black = false;
        let mut has_result = false;

        for tag in &self.tags {
            match tag {
                Tag::Event(_) => has_event = true,
                Tag::Site(_) => has_site = true,
                Tag::Date(_) => has_date = true,
                Tag::Round(_) => has_round = true,
                Tag::White(_) => has_white = true,
                Tag::Black(_) => has_black = true,
                Tag::Result(_) => has_result = true,
                _ => {}
            }
        }

        has_event && has_site && has_date && has_round && has_white && has_black && has_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgn_from_file() {
        let pgn = PGN::from_file(Path::new("test_data/test.pgn")).unwrap();
        println!("{:#?}", pgn);
        println!("{}", pgn.to_string());

        let from_board = board::Board::new();
        let from_board_pgn = PGN::from_board(&from_board);
        println!("{}", from_board_pgn.to_string());

        let b1 = pgn.to_board().unwrap();
        println!("{}", pgn.to_string());
        let pgn1 = PGN::from_board(&b1);
        println!("{}", pgn1.to_string());
        println!("{}", b1.get_current_gamestate());

        assert_eq!(pgn.tags.len(), 10);
        assert_eq!(pgn.moves.len(), 115);
    }
}
