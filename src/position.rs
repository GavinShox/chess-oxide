use crate::errors::FenParseError;
use crate::mailbox;
use crate::movegen::*;
use crate::util;
use crate::zobrist;
use crate::zobrist::PositionHash;

const ABOVE_BELOW: usize = 8; // 8 indexes from i is the square directly above/below in the pos64 array

pub type Pos64 = [Square; 64];

#[derive(Debug, PartialEq, Clone)]
pub struct AttackMap(Vec<Move>);
impl AttackMap {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn new_no_alloc() -> Self {
        Self(Vec::with_capacity(0))
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DefendMap([bool; 64]);
impl DefendMap {
    fn new() -> Self {
        Self([false; 64])
    }

    fn clear(&mut self) {
        self.0 = [false; 64];
    }
}

impl MoveMap for DefendMap {
    fn add_move(&mut self, mv: &Move) {
        self.0[mv.to] = true;
    }
}

impl MoveMap for AttackMap {
    fn add_move(&mut self, mv: &Move) {
        self.0.push(*mv);
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub pos64: Pos64,
    pub side: PieceColour,
    pub movegen_flags: MovegenFlags,
    defend_map: DefendMap, // map of squares opposite colour is defending
    attack_map: AttackMap, // map of moves from attacking side
    wking_idx: usize,
    bking_idx: usize,
}
impl Position {
    // new board with starting Position
    pub fn new_starting() -> Self {
        let mut pos: Pos64 = [Square::Empty; 64];

        let movegen_flags = MovegenFlags {
            white_castle_short: true,
            white_castle_long: true,
            black_castle_short: true,
            black_castle_long: true,
            en_passant: None,
            polyglot_en_passant: None,
        };

        pos[0] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Rook,
        });
        pos[1] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Knight,
        });
        pos[2] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Bishop,
        });
        pos[3] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Queen,
        });
        pos[4] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::King,
        });
        pos[5] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Bishop,
        });
        pos[6] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Knight,
        });
        pos[7] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Rook,
        });
        for i in 8..16 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::Black,
                ptype: PieceType::Pawn,
            });
        }
        for i in 16..48 {
            pos[i] = Square::Empty;
        }
        for i in 48..56 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::White,
                ptype: PieceType::Pawn,
            });
        }
        pos[56] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Rook,
        });
        pos[57] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Knight,
        });
        pos[58] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Bishop,
        });
        pos[59] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Queen,
        });
        pos[60] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::King,
        });
        pos[61] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Bishop,
        });
        pos[62] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Knight,
        });
        pos[63] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Rook,
        });

        let side = PieceColour::White;

        let mut new = Self {
            pos64: pos,
            side,
            movegen_flags,
            defend_map: DefendMap::new(),
            attack_map: AttackMap::new(),
            wking_idx: 60,
            bking_idx: 4,
        };
        new.gen_maps();
        new
    }

    // Assumes a legal move, no legality checks are done, so no bounds checking is done here
    pub fn new_position(&self, mv: &Move) -> Self {
        let mut new_pos = self.clone();
        new_pos.set_en_passant_flag(mv);
        new_pos.set_castle_flags(mv);
        new_pos.set_king_position(mv);

        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                // en passant, 'to' square is different from the captured square
                new_pos.pos64[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                new_pos.pos64[castle_mv.rook_to] = new_pos.pos64[castle_mv.rook_from];
                new_pos.pos64[castle_mv.rook_from] = Square::Empty;
            }
            MoveType::Promotion(ptype, _) => match &mut new_pos.pos64[mv.from] {
                Square::Piece(p) => {
                    p.ptype = ptype;
                }
                Square::Empty => {
                    unreachable!();
                }
            },
            _ => {}
        }

        new_pos.pos64[mv.to] = new_pos.pos64[mv.from];
        new_pos.pos64[mv.from] = Square::Empty;

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos
    }

    #[inline(always)]
    pub fn pos_hash(&self) -> PositionHash {
        zobrist::pos_hash(self)
    }

    // TODO maybe consolidate all movegen flag updates into one place if possible?
    #[inline(always)]
    fn set_king_position(&mut self, mv: &Move) {
        if mv.piece.ptype == PieceType::King {
            if mv.piece.pcolour == PieceColour::White {
                self.wking_idx = mv.to;
            } else {
                self.bking_idx = mv.to;
            }
        }
    }
    // clone function for is_move_legal. Avoids expensive clone attack map
    #[inline(always)]
    fn test_clone(&self) -> Self {
        Self {
            pos64: self.pos64,
            side: self.side,
            movegen_flags: self.movegen_flags,
            defend_map: self.defend_map,
            // create new attack map with empty vec, because it's not needed for testing legality.
            attack_map: AttackMap::new_no_alloc(),
            wking_idx: self.wking_idx,
            bking_idx: self.bking_idx,
        }
    }

    pub fn is_move_legal(&self, mv: &Move) -> bool {
        // TODO defend map only used for castling, so maybe look at where defend map is necessary

        // tests before cloning position
        if mv.piece.ptype == PieceType::King {
            if self.is_defended(mv.to) {
                return false;
            }
            if let MoveType::Castle(castle_mv) = mv.move_type {
                // if any square the king moves from, through or to are defended, move isnt legal
                return !(self.is_defended(castle_mv.king_squares.0)
                    || self.is_defended(castle_mv.king_squares.1)
                    || self.is_defended(castle_mv.king_squares.2));
            }
        }

        let mut test_pos = self.test_clone();
        test_pos.set_king_position(mv);

        if let MoveType::EnPassant(ep_capture) = mv.move_type {
            test_pos.pos64[ep_capture] = Square::Empty;
        }

        test_pos.pos64[mv.to] = test_pos.pos64[mv.from];
        test_pos.pos64[mv.from] = Square::Empty;

        !movegen_in_check(&test_pos.pos64, test_pos.get_king_idx())
    }

    #[inline(always)]
    fn toggle_side(&mut self) {
        self.side = if self.side == PieceColour::White {
            PieceColour::Black
        } else {
            PieceColour::White
        };
    }

    #[inline(always)]
    fn is_defended(&self, i: usize) -> bool {
        self.defend_map.0[i]
    }

    #[inline(always)]
    fn get_king_idx(&self) -> usize {
        if self.side == PieceColour::White {
            self.wking_idx
        } else {
            self.bking_idx
        }
    }

    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        self.is_defended(self.get_king_idx())
    }

    pub fn get_pseudo_legal_moves(&self) -> &Vec<Move> {
        &self.attack_map.0
    }

    pub fn get_legal_moves(&self) -> Vec<&Move> {
        let mut legal_moves = Vec::with_capacity(self.attack_map.0.len());
        for mv in &self.attack_map.0 {
            if self.is_move_legal(mv) {
                legal_moves.push(mv);
            }
        }
        legal_moves
    }

    // is a pawn of colour 'pawn_colour' is either side of square at index i, used for setting polyglot en passant flag
    #[inline(always)]
    fn polyglot_is_pawn_beside(&self, i: usize, pawn_colour: PieceColour) -> bool {
        let piece = Piece {
            pcolour: pawn_colour,
            ptype: PieceType::Pawn,
        };
        let left = mailbox::next_mailbox_number(i, -1);
        // valid mailbox index
        if left >= 0 {
            if let Square::Piece(p) = &self.pos64[left as usize] {
                if p == &piece {
                    return true;
                }
            }
        }
        let right = mailbox::next_mailbox_number(i, 1);
        // valid mailbox index
        if right >= 0 {
            if let Square::Piece(p) = &self.pos64[right as usize] {
                if p == &piece {
                    return true;
                }
            }
        }
        false
    }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    #[inline(always)]
    fn set_en_passant_flag(&mut self, mv: &Move) {
        if mv.move_type == MoveType::DoublePawnPush {
            // if the pawn is beside an enemy pawn, set the polyglot en passant flag
            if self.polyglot_is_pawn_beside(mv.to, !self.side) {
                self.movegen_flags.polyglot_en_passant = Some(mv.to);
            } else {
                self.movegen_flags.polyglot_en_passant = None;
            }
            self.movegen_flags.en_passant = Some(mv.to);
        } else {
            self.movegen_flags.en_passant = None;
        }
    }

    #[inline(always)]
    fn set_castle_flags(&mut self, mv: &Move) {
        if mv.piece.ptype == PieceType::King {
            if mv.piece.pcolour == PieceColour::White {
                self.movegen_flags.white_castle_long = false;
                self.movegen_flags.white_castle_short = false;
            } else {
                self.movegen_flags.black_castle_long = false;
                self.movegen_flags.black_castle_short = false;
            }
        }
        match mv.from {
            LONG_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_long = false;
            }
            LONG_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_long = false;
            }
            SHORT_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_short = false;
            }
            SHORT_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_short = false;
            }
            _ => {}
        }
        // if a rook is captured
        match mv.to {
            LONG_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_long = false;
            }
            LONG_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_long = false;
            }
            SHORT_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_short = false;
            }
            SHORT_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_short = false;
            }
            _ => {}
        }
    }

    fn gen_maps(&mut self) {
        self.defend_map.clear();
        self.attack_map.clear();

        let pos64 = &self.pos64;
        let movegen_flags = &self.movegen_flags;
        let side = self.side;
        for (i, s) in pos64.iter().enumerate() {
            if let Square::Piece(p) = s {
                let is_defending = p.pcolour != side;
                let map: &mut dyn MoveMap = if is_defending {
                    &mut self.defend_map
                } else {
                    &mut self.attack_map
                };
                movegen(pos64, movegen_flags, p, i, is_defending, map);
            }
        }
    }

    // partial implementation of the FEN format, last 2 fields are not used here
    // OK => returns completed Position struct and the parsed FEN fields
    // Err => returns the error message
    pub fn from_fen_partial_impl(fen: &str) -> Result<(Self, Vec<&str>), FenParseError> {
        let mut pos: Pos64 = [Square::Empty; 64];
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
        let movegen_flags = MovegenFlags {
            white_castle_short: false,
            white_castle_long: false,
            black_castle_short: false,
            black_castle_long: false,
            en_passant: None,
            polyglot_en_passant: None,
        };

        // initialise the new Position struct
        let mut new = Self {
            pos64: pos,
            side,
            movegen_flags,
            defend_map: DefendMap::new(),
            attack_map: AttackMap::new(),
            wking_idx: 0, // value set in below for loop
            bking_idx: 0, // value set in below for loop
        };

        // third field of FEN defines castling flags
        for c in fen_vec[2].chars() {
            match c {
                'q' => {
                    new.movegen_flags.black_castle_long = true;
                }
                'Q' => {
                    new.movegen_flags.white_castle_long = true;
                }
                'k' => {
                    new.movegen_flags.black_castle_short = true;
                }
                'K' => {
                    new.movegen_flags.white_castle_short = true;
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
            new.movegen_flags.en_passant = Some(ep_flag);

            // set polyglot en passant flag if the ep_flag is beside a pawn of side to move colour
            if new.polyglot_is_pawn_beside(ep_flag, new.side) {
                new.movegen_flags.polyglot_en_passant = Some(ep_flag);
            }
        }

        for (i, s) in new.pos64.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.ptype == PieceType::King {
                        if p.pcolour == PieceColour::White {
                            new.wking_idx = i;
                        } else {
                            new.bking_idx = i;
                        }
                    }
                }
                Square::Empty => {}
            }
        }
        new.gen_maps();

        Ok((new, fen_vec))
    }

    pub fn to_fen_partial_impl(&self) -> String {
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
        fen_str
    }
}
