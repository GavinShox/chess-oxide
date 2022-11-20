use std::collections::HashMap;

use crate::mailbox;

type Pos64 = Vec<Square>;
type Offset = [i32; 8];

pub type MoveVec = Vec<usize>;

const MOVE_VEC_SIZE: usize = 27; // max number of squares a queen can possibly move to is 27

const PAWN_OFFSET: Offset = [0, 0, 0, 0, 0, 0, 0, 0];
const KNIGHT_OFFSET: Offset = [-21, -19, -12, -8, 8, 12, 19, 21];
const BISHOP_OFFSET: Offset = [-11, -9, 9, 11, 0, 0, 0, 0];
const ROOK_OFFSET: Offset = [-10, -1, 1, 10, 0, 0, 0, 0];
const QUEEN_KING_OFFSET: Offset = [-11, -10, -9, -1, 1, 9, 10, 11];

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

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pcolour: PieceColour,
    ptype: PieceType,
}

#[derive(Debug, Clone, Copy)]
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
    side_move_map: HashMap<usize, MoveVec>, // map of possible moves from "side"
    defend_map: HashMap<usize, MoveVec>, // map of squares opposite colour is defending
    pub legal_moves: HashMap<usize, MoveVec>, // legal moves hashmap only generated on function call
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
        let mut pos: Pos64 = Vec::with_capacity(64);
        let attack_map: HashMap<usize, MoveVec> = HashMap::new();
        let defend_map: HashMap<usize, MoveVec> = HashMap::new();
        let movegen_flags = MovegenFlags {
            white_castle_short: true,
            white_castle_long: true,
            black_castle_short: true,
            black_castle_long: true,
            en_passant: None,
        };

        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Rook }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Knight }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Bishop }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Queen }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::King }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Bishop }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Knight }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Rook }));
        for _ in 8..16 {
            pos.push(Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Pawn }));
        }
        for _ in 16..48 {
            pos.push(Square::Empty);
        }
        for _ in 48..56 {
            pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Pawn }));
        }
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Rook }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Knight }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Bishop }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Queen }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::King }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Bishop }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Knight }));
        pos.push(Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Rook }));

        let side = PieceColour::White;

        let mut new = Self {
            position: pos,
            side,
            movegen_flags,
            side_move_map: attack_map,
            defend_map,
            legal_moves: HashMap::new(),
        };
        new.gen_maps();
        new.gen_legal_moves();

        new
    }

    pub fn new_move(&self, i: usize, j: usize) -> Self {
        // assert move is legal maybe?
        let mut new_pos = self.clone();
        new_pos.en_passant_pawn_push(i, j);
        new_pos.position[j] = new_pos.position[i];
        new_pos.position[i] = Square::Empty;

        let ep = self.en_passant_capture_mv(i, j);
        if ep.is_some() {
            new_pos.position[ep.unwrap()] = Square::Empty;
        }

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos.gen_legal_moves();
        new_pos
    }

    // moves piece at i, to j, without changing side, to regen defend maps to determine if the move is legal
    // legality here meaning would the move leave your king in check. Actual piece movement is done in movegen
    fn is_move_legal(&self, i: usize, j: usize) -> bool {
        let mut new_pos = self.clone();
        new_pos.position[j] = new_pos.position[i];
        new_pos.position[i] = Square::Empty;
        // we only need to gen new defend map
        new_pos.gen_defend_map();

        !new_pos.is_in_check()
    }

    fn toggle_side(&mut self) -> () {
        self.side = if self.side == PieceColour::White {
            PieceColour::Black
        } else {
            PieceColour::White
        };
    }

    pub fn is_in_check(&self) -> bool {
        let mut in_check = false;
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if matches!(p.ptype, PieceType::King) && p.pcolour == self.side {
                        for (_, m) in &self.defend_map {
                            if m.contains(&i) {
                                in_check = true;
                                break;
                            }
                        }
                        break;
                    }
                }
                Square::Empty => {}
            }
        }
        in_check
    }

    // pub fn get_legal_moves(&mut self) -> &HashMap<usize, MoveVec> {
    //     if self.legal_moves.is_empty() {
    //         self.gen_legal_moves();
    //     }
    //     &self.legal_moves
    // }

    pub fn gen_legal_moves(&mut self) -> () {
        self.legal_moves = HashMap::new();

        for (i, move_vec) in &self.side_move_map {
            let mut legal_move_vec: MoveVec = Vec::with_capacity(MOVE_VEC_SIZE);

            for mv in move_vec {
                if self.is_move_legal(*i, *mv) {
                    legal_move_vec.push(*mv);
                }
            }
            self.legal_moves.insert(*i, legal_move_vec);
        }
    }

    // check if a move i -> mv is an en passant capture, if it is return usize of the pawn to be captured
    fn en_passant_capture_mv(&self, i: usize, mv: usize) -> Option<usize> {
        let s = &self.position[i];
        let mv_s = &self.position[mv];
        match s {
            Square::Piece(p) => {
                if p.ptype == PieceType::Pawn {
                    if
                        matches!(mv_s, Square::Empty) &&
                        ((i as i32) - (mv as i32)) % ABOVE_BELOW_MODULO != 0
                    {
                        println!("enpassant!");
                        let offset: i32 = if p.pcolour == PieceColour::White {
                            ABOVE_BELOW_MODULO
                        } else {
                            -ABOVE_BELOW_MODULO
                        };
                        return Some(((mv as i32) + offset) as usize);
                    }
                }
            }
            Square::Empty => {}
        }
        None
    }

    // sets enpassant movegen flag to Some(mv), if the move i -> mv is a double pawn push
    fn en_passant_pawn_push(&mut self, i: usize, mv: usize) -> () {
        let s = &self.position[i];
        match s {
            Square::Piece(p) => {
                if p.ptype == PieceType::Pawn {
                    if ((i as i32) - (mv as i32)) % 16 == 0 {
                        self.movegen_flags.en_passant = Some(mv);
                        return;
                    }
                }
            }
            Square::Empty => {}
        }
        self.movegen_flags.en_passant = None;
    }

    // generates moves for the piece at index i
    pub fn movegen(
        &self,
        piece: &Piece,
        i: usize,
        slide: bool,
        defending: bool,
        _include_pawn_pushes: bool
    ) -> MoveVec {
        let mut move_vec: MoveVec = Vec::with_capacity(MOVE_VEC_SIZE);

        // move gen for pawns
        if matches!(piece.ptype, PieceType::Pawn) {
            let white_push_offset = -10;
            let black_push_offset = 10;

            let push_offset = if piece.pcolour == PieceColour::White {
                white_push_offset
            } else {
                black_push_offset
            };

            if _include_pawn_pushes {
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
                let mut push_if_empty = |mv: i32| -> bool {
                    // check mv is valid
                    if mv >= 0 {
                        // push mv if the square is empty
                        let mv_square = &self.position[mv as usize];
                        if matches!(mv_square, Square::Empty) {
                            move_vec.push(mv as usize);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                let mut mv = mailbox::next_mailbox_number(i, push_offset);
                let empty = push_if_empty(mv);

                // if pawn is on starting square and the square above it is empty
                if starting && empty {
                    mv = mailbox::next_mailbox_number(i, push_offset * 2);
                    push_if_empty(mv);
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
                                move_vec.push(mv as usize);
                                continue;
                            } else {
                                continue;
                            }
                        }
                        Square::Empty => {}
                    }
                }
            }
            // en passant captures
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
                            match mv_above_square {
                                Square::Empty => {
                                    move_vec.push(mv_above as usize);
                                }
                                Square::Piece(_) => {}
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
                                move_vec.push(mv as usize);
                            }
                            break;
                        }
                        Square::Empty => {
                            move_vec.push(mv as usize);
                        }
                    }

                    if slide {
                        slide_idx += j;
                        mv = mailbox::next_mailbox_number(i, slide_idx);

                        continue;
                    } else {
                        break;
                    } // continue through rest of offsets
                }
            }
        }

        move_vec
    }

    fn gen_side_move_map(&mut self) -> () {
        self.side_move_map.drain();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour == self.side {
                        self.side_move_map.insert(
                            i,
                            self.movegen(p, i, Self::get_slide(p), false, true)
                        );
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
    }

    fn gen_defend_map(&mut self) -> () {
        self.defend_map.drain();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        self.defend_map.insert(
                            i,
                            self.movegen(p, i, Self::get_slide(p), true, false)
                        );
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
    }

    pub fn gen_maps(&mut self) -> () {
        // self.side_move_map.drain();
        // self.defend_map.drain();
        // for (i, s) in self.position.iter().enumerate() {
        //     match s {
        //         Square::Piece(p) => {
        //             if p.pcolour == self.side && side_move_map {
        //                 self.side_move_map.insert(
        //                     i,
        //                     self.movegen(p, i, Self::get_slide(p), false, true)
        //                 );
        //             } else if p.pcolour != self.side && defend_map {
        //                 self.defend_map.insert(
        //                     i,
        //                     self.movegen(p, i, Self::get_slide(p), true, false)
        //                 );
        //             }
        //         }
        //         Square::Empty => {
        //             continue;
        //         }
        //     }
        // }
        self.gen_side_move_map();
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