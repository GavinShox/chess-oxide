mod mailbox;
mod position;
mod engine;
mod board;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io;

use position::Position;
use position::Piece;
use std::time::{ Duration, Instant };

struct Board {
    position: Position,
}

// impl Board<'_> {
//     fn new() -> Self {

//     }
// }

fn get_all_legal_positions(mut pos: Position, depth: i32, nodes: &mut u64) -> () {
    if depth == 0 {
        return;
    }
    for mv in pos.get_legal_moves() {

            if depth == 1 {
                *nodes += 1;
            } else {
                let mut p = pos.new_position(mv);

                get_all_legal_positions(p, depth - 1, nodes);
            }
        
    }
    return;
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

        for mv in pos.get_legal_moves() {
            if mv.from == i && mv.to == j {
                pos = pos.new_position(&mv);
                //pos.print_board();
                illegal = false;
                break;
            }
        } 
        if illegal {
            println!("Move isn't legal!");
            input1.clear();
            input2.clear();
            continue;
        }
        let engine_mv = engine::choose_move(&pos);
        pos = pos.new_position(engine_mv);
        pos.print_board();
        input1.clear();
        input2.clear();
    }

    Ok(())
}

fn main() {
    let mut pos = Position::new_starting();
    //let mut pos = Position::new_position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0");
    pos.print_board();
    //println!("{:#?}", pos);
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
    let mut nodes: u64 = 0;
    //println!("{:x}", pos.pos_hash());
    get_all_legal_positions(pos, 5, &mut nodes);

    // let legal_moves = &pos.legal_moves;

    // for (i, mv) in legal_moves {
    //     for j in mv {
    //         let p = pos.new_move(*i, *j);
    //         positions.push(p);
    //     }
    // }

    // for p in &positions {
    //     p.print_board(&Vec::new());
    //     println!();
    // }
    //println!("{}", engine::negatedMax(&pos, 4));
    let duration = start.elapsed();
    println!("nodes: {}", nodes);

    println!("Time elapsed is: {:?}", duration);
    //move_pos(&pos);
}