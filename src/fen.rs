use std::fmt;
use std::str::FromStr;

use crate::board::BoardState;
use crate::errors::FenParseError;
use crate::log_and_return_error;
use crate::movegen::{MovegenFlags, Piece, PieceColour, PieceType, Square};
use crate::position::{Pos64, Position, ABOVE_BELOW};

#[derive(Debug, Clone, Copy)]
pub struct FEN {
    pos64: Pos64,
    side: PieceColour,
    movegen_flags: MovegenFlags,
    halfmove_count: u32,
    move_count: u32,
}

impl FromStr for FEN {
    type Err = FenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fen_vec: Vec<&str> = s.split(' ').collect();
        // check if the FEN string has the correct number of fields, accept the last two as optional with default values given in BoardState
        if fen_vec.len() < 4 || fen_vec.len() > 6 {
            return Err(FenParseError(format!(
                "Invalid number of fields in FEN string: {}. Expected at least 4, max 6",
                fen_vec.len()
            )));
        }
        let mut fen = Self::new();
        // first field of FEN defines the piece positions
        fen.parse_pos_field(fen_vec[0])?;
        // second filed of FEN defines which side it is to move, either 'w' or 'b'
        fen.parse_side_field(fen_vec[1])?;
        // third field of FEN defines castling flags
        fen.parse_castling_flags(fen_vec[2])?;
        // fourth field of FEN defines en passant flag, it gives notation of the square the pawn jumped over
        fen.parse_en_passant_flag(fen_vec[3])?;
        // set last two fields if they exist, otherwise default values are 0 and 1 already set in new()
        fen.parse_halfmove_move_count(fen_vec.get(4).copied(), fen_vec.get(5).copied())?;

        Ok(fen)
    }
}

impl fmt::Display for FEN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fen_str = String::new();

        let mut empty_count: i32 = 0;
        for (idx, sq) in self.pos64.iter().enumerate() {
            match sq {
                Square::Piece(p) => {
                    if empty_count > 0 {
                        fen_str.push_str(empty_count.to_string().as_str());
                        empty_count = 0;
                    }

                    match p.ptype {
                        PieceType::Pawn => match p.pcolour {
                            PieceColour::White => fen_str.push('P'),
                            PieceColour::Black => fen_str.push('p'),
                        },
                        PieceType::Knight => match p.pcolour {
                            PieceColour::White => fen_str.push('N'),
                            PieceColour::Black => fen_str.push('n'),
                        },
                        PieceType::Bishop => match p.pcolour {
                            PieceColour::White => fen_str.push('B'),
                            PieceColour::Black => fen_str.push('b'),
                        },
                        PieceType::Rook => match p.pcolour {
                            PieceColour::White => fen_str.push('R'),
                            PieceColour::Black => fen_str.push('r'),
                        },
                        PieceType::Queen => match p.pcolour {
                            PieceColour::White => fen_str.push('Q'),
                            PieceColour::Black => fen_str.push('q'),
                        },
                        PieceType::King => match p.pcolour {
                            PieceColour::White => fen_str.push('K'),
                            PieceColour::Black => fen_str.push('k'),
                        },
                    }
                }
                Square::Empty => {
                    empty_count += 1;
                }
            }

            // new rank insert '/', except when at last index, then only insert empty count if it's > 0
            if (idx + 1) % 8 == 0 {
                if empty_count > 0 {
                    fen_str.push_str(empty_count.to_string().as_str());
                    empty_count = 0;
                }
                if idx != 63 {
                    fen_str.push('/');
                }
            }
        }
        fen_str.push(' ');

        match self.side {
            PieceColour::White => fen_str.push('w'),
            PieceColour::Black => fen_str.push('b'),
        }
        fen_str.push(' ');

        if self.movegen_flags.white_castle_short {
            fen_str.push('K');
        }
        if self.movegen_flags.white_castle_long {
            fen_str.push('Q');
        }
        if self.movegen_flags.black_castle_short {
            fen_str.push('k');
        }
        if self.movegen_flags.black_castle_long {
            fen_str.push('q');
        }
        if !(self.movegen_flags.white_castle_short
            || self.movegen_flags.white_castle_long
            || self.movegen_flags.black_castle_short
            || self.movegen_flags.black_castle_long)
        {
            fen_str.push('-');
        }
        fen_str.push(' ');

        match self.movegen_flags.en_passant {
            Some(idx) => {
                if self.side == PieceColour::White {
                    fen_str.push_str(index_to_notation(idx - ABOVE_BELOW).as_str());
                } else {
                    fen_str.push_str(index_to_notation(idx + ABOVE_BELOW).as_str());
                }
            }
            None => {
                fen_str.push('-');
            }
        }
        fen_str.push(' ');
        fen_str.push_str(&format!("{} {}", self.halfmove_count, self.move_count));

        write!(f, "{}", fen_str)
    }
}

impl From<&BoardState> for FEN {
    fn from(board_state: &BoardState) -> Self {
        let mut fen = Self::from(board_state.position());
        fen.halfmove_count = board_state.halfmove_count();
        fen.move_count = board_state.move_count();
        fen
    }
}

impl From<&Position> for FEN {
    fn from(pos: &Position) -> Self {
        // default halfmove and move count to 0 and 1 respectively as Position does not store this information
        Self {
            pos64: pos.pos64,
            side: pos.side,
            movegen_flags: pos.movegen_flags,
            halfmove_count: 0,
            move_count: 1,
        }
    }
}

impl FEN {
    fn new() -> Self {
        Self {
            pos64: Pos64::default(),
            side: PieceColour::White,
            movegen_flags: MovegenFlags::default(),
            halfmove_count: 0,
            move_count: 1,
        }
    }

    pub fn pos64(&self) -> Pos64 {
        self.pos64
    }

    pub fn side(&self) -> PieceColour {
        self.side
    }

    pub fn movegen_flags(&self) -> MovegenFlags {
        self.movegen_flags
    }

    pub fn halfmove_count(&self) -> u32 {
        self.halfmove_count
    }

    pub fn move_count(&self) -> u32 {
        self.move_count
    }

    fn parse_pos_field(&mut self, field: &str) -> Result<(), FenParseError> {
        let mut pos = Pos64::default();
        let mut rank_start_idx = 0;
        // check for multiple kings, should be the only issue in terms of pieces on the board
        let mut wking_num = 0;
        let mut bking_num = 0;
        for rank in field.split('/') {
            // check to see if there is 8 squares in a rank.
            let mut square_count = 0;
            for c in rank.chars() {
                if c.is_ascii_digit() {
                    let num = c.to_digit(10).unwrap();
                    square_count += num;
                } else {
                    square_count += 1;
                }
            }
            if square_count != 8 {
                return Err(FenParseError(format!(
                    "Invalid number of squares in rank: {}. Expected 8, got {}",
                    rank, square_count
                )));
            }

            let mut i = 0;
            for c in rank.chars() {
                let square = match c {
                    'p' => Square::Piece(Piece {
                        pcolour: PieceColour::Black,
                        ptype: PieceType::Pawn,
                    }),
                    'P' => Square::Piece(Piece {
                        pcolour: PieceColour::White,
                        ptype: PieceType::Pawn,
                    }),
                    'r' => Square::Piece(Piece {
                        pcolour: PieceColour::Black,
                        ptype: PieceType::Rook,
                    }),
                    'R' => Square::Piece(Piece {
                        pcolour: PieceColour::White,
                        ptype: PieceType::Rook,
                    }),
                    'n' => Square::Piece(Piece {
                        pcolour: PieceColour::Black,
                        ptype: PieceType::Knight,
                    }),
                    'N' => Square::Piece(Piece {
                        pcolour: PieceColour::White,
                        ptype: PieceType::Knight,
                    }),
                    'b' => Square::Piece(Piece {
                        pcolour: PieceColour::Black,
                        ptype: PieceType::Bishop,
                    }),
                    'B' => Square::Piece(Piece {
                        pcolour: PieceColour::White,
                        ptype: PieceType::Bishop,
                    }),
                    'q' => Square::Piece(Piece {
                        pcolour: PieceColour::Black,
                        ptype: PieceType::Queen,
                    }),
                    'Q' => Square::Piece(Piece {
                        pcolour: PieceColour::White,
                        ptype: PieceType::Queen,
                    }),
                    'k' => {
                        bking_num += 1;
                        Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::King,
                        })
                    }
                    'K' => {
                        wking_num += 1;
                        Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::King,
                        })
                    }
                    x if x.is_ascii_digit() => {
                        for _ in 0..x.to_digit(10).unwrap() {
                            pos[i + rank_start_idx] = Square::Empty;
                            i += 1;
                        }
                        continue; // skip the below square assignment for pieces
                    }
                    other => {
                        let err = FenParseError(format!("Invalid char in first field: {}", other));
                        log_and_return_error!(err)
                    }
                };
                pos[i + rank_start_idx] = square;
                i += 1;
            }
            rank_start_idx += 8; // next rank
        }

        if wking_num > 1 || bking_num > 1 {
            let err = FenParseError(format!(
                "Multiple kings (white: {}, black: {}) in FEN field: {}",
                wking_num, bking_num, field
            ));
            log_and_return_error!(err)
        }

        self.pos64 = pos;
        Ok(())
    }

    fn parse_side_field(&mut self, field: &str) -> Result<(), FenParseError> {
        match field {
            "w" => {
                self.side = PieceColour::White;
            }
            "b" => {
                self.side = PieceColour::Black;
            }
            other => {
                return Err(FenParseError(format!(
                    "Invalid second field: {}. Expected 'w' or 'b'",
                    other
                )));
            }
        }
        Ok(())
    }

    fn parse_castling_flags(&mut self, field: &str) -> Result<(), FenParseError> {
        for c in field.chars() {
            match c {
                'q' => {
                    self.movegen_flags.black_castle_long = true;
                }
                'Q' => {
                    self.movegen_flags.white_castle_long = true;
                }
                'k' => {
                    self.movegen_flags.black_castle_short = true;
                }
                'K' => {
                    self.movegen_flags.white_castle_short = true;
                }
                '-' => {}
                other => {
                    return Err(FenParseError(format!(
                        "Invalid char in third field: {}",
                        other
                    )));
                }
            }
        }
        Ok(())
    }

    fn parse_en_passant_flag(&mut self, field: &str) -> Result<(), FenParseError> {
        if field != "-" {
            let ep_mv_idx = notation_to_index(field)?;

            // error if index is out of bounds. FEN defines the index behind the pawn that moved, so valid indexes are only 16->47 (excluded top and bottom two ranks)
            if !(16..=47).contains(&ep_mv_idx) {
                return Err(FenParseError(format!(
                    "Invalid en passant square: {}. Index is out of bounds",
                    field
                )));
            }

            // in our struct however, we store the idx of the pawn to be captured
            let ep_flag = if self.side == PieceColour::White {
                ep_mv_idx + ABOVE_BELOW
            } else {
                ep_mv_idx - ABOVE_BELOW
            };
            self.movegen_flags.en_passant = Some(ep_flag);

            // set polyglot en passant flag if the ep_flag is beside a pawn of side to move colour
            if self.pos64.polyglot_is_pawn_beside(ep_flag, self.side) {
                self.movegen_flags.polyglot_en_passant = Some(ep_flag);
            }
        }
        Ok(())
    }

    fn parse_halfmove_move_count(
        &mut self,
        hm_field: Option<&str>,
        m_field: Option<&str>,
    ) -> Result<(), FenParseError> {
        if let Some(hm) = hm_field {
            self.halfmove_count = match hm.parse::<u32>() {
                Ok(halfmove_count) => halfmove_count,
                Err(_) => {
                    let err = FenParseError(format!("Error parsing halfmove count: {}", hm));
                    log_and_return_error!(err)
                }
            };
        }
        if let Some(m) = m_field {
            self.move_count = match m.parse::<u32>() {
                Ok(move_count) => move_count,
                Err(_) => {
                    let err = FenParseError(format!("Error parsing move count: {}", m));
                    log_and_return_error!(err)
                }
            };
        }
        Ok(())
    }
}

#[inline]
fn notation_to_index(n: &str) -> Result<usize, FenParseError> {
    if n.len() != 2
        || n.chars().next().unwrap() < 'a'
        || n.chars().next().unwrap() > 'h'
        || n.chars().nth(1).unwrap() < '1'
        || n.chars().nth(1).unwrap() > '8'
    {
        log_and_return_error!(FenParseError(format!(
            "Invalid notation ({}) when converting to index:",
            n
        )))
    }
    let file: char = n.chars().next().unwrap();
    let rank: char = n.chars().nth(1).unwrap();
    let rank_starts = [56, 48, 40, 32, 24, 16, 8, 0]; // 1st to 8th rank starting indexes
    let file_offset = match file {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => unreachable!(), // see error checking at start of function
    };
    let rank_digit = rank.to_digit(10).unwrap();
    Ok(file_offset + rank_starts[(rank_digit - 1) as usize])
}

#[inline]
fn index_to_notation(i: usize) -> String {
    let file = match i % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => ' ',
    };
    let rank_num = 8 - i / 8;
    let rank = char::from_digit(rank_num.try_into().unwrap(), 10).unwrap();
    format!("{}{}", file, rank)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_from_str_valid() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen = FEN::from_str(fen_str).unwrap();
        assert_eq!(fen.to_string(), fen_str);
    }

    #[test]
    fn test_fen_from_str_invalid_fields() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_from_str_invalid_piece_positions() {
        let fen_str = "rnbqkbnr/pppppppp/0/8/8/8/PPPPPPPP/RNBQKBNKK w KQkq - 0 1";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_from_str_invalid_side() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR xw KQkq - 0 1";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_from_str_invalid_castling_flags() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KdQkq - 0 1";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_from_str_invalid_en_passant() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq x2 0 1";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_from_str_invalid_halfmove_count() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - x 1";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_from_str_invalid_move_count() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 x";
        assert!(FEN::from_str(fen_str).is_err());
    }

    #[test]
    fn test_fen_to_string() {
        let fen = FEN::new();
        let fen_str = "8/8/8/8/8/8/8/8 w - - 0 1";
        assert_eq!(fen.to_string(), fen_str);
    }

    #[test]
    fn test_fen_from_board_state() {
        let board_state = BoardState::new_starting();
        let fen = FEN::from(&board_state);
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(fen.to_string(), fen_str);
    }

    #[test]
    fn test_fen_to_board_state() {
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen = FEN::from_str(fen_str).unwrap();
        let board_state: BoardState = fen.into();
        let fen_from_board = FEN::from(&board_state);
        assert_eq!(fen_from_board.to_string(), fen_str);
    }

    #[test]
    fn test_notation_to_index() {
        assert_eq!(notation_to_index("a1").unwrap(), 56);
        assert_eq!(notation_to_index("h8").unwrap(), 7);
        assert_eq!(notation_to_index("d4").unwrap(), 35);
        assert!(notation_to_index("i9").is_err());
        assert!(notation_to_index("a9").is_err());
        assert!(notation_to_index("z1").is_err());
    }

    #[test]
    fn test_index_to_notation() {
        assert_eq!(index_to_notation(56), "a1");
        assert_eq!(index_to_notation(7), "h8");
        assert_eq!(index_to_notation(35), "d4");
    }
}
