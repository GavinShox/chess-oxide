use std::mem::MaybeUninit;
use std::mem;

pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

pub enum PieceColour {
    White,
    Black
}

struct Piece {
    colour: PieceColour,
    ptype: PieceType
}

enum Square {
    Piece(Piece),
    Empty
}

struct Board {
    position: [Square; 64]
} 

impl Board {
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

    const MAILBOX64: [usize; 64] = [
        21, 22, 23, 24, 25, 26, 27, 28,
        31, 32, 33, 34, 35, 36, 37, 38,
        41, 42, 43, 44, 45, 46, 47, 48,
        51, 52, 53, 54, 55, 56, 57, 58,
        61, 62, 63, 64, 65, 66, 67, 68,
        71, 72, 73, 74, 75, 76, 77, 78,
        81, 82, 83, 84, 85, 86, 87, 88,
        91, 92, 93, 94, 95, 96, 97, 98
    ];

    // new board with starting position
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

        pos[0] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Rook} );
        pos[1] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Knight} );
        pos[2] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Bishop} );
        pos[3] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Queen} );
        pos[4] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::King} );
        pos[5] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Bishop} );
        pos[6] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Knight} );
        pos[7] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Rook} );
        for i in 8..16 {
            pos[i] = Square::Piece( Piece{colour: PieceColour::Black, ptype: PieceType::Pawn} );
        }
        for i in 48..56 {
            pos[i] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Pawn} );
        }
        pos[56] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Rook} );
        pos[57] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Knight} );
        pos[58] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Bishop} );
        pos[59] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Queen} );
        pos[60] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::King} );
        pos[61] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Bishop} );
        pos[62] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Knight} );
        pos[63] = Square::Piece( Piece{colour: PieceColour::White, ptype: PieceType::Rook} );

        Self { position: pos }
    }

    fn piece_mailbox_index(piece_pos_index: usize) -> usize {
        Self::MAILBOX64[piece_pos_index]
    }
}

fn main() {
    println!("Hello, world!");
    let board: &Board = &Board::new();
    let pos1 = &board.position[1];

    match pos1{
        Square::Empty => println!("Empty"),
        Square::Piece(p) => println!("There is a piece")
    }
    println!("Done");
}