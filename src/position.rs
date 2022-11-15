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

pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(PartialEq)]
pub enum PieceColour {
    White,
    Black
}

pub struct Piece {
    pcolour: PieceColour,
    ptype: PieceType
}

pub enum Square {
    Piece(Piece),
    Empty
}

pub struct Position {
    
    position: Pos64,
    side: PieceColour,
    xside: PieceColour,    
} 

impl Position {

    // new board with starting Position
    pub fn new() -> Self {
        let mut pos: Pos64 = Vec::with_capacity(64);

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

        Self { position: pos, side: PieceColour::White, xside: PieceColour::Black }
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

    // generates possible moves, TODO maybe consider checks. probably definitely....
    fn movegen(&self, piece: &Piece, i: usize, slide: bool) -> MoveVec {
        let mut move_vec: MoveVec = Vec::with_capacity(MOVE_VEC_SIZE);

        // move gen for pawns
        if matches!(piece.ptype, PieceType::Pawn) {
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

            let offset = if piece.pcolour == PieceColour::White {white_offset} else {black_offset};

            // closure that pushes move to move_vec, if move is valid and the mv square is empty 
            // returns true if it pushes successfully
            let mut push_if_empty = |mv: i32| -> bool{
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

            let mut mv = mailbox::next_mailbox_number(i, offset);
            let empty = push_if_empty(mv);

            // if pawn is on starting square and the square above it is empty
            if starting && empty {
                mv = mailbox::next_mailbox_number(i, offset * 2);
                push_if_empty(mv);
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
                            if piece.pcolour == mv_square_piece.pcolour {
                                break;
                            }
                            else {
                                move_vec.push(mv as usize);
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

    pub fn get_moves(&self) -> Vec<usize> {
        let mut move_vec: Vec<usize> = vec![];
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != PieceColour::White {
                        match p.ptype {
                            PieceType::Pawn => {
                                move_vec.extend(self.movegen(p, i, false))
                            }
                            PieceType::Knight => {
                                //move_vec.extend(self.movegen(p, i, false));
                            }
                            PieceType::Bishop => {
                                move_vec.extend(self.movegen(p, i, true));
                            }
                            PieceType::Rook => {
                                move_vec.extend(self.movegen(p, i, true));
                            }
                            PieceType::Queen => {

                            }
                            PieceType::King => {

                            }
                        }
                    }
                }
                _ => {continue}
            }
        }
        move_vec
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