use crate::mailbox;

type Pos64 = Vec<Square>;
type Offset = [i32; 8];

pub type MoveVec = Vec<Move>;

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
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub capture: usize,
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
    side_move_map: MoveVec, // map of possible moves from "side"
    defend_map: Vec<usize>, // map of squares opposite colour is defending
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
        let mut pos: Pos64 = Vec::with_capacity(64);

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
            side_move_map: Vec::new(),
            defend_map: Vec::new(),
            legal_moves: Vec::new(),
        };
        new.gen_maps();
        new.gen_legal_moves();

        new
    }

    pub fn new_move(&self, mv: &Move) -> Self {
        // assert move is legal maybe?
        let mut new_pos = self.clone();
        new_pos.set_en_passant_flag(mv);
        new_pos.set_castle_flags(mv);
        new_pos.position[mv.to] = new_pos.position[mv.from];
        new_pos.position[mv.from] = Square::Empty;
        // en passant, 'to' square is different from the captured square
        if mv.to != mv.capture {
            new_pos.position[mv.capture] = Square::Empty;
        }

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos.gen_legal_moves();
        new_pos
    }

    // fn is_move_legal_clone(&self, i: usize, j: usize) -> bool {
    //     let mut new_pos = self.clone();
    //     new_pos.position[j] = new_pos.position[i];
    //     new_pos.position[i] = Square::Empty;
    //     let ep = new_pos.en_passant_capture_mv(i, j);
    //     if ep.is_some() {
    //         new_pos.position[ep.unwrap()] = Square::Empty;
    //     }
    //     // we only need to gen new defend map
    //     new_pos.gen_defend_map();

    //     !new_pos.is_in_check()

    // }

    // moves piece at i, to j, without changing side, to regen defend maps to determine if the move is legal
    // legality here meaning would the move leave your king in check. Actual piece movement is done in movegen
    fn is_move_legal(&mut self, mv: &Move) -> bool {
        let original_from = self.position[mv.from];
        let original_to = self.position[mv.to];
        let original_capture = self.position[mv.capture];

        self.position[mv.to] = self.position[mv.from];
        self.position[mv.from] = Square::Empty;
        if mv.to != mv.capture {
            self.position[mv.capture] = Square::Empty;
        }

        // we only need to gen new defend map
        self.gen_defend_map();

        let result = !self.is_in_check();

        self.position[mv.from] = original_from;
        self.position[mv.to] = original_to;
        self.position[mv.capture] = original_capture;

        result
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
                        if self.defend_map.contains(&i) {
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

        for mv in self.side_move_map.clone() {
            if self.is_move_legal(&mv) {
                self.legal_moves.push(mv);
            }
        }
    }

    // check if a move i -> mv is an en passant capture, if it is return usize of the pawn to be captured
    // fn en_passant_capture_mv(&self, i: usize, mv: usize) -> Option<usize> {
    //     let s = &self.position[i];
    //     let mv_s = &self.position[mv];
    //     match s {
    //         Square::Piece(p) => {
    //             if p.ptype == PieceType::Pawn {
    //                 if
    //                     matches!(mv_s, Square::Empty) &&
    //                     ((i as i32) - (mv as i32)) % ABOVE_BELOW_MODULO != 0
    //                 {
    //                     let offset: i32 = if p.pcolour == PieceColour::White {
    //                         ABOVE_BELOW_MODULO
    //                     } else {
    //                         -ABOVE_BELOW_MODULO
    //                     };
    //                     return Some(((mv as i32) + offset) as usize);
    //                 }
    //             }
    //         }
    //         Square::Empty => {}
    //     }
    //     None
    // }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    fn set_en_passant_flag(&mut self, mv: &Move) -> () {
        let s = &self.position[mv.from];
        match s {
            Square::Piece(p) => {
                if p.ptype == PieceType::Pawn {
                    if ((mv.from as i32) - (mv.to as i32)) % (ABOVE_BELOW_MODULO * 2) == 0 {
                        self.movegen_flags.en_passant = Some(mv.to);
                        return;
                    }
                }
            }
            Square::Empty => {}
        }
        self.movegen_flags.en_passant = None;
    }

    fn get_piece(&self, pos: usize) -> Option<&Piece> {
        match &self.position[pos] {
            Square::Piece(p) => Some(p),
            Square::Empty => None
        }
    }

    fn set_castle_flags(&mut self, mv: &Move) -> () {
        // starting positions for castling pieces
        const QUEEN_BLACK_ROOK: usize = 0;
        const KING_BLACK_ROOK: usize = 7;
        const QUEEN_WHITE_ROOK: usize = 56;
        const KING_WHITE_ROOK: usize = 63;
        const BLACK_KING: usize = 4;
        const WHITE_KING: usize = 60;


        match mv.from {
            WHITE_KING => {
                self.movegen_flags.white_castle_long = false;
                self.movegen_flags.white_castle_short = false;
            }
            BLACK_KING => {
                self.movegen_flags.black_castle_long = false;
                self.movegen_flags.black_castle_short = false;
            }
            QUEEN_BLACK_ROOK => {
                self.movegen_flags.black_castle_long = false;
            }
            QUEEN_WHITE_ROOK => {
                self.movegen_flags.white_castle_long = false;
            }
            KING_BLACK_ROOK => {
                self.movegen_flags.black_castle_short = false;
            }
            KING_WHITE_ROOK => {
                self.movegen_flags.white_castle_short = false;
            }
            _ => {}
        }
    }

    // generates moves for the piece at index i
    pub fn movegen(
        &self,
        piece: &Piece,
        i: usize,
        slide: bool,
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
                let mut push_if_empty = |mv: i32| -> bool {
                    // check mv is valid
                    if mv >= 0 {
                        // push mv if the square is empty
                        let mv_square = &self.position[mv as usize];
                        if matches!(mv_square, Square::Empty) {
                            move_vec.push(Move { from: i, to: mv as usize, capture: mv as usize });
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
                    // again, only pushing if the second square above is empty
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
                                move_vec.push(Move {
                                    from: i,
                                    to: mv as usize,
                                    capture: mv as usize,
                                });
                            } 
                        }
                        Square::Empty => {
                            if defending {
                                move_vec.push(Move {
                                    from: i,
                                    to: mv as usize,
                                    capture: mv as usize,
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
                                    capture: mv as usize,
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
                                    capture: mv as usize,
                                });
                            }
                            break;
                        }
                        Square::Empty => {
                            move_vec.push(Move { from: i, to: mv as usize, capture: mv as usize });
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

        // finally, movegen for castling
        if piece.ptype == PieceType::King {

        }

        move_vec
    }

    fn gen_side_move_map(&mut self) -> () {
        self.side_move_map.clear();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour == self.side {
                        self.side_move_map.extend(
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
        self.defend_map.clear();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        for mv in self.movegen(p, i, Self::get_slide(p), true, false) {
                            self.defend_map.push(mv.to);
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
