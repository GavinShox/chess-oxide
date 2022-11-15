mod mailbox;
mod position;

use std::collections::HashMap;

use position::Position;
use position::Piece;
use position::MoveVec;



struct Board<'a> {
    position: Position,
    moves: HashMap<&'a Piece, MoveVec>
}





fn main() {
    let pos = Position::new();
    let newvec = pos.get_moves();
    pos.print_board(&newvec);
    println!("{:?}", newvec);

}