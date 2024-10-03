use crate::movegen::{PieceColour, PieceType, Square};
use crate::BoardState;

// TODO Add error handling for invalid str and usizes
#[inline]
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

#[inline]
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

#[inline]
pub fn index_to_file_notation(i: usize) -> char {
    match i % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => ' ',
    }
}

#[inline]
pub fn index_to_rank_notation(i: usize) -> char {
    let rank_num = 8 - i / 8;
    char::from_digit(rank_num.try_into().unwrap(), 10).unwrap()
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
                },
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

#[inline]
pub fn bytes_to_str(size: usize) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = size as f64;
    let mut i = 0;
    while size >= 1024.0 {
        size /= 1024.0;
        i += 1;
    }
    format!("{:.2} {}", size, units[i])
}

// returns the high bits of u64 returning a u32
#[inline(always)]
pub fn high_bits(x: u64) -> u32 {
    // type casting to u32 truncates the high bits, so shift right by 32 bits and cast to u32
    (x >> 32) as u32
}

// returns the low bits of u64 returning a u32
#[inline(always)]
pub fn low_bits(x: u64) -> u32 {
    // type casting to u32 truncates the high bits
    x as u32
}
