// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
pub mod notation;
pub mod tag;
mod token;

use std::fmt;
use std::str::FromStr;

use chrono::prelude::*;

use crate::errors::PGNParseError;
use crate::fen::{FEN, STD_STARTING_FEN_STR};
use crate::movegen::PieceColour;
use crate::{board, GameOverState};
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
            Self::WhiteWin => write!(f, "1-0"),
            Self::BlackWin => write!(f, "0-1"),
            Self::Draw => write!(f, "1/2-1/2"),
            Self::Undecided => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PGN {
    tags: Vec<Tag>,
    moves: Vec<Notation>,
}

impl FromStr for PGN {
    type Err = PGNParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut new = Self {
            tags: Vec::new(),
            moves: Vec::new(),
        };
        let tokens = Tokens::from_pgn_str(s);
        new.tags = tokens.get_tags()?;
        new.moves = tokens.get_move_notations()?;
        // set required tags to defaults if they are missing, using game termination marker as the Result tag if it is missing
        new.set_required_tags_defaults(tokens.get_game_termination());
        Ok(new)
    }
}

impl From<&board::Board> for PGN {
    fn from(board: &board::Board) -> Self {
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
            .push(Tag::Result(board.get_game_over_state().map_or_else(
                || PGNResult::Undecided.to_string(),
                |gos| match gos {
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
                },
            )));
        match board.variant() {
            board::Variant::Standard => {
                let fen = FEN::from(board.get_current_state());
                if fen.to_string() != STD_STARTING_FEN_STR {
                    new.tags.push(Tag::Variant("Standard".to_string()));
                    new.tags.push(Tag::SetUp("1".to_string()));
                    new.tags.push(Tag::FEN(fen.to_string()));
                } else {
                    new.tags.push(Tag::SetUp("0".to_string()));
                }
            }
            board::Variant::Chess960 => {
                let fen = FEN::from(board.get_current_state());
                new.tags.push(Tag::Variant("Chess960".to_string()));
                new.tags.push(Tag::SetUp("1".to_string()));
                new.tags.push(Tag::FEN(fen.to_string()));
            }
        }

        new.tags.push(Tag::Termination("UNIMPLEMENTED".to_string()));
        new.tags.push(Tag::Annotator("chess-oxide".to_string()));
        new.moves = board.move_history_notation();

        new
    }
}

impl fmt::Display for PGN {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        let Tag::Result(termination_indicator) = self
            .tags
            .iter()
            .find(|tag| matches!(tag, Tag::Result(_)))
            .unwrap()
        else {
            unreachable!("Result tag is required and set in all constructors, it will be found");
        };
        pgn.push_str(&format!(" {}\n", termination_indicator));

        write!(f, "{}", pgn)
    }
}

impl PGN {
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn moves(&self) -> &Vec<Notation> {
        &self.moves
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
// todo add better tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_and_return_error;
    use std::fs;
    use std::path::Path;

    // Moved in testing for now TODO see if this belongs in PGN struct or should user handle file parsing to str
    fn from_file(file_path: &Path) -> Result<PGN, PGNParseError> {
        let pgn = match fs::read_to_string(file_path) {
            Ok(pgn) => pgn,
            Err(e) => log_and_return_error!(PGNParseError::FileError(e.to_string())),
        };
        PGN::from_str(&pgn)
    }

    #[test]
    fn test_pgn_from_file() {
        let pgn = from_file(Path::new("test_data/test.pgn")).unwrap();
        println!("{}", pgn);

        let b1 = board::Board::try_from(pgn.clone()).unwrap();
        let pgn1 = PGN::from(&b1);
        let b2 = board::Board::try_from(pgn1.clone()).unwrap();
        let pgn2 = PGN::from(&b2);
        println!("{}", pgn1);
        println!("{}", pgn2);
        assert_eq!(pgn1.to_string(), pgn2.to_string());
        assert_eq!(
            b1.get_current_state().board_hash,
            b2.get_current_state().board_hash
        );

        assert_eq!(pgn.tags.len(), 10);
        assert_eq!(pgn.moves.len(), 115);
    }
}
