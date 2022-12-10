use core::panic;
use std::{ mem::MaybeUninit, default, hash::Hasher };
use std::hash::Hash;

use crate::mailbox;

type Pos64 = [Square; 64];
type DefendMap = [bool; 64]; // array of boolean values determining if a square is defended by opposite side
type Offset = [i32; 8];

const MOVE_VEC_SIZE: usize = 27; // max number of squares a queen can possibly move to is 27

const PAWN_OFFSET: Offset = [0, 0, 0, 0, 0, 0, 0, 0];
const KNIGHT_OFFSET: Offset = [-21, -19, -12, -8, 8, 12, 19, 21];
const BISHOP_OFFSET: Offset = [-11, -9, 9, 11, 0, 0, 0, 0];
const ROOK_OFFSET: Offset = [-10, -1, 1, 10, 0, 0, 0, 0];
const QUEEN_KING_OFFSET: Offset = [-11, -10, -9, -1, 1, 9, 10, 11];

const PROMOTION_PIECE_TYPES: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

// starting indexes for castling logic
const LONG_BLACK_ROOK_START: usize = 0;
const SHORT_BLACK_ROOK_START: usize = 7;
const LONG_WHITE_ROOK_START: usize = 56;
const SHORT_WHITE_ROOK_START: usize = 63;
const BLACK_KING_START: usize = 4;
const WHITE_KING_START: usize = 60;

const ABOVE_BELOW: usize = 8; // 8 indexes from i is the square directly above/below in the pos64 array

#[macro_export]
macro_rules! extract_enum_value {
    ($value:expr, $pattern:pat => $extracted_value:expr) => {
    match $value {
      $pattern => $extracted_value,
      _ => panic!("Pattern doesn't match!"),
    }
    };
}
#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub move_type: MoveType,
}
#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub struct CastleMove {
    pub rook_from: usize,
    pub rook_to: usize,
    pub king_squares: (usize, usize, usize),
}

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub enum MoveType {
    EnPassant(usize),
    Promotion(PieceType),
    Castle(CastleMove),
    DoublePawnPush,
    Normal,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(PartialEq, Debug, Clone, Copy, Hash)]
pub enum PieceColour {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Piece {
    pub pcolour: PieceColour,
    pub ptype: PieceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Square {
    Piece(Piece),
    Empty,
}

#[derive(Debug, Clone, Hash)]
struct MovegenFlags {
    white_castle_short: bool,
    white_castle_long: bool,
    black_castle_short: bool,
    black_castle_long: bool,
    en_passant: Option<usize>,
}

#[derive(Debug, Clone, Hash)]
pub struct Position {
    pub position: Pos64,
    side: PieceColour,
    movegen_flags: MovegenFlags,
    defend_map: DefendMap, // map of squares opposite colour is defending
    attack_map: Vec<Move>,  // map of moves from attacking side
}

impl Position {
    pub fn pos_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn get_offset(piece: &Piece) -> Offset {
        match piece.ptype {
            PieceType::Pawn => PAWN_OFFSET, // not used
            PieceType::Knight => KNIGHT_OFFSET,
            PieceType::Bishop => BISHOP_OFFSET,
            PieceType::Rook => ROOK_OFFSET,
            PieceType::Queen => QUEEN_KING_OFFSET,
            PieceType::King => QUEEN_KING_OFFSET,
        }
    }

    fn get_slide(piece: &Piece) -> bool {
        match piece.ptype {
            PieceType::Pawn => false,
            PieceType::Knight => false,
            PieceType::Bishop => true,
            PieceType::Rook => true,
            PieceType::Queen => true,
            PieceType::King => false,
        }
    }

    // new board with starting Position
    pub fn new_starting() -> Self {
        let mut pos: Pos64 = [Square::Empty; 64];

        let movegen_flags = MovegenFlags {
            white_castle_short: true,
            white_castle_long: true,
            black_castle_short: true,
            black_castle_long: true,
            en_passant: None,
        };

        pos[0] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Rook });
        pos[1] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Knight });
        pos[2] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Bishop });
        pos[3] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Queen });
        pos[4] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::King });
        pos[5] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Bishop });
        pos[6] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Knight });
        pos[7] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Rook });
        for i in 8..16 {
            pos[i] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Pawn });
        }
        for i in 16..48 {
            pos[i] = Square::Empty;
        }
        for i in 48..56 {
            pos[i] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Pawn });
        }
        pos[56] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Rook });
        pos[57] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Knight });
        pos[58] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Bishop });
        pos[59] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Queen });
        pos[60] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::King });
        pos[61] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Bishop });
        pos[62] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Knight });
        pos[63] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Rook });

        let side = PieceColour::White;

        let mut new = Self {
            position: pos,
            side,
            movegen_flags,
            defend_map: [false; 64],
            attack_map: Vec::new(),
        };
        new.gen_maps();
        new.get_legal_moves();

        new
    }

    pub fn new_position_from_fen(fen: &str) -> Self {
        let mut pos: Pos64 = [Square::Empty; 64];
        let fen_vec: Vec<&str> = fen.split(' ').collect();
        assert_eq!(fen_vec.len(), 6);

        // first field of FEN defines the piece positions
        let mut rank_start_idx = 0;
        for rank in fen_vec[0].split('/') {
            let mut i = 0;
            for c in rank.chars() {
                let mut square = Square::Empty;
                match c {
                    'p' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Pawn,
                        });
                    }
                    'P' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Pawn,
                        });
                    }
                    'r' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Rook,
                        });
                    }
                    'R' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Rook,
                        });
                    }
                    'n' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Knight,
                        });
                    }
                    'N' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Knight,
                        });
                    }
                    'b' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Bishop,
                        });
                    }
                    'B' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Bishop,
                        });
                    }
                    'q' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Queen,
                        });
                    }
                    'Q' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Queen,
                        });
                    }
                    'k' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::King,
                        });
                    }
                    'K' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::King,
                        });
                    }
                    x if x.is_ascii_digit() => {
                        for _ in 0..x.to_digit(10).unwrap() {
                            pos[i + rank_start_idx] = Square::Empty;
                            i += 1;
                        }
                        continue; // skip the below square assignment for pieces
                    }
                    other => {
                        panic!("invalid char in first field: {}", other);
                    }
                }
                pos[i + rank_start_idx] = square;
                i += 1;
            }
            rank_start_idx += 8; // next rank
        }

        // second filed of FEN defines which side it is to move, either 'w' or 'b'
        let mut side = PieceColour::White;
        match fen_vec[1] {
            "w" => {/* already set as white */}
            "b" => {
                side = PieceColour::Black;
            }
            other => {
                panic!("invalid second field: {}", other);
            }
        }

        // initialise movegen flags for the next two FEN fields
        let mut movegen_flags = MovegenFlags {
            white_castle_short: false,
            white_castle_long: false,
            black_castle_short: false,
            black_castle_long: false,
            en_passant: None,
        };

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
                other => panic!("invalid char in third field: {}", other),
            }
        }

        // fourth field of FEN defines en passant flag, it gives notation of the square the pawn jumped over
        if !(fen_vec[3] == "-") {
            let ep_mv_idx = Self::notation_to_index(fen_vec[3]);
            // in our struct however, we store the idx of the pawn to be captured
            let ep_flag = if side == PieceColour::White { ep_mv_idx + ABOVE_BELOW } else { ep_mv_idx - ABOVE_BELOW };
            movegen_flags.en_passant = Some(ep_flag);
        }

        // Last two fields not used here, as the 50 move rule isnt calculated in Position struct

        let mut new = Self {
            position: pos,
            side: side,
            movegen_flags,
            defend_map: [false; 64],
            attack_map: Vec::new(),
        };
        new.gen_maps();
        new
    }

    // Assumes a legal move, no legality checks are done, so no bounds checking is done here
    pub fn new_position(&self, mv: &Move) -> Self {
        let mut new_pos = self.clone();
        new_pos.set_en_passant_flag(mv);
        new_pos.set_castle_flags(mv);

        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                // en passant, 'to' square is different from the captured square
                new_pos.position[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                new_pos.position[castle_mv.rook_to] = new_pos.position[castle_mv.rook_from];
                new_pos.position[castle_mv.rook_from] = Square::Empty;
            }
            MoveType::Promotion(ptype) => {
                match &mut new_pos.position[mv.from] {
                    Square::Piece(p) => {
                        p.ptype = ptype;
                    }
                    Square::Empty => {/* should never get here */}
                }
            }
            _ => {}
        }

        new_pos.position[mv.to] = new_pos.position[mv.from];
        new_pos.position[mv.from] = Square::Empty;

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos
    }

    // moves piece at i, to j, without changing side, to regen defend maps to determine if the move is legal
    // legality here meaning would the move leave your king in check. Actual piece movement is done in movegen
    fn is_move_legal(&self, mv: &Move) -> bool {
        let mut test_pos = self.clone();
        // doing special move types first, to check if mv is a castling move, and return there first
        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                test_pos.position[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                return if
                    test_pos.is_defended(castle_mv.king_squares.0) ||
                    test_pos.is_defended(castle_mv.king_squares.1) ||
                    test_pos.is_defended(castle_mv.king_squares.2)
                {
                    false // if any square the king moves from, through or to are defended, move isnt legal
                } else {
                    true // else castling is a legal move
                };
            }
            MoveType::Promotion(_) => {/* the piece the pawn promotes to doesn't effect legality */}
            _ => {}
        }

        // this has to be after the castleing section above, as king cant castle out of check
        test_pos.position[mv.to] = self.position[mv.from];
        test_pos.position[mv.from] = Square::Empty;

        // we only need to gen new defend map
        test_pos.gen_defend_map();

        let result = !test_pos.is_in_check();

        // self.position = original_position;
        // self.defend_map = original_defend_map;
        result
    }

    fn toggle_side(&mut self) -> () {
        self.side = if self.side == PieceColour::White {
            PieceColour::Black
        } else {
            PieceColour::White
        };
    }

    pub fn is_defended(&self, i: usize) -> bool {
        self.defend_map[i]
    }

    pub fn is_in_check(&self) -> bool {
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if matches!(p.ptype, PieceType::King) && p.pcolour == self.side {
                        if self.is_defended(i) {
                            return true;
                        }
                        break;
                    }
                }
                Square::Empty => {}
            }
        }
        return false;
    }

    pub fn get_legal_moves(&self) -> Vec<&Move> {
        let mut legal_moves = Vec::new();
        for mv in &self.attack_map {
            if self.is_move_legal(mv) {
                legal_moves.push(mv);
            }
        }
        legal_moves
    }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    fn set_en_passant_flag(&mut self, mv: &Move) -> () {
        if mv.move_type == MoveType::DoublePawnPush {
            self.movegen_flags.en_passant = Some(mv.to);
        } else {
            self.movegen_flags.en_passant = None;
        }
    }

    fn set_castle_flags(&mut self, mv: &Move) -> () {
        match mv.from {
            WHITE_KING_START => {
                self.movegen_flags.white_castle_long = false;
                self.movegen_flags.white_castle_short = false;
            }
            BLACK_KING_START => {
                self.movegen_flags.black_castle_long = false;
                self.movegen_flags.black_castle_short = false;
            }
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

    // generates moves for the piece at index i, only checks legality regarding where pieces could possibly move to
    // doesnt account for king checks

    pub fn movegen(
        &self,
        piece: &Piece,
        i: usize,
        defending: bool,
    ) -> Vec<Move> {
        let mut move_vec: Vec<Move> = Vec::new();

        // move gen for pawns
        if matches!(piece.ptype, PieceType::Pawn) {
            // mailbox offset for moving pawns straight up
            let white_push_offset = -10;
            let black_push_offset = 10;

            // set push offset to either white or black
            let push_offset = if piece.pcolour == PieceColour::White {
                white_push_offset
            } else {
                black_push_offset
            };

            // pawn push logic, only when defending is false, as pawn pushes are non-controlling moves
            if !defending {
                let starting: bool;

                // check if pawn is still on starting rank
                match piece.pcolour {
                    PieceColour::White => {
                        starting = if i < 56 && i > 47 { true } else { false };
                    }
                    PieceColour::Black => {
                        starting = if i < 16 && i > 7 { true } else { false };
                    }
                }

                // closure that pushes move to move_vec, if move is valid and the mv square is empty
                // returns true if it pushes successfully
                let mut push_if_empty = |mv: i32, mvtype: MoveType| -> bool {
                    // check mv is valid
                    if mv >= 0 {
                        // push mv if the square is empty
                        let mv_square = &self.position[mv as usize];
                        if matches!(mv_square, Square::Empty) {
                            move_vec.push(Move {
                                from: i,
                                to: mv as usize,
                                move_type: mvtype,
                            });
                            true
                        } else {
                            false
                        }
                    } else {
                        false  // also return false if mv is out of bounds
                    }
                };

                let mut mv = mailbox::next_mailbox_number(i, push_offset);
                let empty = push_if_empty(mv, MoveType::Normal);

                // if pawn is on starting square and the square above it is empty
                if starting && empty {
                    mv = mailbox::next_mailbox_number(i, push_offset * 2);
                    // again, only pushing if the second square above is empty
                    push_if_empty(mv, MoveType::DoublePawnPush);
                }
            }

            // attack/defend moves for pawns
            let white_attack_offset = [-9, -11];
            let black_attack_offset = [9, 11];

            let attack_offset = if piece.pcolour == PieceColour::White {
                white_attack_offset
            } else {
                black_attack_offset
            };

            for j in attack_offset {
                let mv = mailbox::next_mailbox_number(i, j);
                if mv > 0 {
                    let mv_square = &self.position[mv as usize];
                    match mv_square {
                        Square::Piece(mv_square_piece) => {
                            if piece.pcolour != mv_square_piece.pcolour || defending {
                                move_vec.push(Move {
                                    from: i,
                                    to: mv as usize,
                                    move_type: MoveType::Normal,
                                });
                            }
                        }
                        Square::Empty => {
                            if defending {
                                move_vec.push(Move {
                                    from: i,
                                    to: mv as usize,
                                    move_type: MoveType::Normal,
                                });
                            }
                        }
                    }
                }
            }
            // en passant captures, checking pawns left and right
            let attack_en_passant_offset = [-1, 1];
            if self.movegen_flags.en_passant.is_some() {
                let en_passant_mv = self.movegen_flags.en_passant.unwrap();
                for j in attack_en_passant_offset {
                    let mv = mailbox::next_mailbox_number(i, j);
                    if mv == (en_passant_mv as i32) {
                        // check if square above this is empty
                        let mv_above = mailbox::next_mailbox_number(mv as usize, push_offset);
                        if mv_above >= 0 {
                            let mv_above_square = &self.position[mv_above as usize];
                            if matches!(mv_above_square, Square::Empty) {
                                move_vec.push(Move {
                                    from: i,
                                    to: mv_above as usize,
                                    move_type: MoveType::EnPassant(mv as usize),
                                });
                            }
                        }
                    } else {
                        continue;
                    }
                }
            }
            // promotion movegen, we only need to do this if a pawn is one rank away from promotion, to save performance
            if
                (piece.pcolour == PieceColour::White && i < 16 && i > 7) ||
                (piece.pcolour == PieceColour::Black && i < 56 && i > 47)
            {
                let mut promotion_move_vec: Vec<Move> = Vec::new();
                move_vec.retain(|&mv| {
                    if
                        (piece.pcolour == PieceColour::White && mv.to <= 7) ||
                        (piece.pcolour == PieceColour::Black && mv.to >= 56)
                    {
                        for ptype in PROMOTION_PIECE_TYPES {
                            promotion_move_vec.push(Move {
                                from: mv.from,
                                to: mv.to,
                                move_type: MoveType::Promotion(ptype),
                            });
                        }
                        false // delete original move from movevec
                    } else {
                        true
                    }
                });
                move_vec.extend(promotion_move_vec);
            }
        } else {
            // move gen for other pieces
            for j in Self::get_offset(piece) {
                // end of offsets
                if j == 0 {
                    break;
                }

                let mut mv = mailbox::next_mailbox_number(i, j);
                let mut slide_idx = j;

                while mv >= 0 {
                    let mv_square = &self.position[mv as usize];
                    match mv_square {
                        Square::Piece(mv_square_piece) => {
                            if piece.pcolour != mv_square_piece.pcolour || defending {
                                move_vec.push(Move {
                                    from: i,
                                    to: mv as usize,
                                    move_type: MoveType::Normal,
                                });
                            }
                            break;
                        }
                        Square::Empty => {
                            move_vec.push(Move {
                                from: i,
                                to: mv as usize,
                                move_type: MoveType::Normal,
                            });
                        }
                    }
                    // is piece a sliding type
                    if Self::get_slide(piece) {
                        slide_idx += j;
                        mv = mailbox::next_mailbox_number(i, slide_idx);

                        continue;
                    } else {
                        break;
                    } // continue through rest of offsets
                }
            }
        }

        // finally, movegen for castling
        if piece.ptype == PieceType::King && !defending {
            if
                (piece.pcolour == PieceColour::White && i == WHITE_KING_START) ||
                (piece.pcolour == PieceColour::Black && i == BLACK_KING_START)
            {
                // no need to check mailbox, or check if an index is out of bounds
                // as we check that the king is on its starting square

                if
                    (piece.pcolour == PieceColour::White &&
                        self.movegen_flags.white_castle_short) ||
                    (piece.pcolour == PieceColour::Black && self.movegen_flags.black_castle_short)
                {
                    let short_mv_through_idx = i + 1;
                    let short_mv_to_idx = i + 2;
                    let short_rook_start_idx = i + 3;
                    let short_rook_end_idx = short_mv_through_idx;
                    if
                        matches!(&self.position[short_mv_through_idx], Square::Empty) &&
                        matches!(&self.position[short_mv_to_idx], Square::Empty)
                    {
                        move_vec.push(Move {
                            from: i,
                            to: short_mv_to_idx,
                            move_type: MoveType::Castle(CastleMove {
                                rook_from: short_rook_start_idx,
                                rook_to: short_rook_end_idx,
                                king_squares: (i, short_mv_through_idx, short_mv_to_idx),
                            }),
                        });
                    }
                }
                if
                    (piece.pcolour == PieceColour::White && self.movegen_flags.white_castle_long) ||
                    (piece.pcolour == PieceColour::Black && self.movegen_flags.black_castle_long)
                {
                    let long_mv_through_idx = i - 1;
                    let long_mv_to_idx = i - 2;
                    let long_mv_past_idx = i - 3; // sqaure not in kings path but still needs to be empty for rook
                    let long_rook_start_idx = i - 4;
                    let long_rook_end_idx = long_mv_through_idx;
                    if
                        matches!(&self.position[long_mv_through_idx], Square::Empty) &&
                        matches!(&self.position[long_mv_to_idx], Square::Empty) &&
                        matches!(&self.position[long_mv_past_idx], Square::Empty)
                    {
                        move_vec.push(Move {
                            from: i,
                            to: long_mv_to_idx,
                            move_type: MoveType::Castle(CastleMove {
                                rook_from: long_rook_start_idx,
                                rook_to: long_rook_end_idx,
                                king_squares: (i, long_mv_through_idx, long_mv_to_idx),
                            }),
                        });
                    }
                }
            }
        }

        move_vec
    }

    fn gen_defend_map(&mut self) -> () {
        self.defend_map = [false; 64];
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        for mv in self.movegen(p, i, true) {
                            self.defend_map[mv.to] = true;
                        }
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
    }

    pub fn gen_maps(&mut self) -> () {
        self.defend_map = [false; 64];
        self.attack_map = Vec::new();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        for mv in self.movegen(p, i, true) {
                            self.defend_map[mv.to] = true;
                        }
                    } else {
                        // for mv in self.movegen(p, i, false, true) {
                        //     self.attack_map.push(mv);
                        // }
                        self.attack_map.extend(self.movegen(p, i, false));
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
    }

    pub fn print_board(&self) {
        let pawn = " ♙ ";
        let knight = " ♘ ";
        let bishop = " ♗ ";
        let rook = " ♖ ";
        let queen = " ♕ ";
        let king = " ♔ ";

        let bking = " ♚ ";
        let bqueen = " ♛ ";
        let brook = " ♜ ";
        let bbishop = " ♝ ";
        let bknight = " ♞ ";
        let bpawn = " ♟︎ ";

        for (num, j) in self.position.iter().enumerate() {
            match j {
                Square::Piece(p) => {
                    match p.pcolour {
                        PieceColour::White => {
                            match p.ptype {
                                PieceType::Pawn => {
                                    print!("{}", pawn);
                                }
                                PieceType::Knight => {
                                    print!("{}", knight);
                                }
                                PieceType::Bishop => {
                                    print!("{}", bishop);
                                }
                                PieceType::Rook => {
                                    print!("{}", rook);
                                }
                                PieceType::Queen => {
                                    print!("{}", queen);
                                }
                                PieceType::King => {
                                    print!("{}", king);
                                }
                            }
                        }
                        PieceColour::Black => {
                            match p.ptype {
                                PieceType::Pawn => {
                                    print!("{}", bpawn);
                                }
                                PieceType::Knight => {
                                    print!("{}", bknight);
                                }
                                PieceType::Bishop => {
                                    print!("{}", bbishop);
                                }
                                PieceType::Rook => {
                                    print!("{}", brook);
                                }
                                PieceType::Queen => {
                                    print!("{}", bqueen);
                                }
                                PieceType::King => {
                                    print!("{}", bking);
                                }
                            }
                        }
                    }
                }
                Square::Empty => {
                    print!(" - ");
                }
            }

            // new rank
            if (num + 1) % 8 == 0 {
                println!();
            }
        }
    }

    pub fn notation_to_index(n: &str) -> usize {
        let file: char = n.chars().nth(0).unwrap();
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
            _ => 0,
        };
        file_offset + rank_starts[(rank.to_digit(10).unwrap() - 1) as usize]
    }

    pub fn index_to_notation(i: usize) -> String {
        let file = match i % 8 {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => ' '
        };
        let rank_num = (i / 8) + 1;
        let rank = char::from_digit(rank_num.try_into().unwrap(), 10).unwrap(); 
        format!("{}{}", file, rank)
    }

    pub fn move_as_notation(p: &str, mv: &str) -> (usize, usize) {
        let i: usize = Self::notation_to_index(p);
        let j: usize = Self::notation_to_index(mv);

        (i, j)
    }
}