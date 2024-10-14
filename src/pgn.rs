// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
pub mod notation;
mod tag;
mod token;

use std::fmt;
use std::fs;
use std::path::Path;

use chrono::prelude::*;

use crate::errors::PGNParseError;
use crate::log_and_return_error;
use crate::PieceColour;
use crate::{board, GameOverState, GameState};
use notation::*;
use tag::*;
use token::*;

enum PGNResult {
    WhiteWin,
    BlackWin,
    Draw,
    Undecided,
}
impl fmt::Display for PGNResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PGNResult::WhiteWin => write!(f, "1-0"),
            PGNResult::BlackWin => write!(f, "0-1"),
            PGNResult::Draw => write!(f, "1/2-1/2"),
            PGNResult::Undecided => write!(f, "*"),
        }
    }
}

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
        // set required tags to defaults if they are missing, using game termination marker as the Result tag if it is missing
        new.set_required_tags_defaults(tokens.get_game_termination());
        Ok(new)
    }

    pub fn from_board(board: &board::Board) -> Self {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
        };

        new.tags.push(Tag::Event("Chess Oxide".to_string()));
        new.tags.push(Tag::Site("chess-oxide".to_string()));

        // set date tag
        let date_time = Local::now();
        let date = date_time.format("%Y.%m.%d").to_string();
        new.tags.push(Tag::Date(date));

        new.tags.push(Tag::Round("?".to_string()));
        new.tags.push(Tag::White("?".to_string()));
        new.tags.push(Tag::Black("?".to_string()));

        // set result tag based on Board GameOverState
        new.tags
            .push(Tag::Result(match board.get_game_over_state() {
                None => PGNResult::Undecided.to_string(),
                Some(gos) => {
                    match gos {
                        GameOverState::WhiteResign => PGNResult::BlackWin.to_string(),
                        GameOverState::BlackResign => PGNResult::WhiteWin.to_string(),
                        GameOverState::AgreedDraw => PGNResult::Draw.to_string(),
                        GameOverState::Forced(gs) => {
                            if gs.is_win() {
                                // the side to move is the loser, the last move was the winning move
                                if board.get_side_to_move() == PieceColour::White {
                                    PGNResult::BlackWin.to_string()
                                } else {
                                    PGNResult::WhiteWin.to_string()
                                }
                            } else if gs.is_draw() {
                                PGNResult::Draw.to_string()
                            } else {
                                PGNResult::Undecided.to_string()
                            }
                        }
                    }
                }
            }));
        new.tags.push(Tag::SetUp("0".to_string()));
        //new.tags.push(Tag::FEN(board.get_starting_state().to_fen().to_string()));
        new.tags.push(Tag::Termination("UNIMPLEMENTED".to_string()));
        new.tags.push(Tag::Annotator("chess-oxide".to_string()));
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
        // unwrap is safe, the Result tag is required and set in all constructors
        let termination_indicator = if let Tag::Result(term) = self
            .tags
            .iter()
            .find(|tag| matches!(tag, Tag::Result(_)))
            .unwrap()
        {
            term
        } else {
            unreachable!("Result tag is required and set in all constructors, it will be found");
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
        //TODO when board can store more info set it here
        for tag in &self.tags {
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

    fn set_required_tags_defaults(&mut self, termination: Option<String>) {
        let mut missing_event = true;
        let mut missing_site = true;
        let mut missing_date = true;
        let mut missing_round = true;
        let mut missing_white = true;
        let mut missing_black = true;
        let mut missing_result = true;

        for tag in &self.tags {
            match tag {
                Tag::Event(_) => missing_event = false,
                Tag::Site(_) => missing_site = false,
                Tag::Date(_) => missing_date = false,
                Tag::Round(_) => missing_round = false,
                Tag::White(_) => missing_white = false,
                Tag::Black(_) => missing_black = false,
                Tag::Result(_) => missing_result = false,
                _ => {}
            }
        }

        if missing_event {
            self.tags.push(Tag::Event("Chess Oxide".to_string()));
        }
        if missing_site {
            self.tags.push(Tag::Site("chess-oxide".to_string()));
        }
        if missing_date {
            let date_time = Local::now();
            let date = date_time.format("%Y.%m.%d").to_string();
            self.tags.push(Tag::Date(date));
        }
        if missing_round {
            self.tags.push(Tag::Round("?".to_string()));
        }
        if missing_white {
            self.tags.push(Tag::White("?".to_string()));
        }
        if missing_black {
            self.tags.push(Tag::Black("?".to_string()));
        }
        if missing_result {
            self.tags.push(Tag::Result(
                termination.unwrap_or_else(|| PGNResult::Undecided.to_string()),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgn_from_file() {
        let pgn = PGN::from_file(Path::new("test_data/test.pgn")).unwrap();
        println!("{}", pgn.to_string());

        let b1 = pgn.to_board().unwrap();
        let pgn1 = PGN::from_board(&b1);
        let b2 = pgn1.to_board().unwrap();
        let pgn2 = PGN::from_board(&b2);
        println!("{}", pgn1.to_string());
        println!("{}", pgn2.to_string());
        assert_eq!(pgn1.to_string(), pgn2.to_string());
        assert_eq!(
            b1.get_current_state().board_hash,
            b2.get_current_state().board_hash
        );

        assert_eq!(pgn.tags.len(), 10);
        assert_eq!(pgn.moves.len(), 115);
    }
}
