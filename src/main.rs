use std::mem::MaybeUninit;
use std::mem;

type Pos64 = [Square; 64];
type Offset = [i32; 8];

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
struct Piece {
    pcolour: PieceColour,
    ptype: PieceType,
}
enum Square {
    Piece(Piece),
    Empty
}

struct Position {
    side: PieceColour,
    position: Pos64,
} 

impl Position {

    const MAILBOX: [i32; 120] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1,  0,  1,  2,  3,  4,  5,  6,  7, -1,
        -1,  8,  9, 10, 11, 12, 13, 14, 15, -1,
        -1, 16, 17, 18, 19, 20, 21, 22, 23, -1,
        -1, 24, 25, 26, 27, 28, 29, 30, 31, -1,
        -1, 32, 33, 34, 35, 36, 37, 38, 39, -1,
        -1, 40, 41, 42, 43, 44, 45, 46, 47, -1,
        -1, 48, 49, 50, 51, 52, 53, 54, 55, -1,
        -1, 56, 57, 58, 59, 60, 61, 62, 63, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
    ];

    // what index a given board position is in the 120 mailbox
    const MAILBOX64: [i32; 64] = [
        21, 22, 23, 24, 25, 26, 27, 28,
        31, 32, 33, 34, 35, 36, 37, 38,
        41, 42, 43, 44, 45, 46, 47, 48,
        51, 52, 53, 54, 55, 56, 57, 58,
        61, 62, 63, 64, 65, 66, 67, 68,
        71, 72, 73, 74, 75, 76, 77, 78,
        81, 82, 83, 84, 85, 86, 87, 88,
        91, 92, 93, 94, 95, 96, 97, 98
    ];

    // new board with starting Position
    pub fn new() -> Self {
        let mut pos = {
            let mut pos: [MaybeUninit<Square>; 64] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for i in 0..64 {
                pos[i] = MaybeUninit::new(Square::Empty);
            }
            unsafe {mem::transmute::<_, [Square; 64]>(pos)}
        };

        pos[0] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Rook} );
        pos[1] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Knight} );
        pos[2] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Bishop} );
        pos[3] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Queen} );
        pos[4] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::King} );
        pos[5] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Bishop} );
        pos[6] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Knight} );
        pos[7] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Rook} );
        for i in 8..16 {
            pos[i] = Square::Piece( Piece{pcolour: PieceColour::Black, ptype: PieceType::Pawn} );
        }
        for i in 48..56 {
            pos[i] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Pawn} );
        }
        pos[56] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Rook} );
        pos[57] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Knight} );
        pos[58] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Bishop} );
        pos[59] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Queen} );
        pos[60] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::King} );
        pos[61] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Bishop} );
        pos[62] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Knight} );
        pos[63] = Square::Piece( Piece{pcolour: PieceColour::White, ptype: PieceType::Rook} );

        Self { position: pos, side: PieceColour::White }
    }

    fn get_offset(piece: &Piece) -> Offset {
        match piece.ptype {
            PieceType::Pawn => PAWN_OFFSET,
            PieceType::Knight => KNIGHT_OFFSET,
            PieceType::Bishop => BISHOP_OFFSET,
            PieceType::Rook => ROOK_OFFSET,
            PieceType::Queen => QUEEN_KING_OFFSET,
            PieceType::King => QUEEN_KING_OFFSET,
        }
    }

    fn movegen(&self, piece: &Piece, i: usize, slide: bool) -> Vec<i32> {
        let mut move_vec = Vec::with_capacity(64);

        for j in Self::get_offset(piece) {

            // end of offsets
            if j == 0 { break; }

            let mut slide_idx = j;
            let mut mv = Self::MAILBOX[(Self::MAILBOX64[i] + j) as usize];

            while mv >= 0 {
                
                let mv_square = &self.position[mv as usize];
                match mv_square {
                    Square::Piece(mv_square_piece) => {
                        if piece.pcolour == mv_square_piece.pcolour {
                            break;
                        }
                        else {
                            move_vec.push(mv);
                            break;
                        }
                    }
                    Square::Empty => { move_vec.push(mv); }
                }
                
                if slide {

                    let next_idx = (Self::MAILBOX64[i] + slide_idx) as usize;     
                    mv = Self::MAILBOX[next_idx];
                    slide_idx += j;
                    
                    continue;

                } else { break; }
            }
        }
        move_vec
    }

    fn get_moves(&self) -> Vec<i32> {
        let mut move_vec: Vec<i32> = vec![];
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != PieceColour::Black {
                        match p.ptype {
                            PieceType::Pawn => {
                            }
                            PieceType::Knight => {
                                move_vec.extend(self.movegen(p, i, false));
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


}

fn print_board(board: &Position) {
    let pawn = " ♙ ";
    let knight = " ♘ ";
    let bishop = " ♗ ";
    let rook = " ♖ ";
    let queen = " ♕ ";
    let king = " ♔ ";


    for (num, j) in board.position.iter().enumerate() {
        match j {
            Square::Piece(p) => {
                match p.ptype {
                    PieceType::Pawn => { print!("{}", pawn); },
                    PieceType::Knight => { print!("{}", knight); },
                    PieceType::Bishop => { print!("{}", bishop); },
                    PieceType::Rook => { print!("{}", rook); },
                    PieceType::Queen => { print!("{}", queen); },
                    PieceType::King => { print!("{}", king); },
                }
            },
            Square::Empty => { print!(" - "); },
        }
        
        if ((num+1) % 8) == 0 {
            println!()
        } 
    }
}

fn main() {
    let board = &mut Position::new();
    let newvec = board.get_moves();
    print_board(board);
    println!("{:?}", newvec);
}