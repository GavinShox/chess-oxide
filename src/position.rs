use std::mem::MaybeUninit;

use crate::mailbox;

type Pos64 = [Square; 64];
type DefendMap = [bool; 64]; // array of boolean values determining if a square is defended by opposite side
type Offset = [i32; 8];

pub type MoveVec = Vec<Move>;

const MOVE_VEC_SIZE: usize = 27; // max number of squares a queen can possibly move to is 27

const PAWN_OFFSET: Offset = [0, 0, 0, 0, 0, 0, 0, 0];
const KNIGHT_OFFSET: Offset = [-21, -19, -12, -8, 8, 12, 19, 21];
const BISHOP_OFFSET: Offset = [-11, -9, 9, 11, 0, 0, 0, 0];
const ROOK_OFFSET: Offset = [-10, -1, 1, 10, 0, 0, 0, 0];
const QUEEN_KING_OFFSET: Offset = [-11, -10, -9, -1, 1, 9, 10, 11];

// starting indexes for castling logic
const LONG_BLACK_ROOK_START: usize = 0;
const SHORT_BLACK_ROOK_START: usize = 7;
const LONG_WHITE_ROOK_START: usize = 56;
const SHORT_WHITE_ROOK_START: usize = 63;
const BLACK_KING_START: usize = 4;
const WHITE_KING_START: usize = 60;

// offset of rook from original position after castling
const AFTER_LONG_CASTLE_ROOK_OFFSET: i32 = 3;
const AFTER_SHORT_CASTLE_ROOK_OFFSET: i32 = -2;

const ABOVE_BELOW_MODULO: i32 = 8; // (initial square index - move square index) % const == 0 if either index is above/below the other.

#[macro_export]
macro_rules! extract_enum_value {
    ($value:expr, $pattern:pat => $extracted_value:expr) => {
    match $value {
      $pattern => $extracted_value,
      _ => panic!("Pattern doesn't match!"),
    }
    };
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub move_type: MoveType,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CastleMove {
    pub rook_from: usize,
    pub rook_to: usize,
    pub king_squares: (usize, usize, usize),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MoveType {
    EnPassant(usize),
    Castle(CastleMove),
    DoublePawnPush,
    Normal,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PieceColour {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pcolour: PieceColour,
    ptype: PieceType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Square {
    Piece(Piece),
    Empty,
}

#[derive(Debug, Clone)]
struct MovegenFlags {
    white_castle_short: bool,
    white_castle_long: bool,
    black_castle_short: bool,
    black_castle_long: bool,
    en_passant: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub position: Pos64,
    side: PieceColour,
    movegen_flags: MovegenFlags,
    defend_map: DefendMap, // map of squares opposite colour is defending
    pub legal_moves: MoveVec, // legal moves in given position
}
// TODO impl hash for position
impl Position {
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
            legal_moves: Vec::new(),
        };
        new.gen_maps();
        new.gen_legal_moves();

        new
    }

    // Assumes a legal move, no legality checks are done, so no bounds checking is done here
    pub fn new_position(&self, mv: &Move) -> Self {
        // assert move is legal maybe?
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
            _ => {}
        }

        new_pos.position[mv.to] = new_pos.position[mv.from];
        new_pos.position[mv.from] = Square::Empty;

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos.gen_legal_moves();
        new_pos
    }

    // moves piece at i, to j, without changing side, to regen defend maps to determine if the move is legal
    // legality here meaning would the move leave your king in check. Actual piece movement is done in movegen
    fn is_move_legal(&mut self, mv: &Move) -> bool {
        let original_position: Pos64 = self.position.clone();

        // doing special move types first, to check if mv is a castling move, and return there first
        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                self.position[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                return if
                    self.is_defended(castle_mv.king_squares.0) ||
                    self.is_defended(castle_mv.king_squares.1) ||
                    self.is_defended(castle_mv.king_squares.2)
                {
                    false  // if any square the king moves from, through or to are defended, move isnt legal
                } else {
                    true  // else castling is a legal move
                }
            }
            _ => {}
        }

        // this has to be after the castleing section above, as king cant castle out of check
        self.position[mv.to] = self.position[mv.from];
        self.position[mv.from] = Square::Empty;

        // we only need to gen new defend map
        self.gen_defend_map();

        let result = !self.is_in_check();

        self.position = original_position;

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
        let mut in_check = false;
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if matches!(p.ptype, PieceType::King) && p.pcolour == self.side {
                        if self.is_defended(i) {
                            in_check = true;
                            break;
                        }
                        break;
                    }
                }
                Square::Empty => {}
            }
        }
        in_check
    }

    fn gen_legal_moves(&mut self) -> () {
        self.legal_moves = Vec::new();
        let mut side_moves = Vec::new();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour == self.side {
                        side_moves.extend(self.movegen(p, i, false, true));
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
        for mv in &side_moves {
            if self.is_move_legal(mv) {
                self.legal_moves.push(*mv);
            }
        }
    }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    fn set_en_passant_flag(&mut self, mv: &Move) -> () {
        if mv.move_type == MoveType::DoublePawnPush {
            self.movegen_flags.en_passant = Some(mv.to)
        } else {
            self.movegen_flags.en_passant = None;
        }
    }

    fn get_piece(&self, pos: usize) -> Option<&Piece> {
        match &self.position[pos] {
            Square::Piece(p) => Some(p),
            Square::Empty => None,
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
        include_pawn_pushes: bool
    ) -> MoveVec {
        let mut move_vec: MoveVec = Vec::with_capacity(MOVE_VEC_SIZE);

        // move gen for pawns
        if matches!(piece.ptype, PieceType::Pawn) {
            // mailbox offset for moving pawns straight up/down
            let white_push_offset = -10;
            let black_push_offset = 10;

            // set push offset to either white or black
            let push_offset = if piece.pcolour == PieceColour::White {
                white_push_offset
            } else {
                black_push_offset
            };

            if include_pawn_pushes {
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
                        false
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
                        for mv in self.movegen(p, i, true, false) {
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
        self.gen_defend_map();
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

    pub fn move_as_notation(p: &str, mv: &str) -> (usize, usize) {
        let rank_starts = [56, 48, 40, 32, 24, 16, 8, 0]; // 8th to 1st rank starting indexes

        let get_file_offset = |c: &char| -> usize {
            match c {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                'f' => 5,
                'g' => 6,
                'h' => 7,
                _ => 0,
            }
        };

        let pfile: char = p.chars().nth(0).unwrap();
        let prank: char = p.chars().nth(1).unwrap();

        let mvfile: char = mv.chars().nth(0).unwrap();
        let mvrank: char = mv.chars().nth(1).unwrap();

        let i: usize =
            get_file_offset(&pfile) + rank_starts[(prank.to_digit(10).unwrap() - 1) as usize];
        let j: usize =
            get_file_offset(&mvfile) + rank_starts[(mvrank.to_digit(10).unwrap() - 1) as usize];

        (i, j)
    }
}