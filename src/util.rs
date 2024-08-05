use crate::movegen::{PieceColour, PieceType, Square};
use crate::BoardState;

// TODO Add error handling for invalid str and usizes
pub fn notation_to_index(n: &str) -> usize {
    let file: char = n.chars().next().unwrap();
    let rank: char = n.chars().nth(1).unwrap();
    let rank_starts = [56, 48, 40, 32, 24, 16, 8, 0]; // 1st to 8th rank starting indexes

    let file_offset = match file {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => 0,
    };
    file_offset + rank_starts[(rank.to_digit(10).unwrap() - 1) as usize]
}

pub fn index_to_notation(i: usize) -> String {
    let file = match i % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => ' ',
    };
    let rank_num = 8 - i / 8;
    let rank = char::from_digit(rank_num.try_into().unwrap(), 10).unwrap();
    format!("{}{}", file, rank)
}

#[allow(dead_code)]
pub fn print_board(bs: &BoardState) {
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

    for (num, j) in bs.get_pos64().iter().enumerate() {
        match j {
            Square::Piece(p) => match p.pcolour {
                PieceColour::White => match p.ptype {
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
                    PieceType::None => {
                        print!(" - ");
                    }
                },
                PieceColour::Black => match p.ptype {
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
                    PieceType::None => {
                        print!(" - ");
                    }
                },
                PieceColour::None => {}
            },
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
