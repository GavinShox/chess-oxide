mod mailbox;
mod position;

use std::collections::HashMap;

use position::Position;
use position::Piece;
use position::MoveVec;
use std::time::{Duration, Instant};


struct Board {
    position: Position,
}

// impl Board<'_> {
//     fn new() -> Self {
        
//     }
// }

fn main() {
    let mut pos = Position::new_starting();
    // let idx: usize = 62;
    // let s = &pos.position[idx];
    // match s {
    //     position::Square::Piece(p) => {
    //         pos.print_board(pos.attack_map.get(&idx).unwrap());
    //         println!("{:p}", p)

    //     }
    //     _ => {}
    // }
    // todo move king to 31 and see if it allows a legal move
    let mut movevec: Vec<usize> = Vec::new();
    let start = Instant::now();
    let legal_moves: HashMap<usize, MoveVec> = pos.movegen_legal();
    let duration = start.elapsed();

    for (_, mv) in legal_moves {
        movevec.extend(mv);
    }
    pos.print_board(&movevec);
    println!("Time elapsed in legal_moves is: {:?}", duration);

}