use crate::board::*;
use crate::errors::FenParseError;
use crate::movegen::*;
use crate::position;
use crate::position::*;
use crate::util;

pub struct FEN {
    string: String,
}

impl FEN {
    pub fn from_string(fen: String) -> Self {
        Self { string: fen }
    }
    // partial implementation of the FEN format, last 2 fields are not used here
    // OK => returns completed Position struct and the parsed FEN fields
    // Err => returns the error message
    pub fn to_position(fen: &str) -> Result<(Position, Vec<&str>), FenParseError> {
        let mut pos = Pos64::default();
        let fen_vec: Vec<&str> = fen.split(' ').collect();

        // check if the FEN string has the correct number of fields, accept the last two as optional with default values given in BoardState
        if fen_vec.len() < 4 || fen_vec.len() > 6 {
            return Err(FenParseError(format!(
                "Invalid number of fields in FEN string: {}. Expected at least 4, max 6",
                fen_vec.len()
            )));
        }

        // first field of FEN defines the piece positions
        let mut rank_start_idx = 0;
        for rank in fen_vec[0].split('/') {
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
                    'k' => Square::Piece(Piece {
                        pcolour: PieceColour::Black,
                        ptype: PieceType::King,
                    }),
                    'K' => Square::Piece(Piece {
                        pcolour: PieceColour::White,
                        ptype: PieceType::King,
                    }),
                    x if x.is_ascii_digit() => {
                        for _ in 0..x.to_digit(10).unwrap() {
                            pos[i + rank_start_idx] = Square::Empty;
                            i += 1;
                        }
                        continue; // skip the below square assignment for pieces
                    }
                    other => {
                        return Err(FenParseError(format!(
                            "Invalid char in first field: {}",
                            other
                        )));
                    }
                };
                pos[i + rank_start_idx] = square;
                i += 1;
            }
            rank_start_idx += 8; // next rank
        }

        // second filed of FEN defines which side it is to move, either 'w' or 'b'
        let mut side = PieceColour::White;
        match fen_vec[1] {
            "w" => { /* already set as white */ }
            "b" => {
                side = PieceColour::Black;
            }
            other => {
                return Err(FenParseError(format!(
                    "Invalid second field: {}. Expected 'w' or 'b'",
                    other
                )));
            }
        }

        // initialise movegen flags for the next two FEN fields
        let mut movegen_flags = MovegenFlags::default();

        // third field of FEN defines castling flags
        for c in fen_vec[2].chars() {
            match c {
                'q' => {
                    movegen_flags.black_castle_long = true;
                }
                'Q' => {
                    movegen_flags.white_castle_long = true;
                }
                'k' => {
                    movegen_flags.black_castle_short = true;
                }
                'K' => {
                    movegen_flags.white_castle_short = true;
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

        // fourth field of FEN defines en passant flag, it gives notation of the square the pawn jumped over
        if fen_vec[3] != "-" {
            let ep_mv_idx = util::notation_to_index(fen_vec[3])?;

            // error if index is out of bounds. FEN defines the index behind the pawn that moved, so valid indexes are only 16->47 (excluded top and bottom two ranks)
            if !(16..=47).contains(&ep_mv_idx) {
                return Err(FenParseError(format!(
                    "Invalid en passant square: {}. Index is out of bounds",
                    fen_vec[3]
                )));
            }

            // in our struct however, we store the idx of the pawn to be captured
            let ep_flag = if side == PieceColour::White {
                ep_mv_idx + ABOVE_BELOW
            } else {
                ep_mv_idx - ABOVE_BELOW
            };
            movegen_flags.en_passant = Some(ep_flag);

            // set polyglot en passant flag if the ep_flag is beside a pawn of side to move colour
            if pos.polyglot_is_pawn_beside(ep_flag, side) {
                movegen_flags.polyglot_en_passant = Some(ep_flag);
            }
        }

        // initialise the new Position struct
        let new = Position::new_from_pub_parts(pos, side, movegen_flags);

        Ok((new, fen_vec))
    }

    pub fn from_position(pos: &Position) -> Self {
        let mut fen_str = String::new();

        let mut empty_count: i32 = 0;

        for (idx, sq) in pos.pos64.iter().enumerate() {
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

        match pos.side {
            PieceColour::White => fen_str.push('w'),
            PieceColour::Black => fen_str.push('b'),
        }
        fen_str.push(' ');

        if pos.movegen_flags.white_castle_short {
            fen_str.push('K');
        }
        if pos.movegen_flags.white_castle_long {
            fen_str.push('Q');
        }
        if pos.movegen_flags.black_castle_short {
            fen_str.push('k');
        }
        if pos.movegen_flags.black_castle_long {
            fen_str.push('q');
        }
        if !(pos.movegen_flags.white_castle_short
            || pos.movegen_flags.white_castle_long
            || pos.movegen_flags.black_castle_short
            || pos.movegen_flags.black_castle_long)
        {
            fen_str.push('-');
        }
        fen_str.push(' ');

        match pos.movegen_flags.en_passant {
            Some(idx) => {
                if pos.side == PieceColour::White {
                    fen_str.push_str(util::index_to_notation(idx - ABOVE_BELOW).as_str());
                } else {
                    fen_str.push_str(util::index_to_notation(idx + ABOVE_BELOW).as_str());
                }
            }
            None => {
                fen_str.push('-');
            }
        }
        fen_str.push(' ');

        // last two fields implemented in BoardState
        Self::from_string(fen_str)
    }
}
