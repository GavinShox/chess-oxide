use crate::mailbox;
use crate::position;

//pub const MOVE_VEC_SIZE: usize = 27; // max number of squares a queen can possibly move to is 27
type Offset = [i32; 8];

const PAWN_OFFSET: Offset = [0, 0, 0, 0, 0, 0, 0, 0];
const KNIGHT_OFFSET: Offset = [-21, -19, -12, -8, 8, 12, 19, 21];
const BISHOP_OFFSET: Offset = [-11, -9, 9, 11, 0, 0, 0, 0];
const ROOK_OFFSET: Offset = [-10, -1, 1, 10, 0, 0, 0, 0];
const QUEEN_KING_OFFSET: Offset = [-11, -10, -9, -1, 1, 9, 10, 11];

// indexes for *standard* starting position
const STD_LONG_WHITE_ROOK_START: usize = 56;
const STD_SHORT_WHITE_ROOK_START: usize = 63;
const STD_LONG_BLACK_ROOK_START: usize = 0;
const STD_SHORT_BLACK_ROOK_START: usize = 7;

// in both fischer random and standard starting positions
const BLACK_KING_SHORT_CASTLE_END: usize = 6;
const BLACK_KING_LONG_CASTLE_END: usize = 2;
const WHITE_KING_SHORT_CASTLE_END: usize = 62;
const WHITE_KING_LONG_CASTLE_END: usize = 58;
const BLACK_ROOK_SHORT_CASTLE_END: usize = 5;
const BLACK_ROOK_LONG_CASTLE_END: usize = 3;
const WHITE_ROOK_SHORT_CASTLE_END: usize = 61;
const WHITE_ROOK_LONG_CASTLE_END: usize = 59;

const PROMOTION_PIECE_TYPES: [PieceType; 4] = [
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
];

// from and to are out of bounds
pub const NULL_MOVE: Move = Move {
    piece: Piece {
        ptype: PieceType::King,
        pcolour: PieceColour::White,
    }, // dummy piece
    from: usize::MAX,
    to: usize::MAX,
    move_type: MoveType::None,
};

// from and to are out of bounds
pub const NULL_SHORT_MOVE: ShortMove = ShortMove {
    from: u8::MAX,
    to: u8::MAX,
    promotion_ptype: None,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PieceColour {
    White,
    Black,
}

impl core::ops::Not for PieceColour {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub pcolour: PieceColour,
    pub ptype: PieceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Square {
    Piece(Piece),
    Empty,
}
// todo maybe have a separate struct for starting flags instead of using movegen flags
#[derive(Debug, Clone, Copy)]
pub struct MovegenFlags {
    pub white_castle_short: bool,
    pub white_castle_long: bool,
    pub black_castle_short: bool,
    pub black_castle_long: bool,
    pub en_passant: Option<usize>,
    pub polyglot_en_passant: Option<usize>,
    pub long_white_rook_start: usize,
    pub short_white_rook_start: usize,
    pub long_black_rook_start: usize,
    pub short_black_rook_start: usize,
}

impl Default for MovegenFlags {
    fn default() -> Self {
        Self {
            white_castle_short: false,
            white_castle_long: false,
            black_castle_short: false,
            black_castle_long: false,
            en_passant: None,
            polyglot_en_passant: None,
            long_white_rook_start: STD_LONG_WHITE_ROOK_START,
            short_white_rook_start: STD_SHORT_WHITE_ROOK_START,
            long_black_rook_start: STD_LONG_BLACK_ROOK_START,
            short_black_rook_start: STD_SHORT_BLACK_ROOK_START,
        }
    }
}

impl MovegenFlags {
    // default flags for a standard starting position
    pub fn default_starting() -> Self {
        Self {
            white_castle_short: true,
            white_castle_long: true,
            black_castle_short: true,
            black_castle_long: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Move {
    pub piece: Piece,
    pub from: usize,
    pub to: usize,
    pub move_type: MoveType,
}

impl Move {
    pub const fn short_move(&self) -> ShortMove {
        ShortMove {
            from: self.from as u8,
            to: self.to as u8,
            promotion_ptype: match self.move_type {
                MoveType::Promotion(ptype, _) => Some(ptype),
                _ => None,
            },
        }
    }
}

// struct that stores enough information to identify any full sized move
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ShortMove {
    pub from: u8,
    pub to: u8,
    pub promotion_ptype: Option<PieceType>,
}

impl PartialEq<ShortMove> for Move {
    fn eq(&self, other: &ShortMove) -> bool {
        let result = self.from == other.from as usize && self.to == other.to as usize;
        // promotion checks
        if let Some(other_ptype) = other.promotion_ptype {
            if let MoveType::Promotion(self_ptype, _) = self.move_type {
                return result && self_ptype == other_ptype;
            }
        }
        result
    }
}

impl PartialEq<Move> for ShortMove {
    fn eq(&self, other: &Move) -> bool {
        let result = self.from as usize == other.from && self.to as usize == other.to;
        // promotion checks
        if let Some(self_ptype) = self.promotion_ptype {
            if let MoveType::Promotion(other_ptype, _) = other.move_type {
                return result && self_ptype == other_ptype;
            }
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CastleSide {
    Short,
    Long,
}
// TODO king squares include to and from indexes, which are already in the move struct. Maybe change this
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CastleMove {
    pub rook_from: usize,
    pub rook_to: usize,
    pub side: CastleSide,
}

impl CastleMove {
    pub const fn get_castle_side(&self) -> CastleSide {
        self.side
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveType {
    EnPassant(usize),
    Promotion(PieceType, Option<PieceType>),
    Castle(CastleMove),
    DoublePawnPush,
    PawnPush,
    Capture(PieceType),
    Normal,
    None, // used to represent null move, or moves that are only used in generating defend map, and are not actually possible to play
}

impl MoveType {
    #[inline]
    pub const fn is_capture(&self) -> bool {
        matches!(
            self,
            Self::Capture(_) | Self::EnPassant(_) | Self::Promotion(_, Some(_))
        )
    }
}

pub trait MoveMap {
    fn add_move(&mut self, _: &Move);
}

#[inline(always)]
fn pawn_promotion(
    mv_map: &mut dyn MoveMap,
    i: usize,
    piece: Piece,
    mv: i32,
    capture: Option<PieceType>,
) {
    for ptype in PROMOTION_PIECE_TYPES {
        mv_map.add_move(
            &(Move {
                piece,
                from: i,
                to: mv as usize,
                move_type: MoveType::Promotion(ptype, capture),
            }),
        );
    }
}

#[inline(always)]
fn is_square_empty(pos: &position::Pos64, i: usize) -> bool {
    // unsafe { return pos.get_unchecked(i) == &Square::Empty }
    pos[i] == Square::Empty
}

#[inline(always)]
const fn mb_get_pawn_push_offset(piece: Piece) -> i32 {
    match piece.pcolour {
        PieceColour::White => -10,
        PieceColour::Black => 10,
    }
}

#[inline(always)]
const fn mb_get_pawn_attack_offset(piece: Piece) -> [i32; 2] {
    const WHITE_ATTACK_OFFSET: [i32; 2] = [-9, -11];
    const BLACK_ATTACK_OFFSET: [i32; 2] = [9, 11];
    match piece.pcolour {
        PieceColour::White => WHITE_ATTACK_OFFSET,
        PieceColour::Black => BLACK_ATTACK_OFFSET,
    }
}

#[inline(always)]
const fn mb_get_offset(piece: Piece) -> Offset {
    match piece.ptype {
        PieceType::Pawn => PAWN_OFFSET, // not used
        PieceType::Knight => KNIGHT_OFFSET,
        PieceType::Bishop => BISHOP_OFFSET,
        PieceType::Rook => ROOK_OFFSET,
        PieceType::Queen => QUEEN_KING_OFFSET,
        PieceType::King => QUEEN_KING_OFFSET,
    }
}

#[inline(always)]
const fn get_slide(piece: Piece) -> bool {
    match piece.ptype {
        PieceType::Pawn | PieceType::Knight | PieceType::King => false,
        PieceType::Bishop | PieceType::Rook | PieceType::Queen => true,
    }
}

#[inline(always)]
const fn pawn_is_promotion_square(i: i32, piece: Piece) -> bool {
    match piece.pcolour {
        PieceColour::White => i <= 7,
        PieceColour::Black => i >= 56,
    }
}

#[inline(always)]
const fn pawn_is_starting_rank(i: usize, piece: Piece) -> bool {
    match piece.pcolour {
        PieceColour::White => i < 56 && i > 47,
        PieceColour::Black => i < 16 && i > 7,
    }
}

#[inline(always)]
fn is_castle_possible(
    pos: &position::Pos64,
    king_start: usize,
    king_end: usize,
    rook_start: usize,
    rook_end: usize,
) -> bool {
    let king_direction = if king_start < king_end { 1 } else { -1 };
    let rook_direction = if rook_start < rook_end { 1 } else { -1 };

    // Check if squares between king's start and end(inclusive) are empty
    for j in 1..=(king_end as i32 - king_start as i32).abs() {
        let idx = (king_start as i32 + j * king_direction) as usize;

        if let Square::Piece(p) = pos[idx] {
            if !(p.ptype == PieceType::Rook && idx == rook_start) {
                return false;
            }
        }
    }

    // Check if squares between rook's start and end are empty(inclusive)
    for j in 1..=(rook_end as i32 - rook_start as i32).abs() {
        let idx = (rook_start as i32 + j * rook_direction) as usize;
        if let Square::Piece(p) = pos[idx] {
            if !(p.ptype == PieceType::King && idx == king_start) {
                return false;
            }
        }
    }

    true
}

// generates moves for the piece at index i, only checks legality regarding where pieces could possibly move to
// doesnt account for discovered king checks after the move
pub(crate) fn movegen(
    pos: &position::Pos64,
    movegen_flags: &MovegenFlags,
    piece: Piece,
    i: usize,
    mv_map: &mut dyn MoveMap,
) {
    // Move gen for pawns
    if piece.ptype == PieceType::Pawn {
        // mailbox offset for moving pawns straight up
        let push_offset = mb_get_pawn_push_offset(piece);

        // closure that pushes move to mv_map, if move is valid and the mv square is empty
        // returns true if it pushes successfully
        let mut push_if_empty = |mv: i32, mvtype: MoveType| -> bool {
            // check mv is valid
            if mv >= 0 {
                // push mv if the square is empty
                if is_square_empty(pos, mv as usize) {
                    if pawn_is_promotion_square(mv, piece) {
                        pawn_promotion(mv_map, i, piece, mv, None);
                    } else {
                        mv_map.add_move(
                            &(Move {
                                piece,
                                from: i,
                                to: mv as usize,
                                move_type: mvtype,
                            }),
                        );
                    }
                    true
                } else {
                    false
                }
            } else {
                false // also return false if mv is out of bounds
            }
        };

        let mv_single_push = mailbox::next_mailbox_number(i, push_offset);
        let is_empty = push_if_empty(mv_single_push, MoveType::PawnPush);

        // check if pawn is still on starting rank
        let is_starting = pawn_is_starting_rank(i, piece);

        // if pawn is on starting square and the first square above it was empty
        // this is to prevent the pawn from jumping over a piece on it's first move
        if is_starting && is_empty {
            let mv_double_push = mailbox::next_mailbox_number(i, push_offset * 2);
            // again, only pushing if the second square above is empty
            push_if_empty(mv_double_push, MoveType::DoublePawnPush);
        }

        // Attacking moves for pawns
        let attack_offset = mb_get_pawn_attack_offset(piece);

        for j in attack_offset {
            let mv = mailbox::next_mailbox_number(i, j);
            if mv >= 0 {
                let mv_square = &pos[mv as usize];
                match mv_square {
                    Square::Piece(mv_square_piece) => {
                        if piece.pcolour != mv_square_piece.pcolour {
                            if pawn_is_promotion_square(mv, piece) {
                                pawn_promotion(mv_map, i, piece, mv, Some(mv_square_piece.ptype));
                            } else {
                                mv_map.add_move(
                                    &(Move {
                                        piece,
                                        from: i,
                                        to: mv as usize,
                                        move_type: MoveType::Capture(mv_square_piece.ptype),
                                    }),
                                );
                            }
                        }
                    }
                    Square::Empty => {}
                }
            }
        }
        // en passant captures, checking pawns left and right
        // also dont check for promotion, as a pawn cannot en passant to the back rank
        if movegen_flags.en_passant.is_some() {
            let attack_en_passant_offset = [-1, 1];
            let en_passant_mv = movegen_flags.en_passant.unwrap();
            for j in attack_en_passant_offset {
                let mv = mailbox::next_mailbox_number(i, j);
                if mv == (en_passant_mv as i32) {
                    // check if square above this is empty
                    let mv_above = mailbox::next_mailbox_number(mv as usize, push_offset);
                    if mv_above >= 0 && is_square_empty(pos, mv_above as usize) {
                        mv_map.add_move(
                            &(Move {
                                piece,
                                from: i,
                                to: mv_above as usize,
                                move_type: MoveType::EnPassant(mv as usize),
                            }),
                        );
                    }
                } else {
                    continue;
                }
            }
        }
    } else {
        // move gen for other pieces
        for j in mb_get_offset(piece) {
            // end of offsets
            if j == 0 {
                break;
            }

            let mut mv = mailbox::next_mailbox_number(i, j);
            let mut slide_idx = j;

            while mv >= 0 {
                let mv_square = &pos[mv as usize];
                match mv_square {
                    Square::Piece(mv_square_piece) => {
                        if piece.pcolour != mv_square_piece.pcolour {
                            mv_map.add_move(
                                &(Move {
                                    piece,
                                    from: i,
                                    to: mv as usize,
                                    move_type: MoveType::Capture(mv_square_piece.ptype),
                                }),
                            );
                        }
                        break; // break the slide after encountering a piece
                    }
                    Square::Empty => {
                        mv_map.add_move(
                            &(Move {
                                piece,
                                from: i,
                                to: mv as usize,
                                move_type: MoveType::Normal,
                            }),
                        );
                    }
                }
                // is piece a sliding type
                if get_slide(piece) {
                    slide_idx += j;
                    mv = mailbox::next_mailbox_number(i, slide_idx);

                    continue;
                } else {
                    break;
                } // continue through rest of offsets
            }
        }
    }

    // Castling movegen
    if piece.ptype == PieceType::King
        && ((piece.pcolour == PieceColour::White
            && (movegen_flags.white_castle_short || movegen_flags.white_castle_long))
            || (piece.pcolour == PieceColour::Black
                && (movegen_flags.black_castle_short || movegen_flags.black_castle_long)))
    {
        let (rook_short_start, rook_long_start, short_castle, long_castle) = match piece.pcolour {
            PieceColour::White => (
                movegen_flags.short_white_rook_start,
                movegen_flags.long_white_rook_start,
                movegen_flags.white_castle_short,
                movegen_flags.white_castle_long,
            ),
            PieceColour::Black => (
                movegen_flags.short_black_rook_start,
                movegen_flags.long_black_rook_start,
                movegen_flags.black_castle_short,
                movegen_flags.black_castle_long,
            ),
        };

        if short_castle {
            let king_short_end = if piece.pcolour == PieceColour::White {
                WHITE_KING_SHORT_CASTLE_END
            } else {
                BLACK_KING_SHORT_CASTLE_END
            };

            let rook_short_end = if piece.pcolour == PieceColour::White {
                WHITE_ROOK_SHORT_CASTLE_END
            } else {
                BLACK_ROOK_SHORT_CASTLE_END
            };

            if is_castle_possible(pos, i, king_short_end, rook_short_start, rook_short_end) {
                mv_map.add_move(
                    &(Move {
                        piece,
                        from: i,
                        to: king_short_end,
                        move_type: MoveType::Castle(CastleMove {
                            rook_from: rook_short_start,
                            rook_to: rook_short_end,
                            side: CastleSide::Short,
                        }),
                    }),
                );
            }
        }

        if long_castle {
            let king_long_end = if piece.pcolour == PieceColour::White {
                WHITE_KING_LONG_CASTLE_END
            } else {
                BLACK_KING_LONG_CASTLE_END
            };

            let rook_long_end = if piece.pcolour == PieceColour::White {
                WHITE_ROOK_LONG_CASTLE_END
            } else {
                BLACK_ROOK_LONG_CASTLE_END
            };

            if is_castle_possible(pos, i, king_long_end, rook_long_start, rook_long_end) {
                mv_map.add_move(
                    &(Move {
                        piece,
                        from: i,
                        to: king_long_end,
                        move_type: MoveType::Castle(CastleMove {
                            rook_from: rook_long_start,
                            rook_to: rook_long_end,
                            side: CastleSide::Long,
                        }),
                    }),
                );
            }
        }
    }
}

pub fn movegen_in_check(pos: &position::Pos64, king_idx: usize) -> bool {
    let king_colour = if let Square::Piece(p) = pos[king_idx] {
        p.pcolour
    } else {
        unreachable!("king_idx does not contain a king....")
    }; // just give the correct value please and we dont need to panic
    for (i, s) in pos.iter().enumerate() {
        match s {
            Square::Piece(piece) => {
                if piece.pcolour != king_colour {
                    // Move gen for pawns
                    if piece.ptype == PieceType::Pawn {
                        // Defending moves for pawns
                        let attack_offset = mb_get_pawn_attack_offset(*piece);

                        for j in attack_offset {
                            let mv = mailbox::next_mailbox_number(i, j);
                            if mv >= 0 {
                                if (mv as usize) == king_idx {
                                    return true;
                                } else {
                                    continue;
                                }
                            }
                        }
                    } else {
                        // move gen for other pieces
                        for j in mb_get_offset(*piece) {
                            // end of offsets
                            if j == 0 {
                                break;
                            }

                            let mut mv = mailbox::next_mailbox_number(i, j);
                            let mut slide_idx = j;

                            while mv >= 0 {
                                let mv_square = &pos[mv as usize];
                                match mv_square {
                                    Square::Piece(_) => {
                                        if (mv as usize) == king_idx {
                                            return true;
                                        } else {
                                            break; // break the slide after encountering a piece
                                        }
                                    }
                                    Square::Empty => {
                                        if (mv as usize) == king_idx {
                                            return true;
                                        }
                                    }
                                }
                                // is piece a sliding type
                                if get_slide(*piece) {
                                    slide_idx += j;
                                    mv = mailbox::next_mailbox_number(i, slide_idx);

                                    continue;
                                } else {
                                    break;
                                } // continue through rest of offsets
                            }
                        }
                    }
                } else {
                    continue;
                }
            }
            Square::Empty => {
                continue;
            }
        }
    }
    false
}
