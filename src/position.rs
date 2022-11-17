use std::{collections::HashMap, hash::Hash};

use crate::mailbox;

type Pos64 = Vec<Square>;
type Offset = [i32; 8];

pub type MoveVec = Vec<usize>;

const MOVE_VEC_SIZE: usize = 27; // max number of squares a queen can possibly move to is 27
const ATTACK_DEFEND_MAP_SIZE: usize = 16; // max number of pieces each side

const PAWN_OFFSET: Offset = [0, 0, 0, 0, 0, 0, 0, 0];
const KNIGHT_OFFSET: Offset = [-21, -19, -12, -8, 8, 12, 19, 21];
const BISHOP_OFFSET: Offset = [-11, -9, 9, 11, 0, 0, 0, 0];
const ROOK_OFFSET: Offset = [-10, -1, 1, 10, 0, 0, 0, 0];
const QUEEN_KING_OFFSET: Offset = [-11, -10, -9, -1, 1, 9, 10, 11];

#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PieceColour {
    White,
    Black
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pcolour: PieceColour,
    ptype: PieceType
}

#[derive(Debug, Clone, Copy)]
pub enum Square {
    Piece(Piece),
    Empty
}

#[derive(Debug, Clone)]
pub struct Position {
    pub position: Pos64,
    side: PieceColour,
    attack_map: HashMap<usize, MoveVec>, // map of squares colour "side" is attacking
    defend_map: HashMap<usize, MoveVec>, // map of squares opposite colour is defending (same as an attack map, but includes their own pieces)
    legal_moves: Option<HashMap<usize, MoveVec>>
} 
impl Position {

    // new board with starting Position
    pub fn new_starting() -> Self {
        let mut pos: Pos64 = Vec::with_capacity(64);
        let mut attack_map: HashMap<usize, MoveVec> = HashMap::new();
        let mut defend_map: HashMap<usize, MoveVec> = HashMap::new();

        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Rook} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Knight} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Bishop} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Queen} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::King} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Bishop} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Knight} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Rook} ) );
        for _ in 8..16 {
            pos.push( Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Pawn} ) );
        }
        for _ in 16..48 {
            pos.push( Square::Empty );
        }
        for _ in 48..56 {
            pos.push(Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Pawn} ) );
        }
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Rook} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Knight} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Bishop} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Queen} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::King} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Bishop} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Knight} ) );
        pos.push( Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Rook} ) );

        let side = PieceColour::White;

        let mut new = Self { position: pos, side, attack_map, defend_map };
        new.gen_maps();

        new
    }

    // maybe add new field containing legal moves? TODO
    pub fn new_move_legal(&self) -> Self {todo!() }

    // moves piece at i, to j, without checking legality
    fn _new_move_force(&self, i: usize, j: usize, toggle_side: bool) -> Self {
        let mut new_self = self.clone();

        new_self.position[j] = new_self.position[i];
        new_self.position[i] = Square::Empty;
        if toggle_side { new_self.toggle_side(); }
        
        new_self.gen_maps();

        new_self
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
            PieceType::King => false
        }
    }

    fn toggle_side(&mut self) -> () {
        self.side = if self.side == PieceColour::White {PieceColour::Black} else {PieceColour::White};
    }

    fn is_in_check(&self) -> bool {
        let mut in_check = false;
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if matches!(p.ptype, PieceType::King) && p.pcolour == self.side {
                        for m in &self.defend_map {
                            if m.1.contains(&i) {
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
    
    
    fn movegen_legal(&mut self) -> () {
        let mut legal_moves: HashMap<usize, MoveVec> = HashMap::new();

        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour == self.side {
                        let piece_move_vec = self.movegen(p, i, Self::get_slide(p), false, true);
                        let mut legal_move_vec: Vec<usize> = Vec::with_capacity(MOVE_VEC_SIZE);
                        for mv in piece_move_vec {
                            let new_pos = self._new_move_force(i, mv, false);
                            if !new_pos.is_in_check() {
                                legal_move_vec.push(mv);
                            }
                        }
                        legal_moves.insert(i, legal_move_vec);
                    } else { continue; }
                }
                Square::Empty => { continue; }
            
            }
        }

        self.legal_moves = Some(legal_moves);
    }

    // generates moves given a piece and index
    pub fn movegen(&self, piece: &Piece, i: usize, slide: bool, defending: bool, _include_pawn_pushes: bool) -> MoveVec {
        let mut move_vec: MoveVec = Vec::with_capacity(MOVE_VEC_SIZE);

        // move gen for pawns
        if matches!(piece.ptype, PieceType::Pawn) {
            
            if _include_pawn_pushes {
                let white_offset = -10;
                let black_offset = 10;
                let mut starting = false;

                // check if pawn is still on starting rank
                match piece.pcolour {
                    PieceColour::White => {
                        starting = if i < 56 && i > 47 {true} else {false};
                    }
                    PieceColour::Black => {
                        starting = if i < 16 && i > 7 {true} else {false};
                    }
                }

                // closure that pushes move to move_vec, if move is valid and the mv square is empty 
                // returns true if it pushes successfully
                let mut push_if_empty = |mv: i32| -> bool {
                    // check mv is valid
                    if mv > 0 {
                        // push mv if the square is empty
                        let mv_square = &self.position[mv as usize];
                        if matches!(mv_square, Square::Empty) {
                            move_vec.push(mv as usize);
                            true

                        } else { false }

                    } else { false }
                };

                let offset = if piece.pcolour == PieceColour::White {white_offset} else {black_offset};
                let mut mv = mailbox::next_mailbox_number(i, offset);
                let empty = push_if_empty(mv);

                // if pawn is on starting square and the square above it is empty
                if starting && empty {
                    mv = mailbox::next_mailbox_number(i, offset * 2);
                    push_if_empty(mv);
                }

            }

            // attack/defend moves for pawns
            let white_attack_offset = [-9, -11];
            let black_attack_offset = [9, 11];

            let attack_offset = if piece.pcolour == PieceColour::White {white_attack_offset} else {black_attack_offset};
            
            for j in attack_offset {
                let mv = mailbox::next_mailbox_number(i, j);
                if mv > 0 {
                    let mv_square = &self.position[mv as usize];
                    match mv_square {
                        Square::Piece(mv_square_piece) => {
                            if piece.pcolour != mv_square_piece.pcolour || defending {
                                move_vec.push(mv as usize);
                                continue;
                            }
                            else {
                                continue;
                            }
                        }
                        Square::Empty => {}
                    }
                }
            }
        }
        // move gen for other pieces
        else {
            for j in Self::get_offset(piece) {

                // end of offsets
                if j == 0 { break; }
    
                let mut mv = mailbox::next_mailbox_number(i, j);
                let mut slide_idx = j;
    
                while mv >= 0 {
                    
                    let mv_square = &self.position[mv as usize];
                    match mv_square {
                        Square::Piece(mv_square_piece) => {
                            if piece.pcolour != mv_square_piece.pcolour || defending {
                                move_vec.push(mv as usize);
                                break;
                            }
                            else {
                                break;
                            }
                        }
                        Square::Empty => { move_vec.push(mv as usize); }
                    }
                    
                    if slide {
    
                        slide_idx += j;
                        mv = mailbox::next_mailbox_number(i, slide_idx);
                        
                        continue;
    
                    } else { break; }  // continue through rest of offsets
                }
            }
        }

        move_vec
    }

    pub fn gen_maps(&mut self) -> () {
        self.attack_map.drain();
        self.defend_map.drain();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour == self.side {
                        self.attack_map.insert(i, self.movegen(p, i, Self::get_slide(p), false, false));
                    } else {
                        self.defend_map.insert(i, self.movegen(p, i, Self::get_slide(p), true, false));                    }
                }
                Square::Empty => { continue; }
            }
        }
    }

    pub fn print_board(&self, move_vec: &MoveVec) {
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
                                PieceType::Pawn => { print!("{}", pawn); },
                                PieceType::Knight => { print!("{}", knight); },
                                PieceType::Bishop => { print!("{}", bishop); },
                                PieceType::Rook => { print!("{}", rook); },
                                PieceType::Queen => { print!("{}", queen); },
                                PieceType::King => { print!("{}", king); },
                            }
                        }
                        PieceColour::Black => {
                            match p.ptype {
                                PieceType::Pawn => { print!("{}", bpawn); },
                                PieceType::Knight => { print!("{}", bknight); },
                                PieceType::Bishop => { print!("{}", bbishop); },
                                PieceType::Rook => { print!("{}", brook); },
                                PieceType::Queen => { print!("{}", bqueen); },
                                PieceType::King => { print!("{}", bking); },
                            }
                        }
                    }
                    
                },
                Square::Empty => { 
                    if move_vec.contains(&num) {
                        print!(" + ")
                    } else {
                        print!(" - ")
                    }
                    
                },
            }
            
            // new rank
            if ((num+1) % 8) == 0 {
                println!()
            } 
        }
    }

}