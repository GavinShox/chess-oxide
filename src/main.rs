mod mailbox;
mod position;

use std::collections::HashMap;
use std::io;

use position::Position;
use position::Piece;
use position::MoveVec;
use std::time::{ Duration, Instant };

struct Board {
    position: Position,
}

// impl Board<'_> {
//     fn new() -> Self {

//     }
// }

fn get_all_legal_positions(pos: &Position, depth: i32) -> Vec<Position> {
    let mut positions = Vec::new();
    if depth == 0 {
        return positions;
    }
    for mv in &pos.legal_moves {
            let p = pos.new_position(mv);
            positions.extend(get_all_legal_positions(&p, depth - 1));
            if depth == 1 {
                positions.push(p);
            }
        
    }
    positions
}

fn move_pos(p: &Position) -> io::Result<()> {
    let mut pos = p.clone();
    let stdin = io::stdin();
    let mut input1 = String::new();
    let mut input2 = String::new();

    loop {
        println!("Move from:");
        stdin.read_line(&mut input1)?;
        println!("Move to:");
        stdin.read_line(&mut input2)?;
        let mut illegal = true;
        let (i, j) = Position::move_as_notation(&input1, &input2);

        for mv in pos.legal_moves.clone() {
            if mv.from == i && mv.to == j {
                pos = pos.new_position(&mv);
                pos.print_board();
                illegal = false;
            }
        } 
        if illegal {
            println!("Move isn't legal!");
        }
        input1.clear();
        input2.clear();
    }

    Ok(())
}

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

    let start = Instant::now();

    let positions: Vec<Position> = get_all_legal_positions(&pos, 3);

    // let legal_moves = &pos.legal_moves;

    // for (i, mv) in legal_moves {
    //     for j in mv {
    //         let p = pos.new_move(*i, *j);
    //         positions.push(p);
    //     }
    // }
    let duration = start.elapsed();

    // for p in &positions {
    //     p.print_board(&Vec::new());
    //     println!();
    // }
    println!("{}", positions.len());
    println!("Time elapsed is: {:?}", duration);
    move_pos(&pos);
}