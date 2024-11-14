use crate::engine::{get_checkmate_ply, is_eval_checkmate};
use crate::movegen::{PieceColour, PieceType, Square};
use crate::BoardState;

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
#[allow(clippy::cast_possible_truncation)]
#[inline(always)]
pub const fn high_bits(x: u64) -> u32 {
    // type casting to u32 truncates the high bits, so shift right by 32 bits and cast to u32
    (x >> 32) as u32
}

// returns the low bits of u64 returning a u32
#[allow(clippy::cast_possible_truncation)]
#[inline(always)]
pub const fn low_bits(x: u64) -> u32 {
    // type casting to u32 truncates the high bits
    x as u32
}

// return pretty-print string of a hash (full width hex hash)
#[inline(always)]
pub fn hash_to_string(hash: u64) -> String {
    format!("{:016x}", hash)
}

// Display engine eval in pawn units or handle checkmate evals as Mate in x ply/Checkmate
pub fn eval_to_string(eval: i32) -> String {
    if is_eval_checkmate(eval) {
        match get_checkmate_ply(eval) - 1 {
            0 => "Checkmate".to_string(),
            x => format!("Mate in {} ply", x),
        }
    } else {
        let eval = eval as f64 / 100.0; // convert centipawns to pawns
        format!("{:+.2}", eval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_str() {
        assert_eq!(bytes_to_str(1023), "1023.00 B");
        assert_eq!(bytes_to_str(1024), "1.00 KiB");
        assert_eq!(bytes_to_str(1048576), "1.00 MiB");
    }

    #[test]
    fn test_high_bits() {
        assert_eq!(high_bits(0x123456789ABCDEF0), 0x12345678);
        assert_eq!(high_bits(0xFFFFFFFFFFFFFFFF), 0xFFFFFFFF);
    }

    #[test]
    fn test_low_bits() {
        assert_eq!(low_bits(0x123456789ABCDEF0), 0x9ABCDEF0);
        assert_eq!(low_bits(0xFFFFFFFFFFFFFFFF), 0xFFFFFFFF);
    }

    #[test]
    fn test_hash_to_string() {
        assert_eq!(hash_to_string(0x123456789ABCDEF0), "123456789abcdef0");
        assert_eq!(hash_to_string(0xFFFFFFFFFFFFFF), "00ffffffffffffff");
    }
}
