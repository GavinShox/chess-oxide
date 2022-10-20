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



fn main() {
    println!("Hello, world!");
}



struct Piece {
    colour: PieceColour,
    ptype: PieceType
}

struct Board {

}