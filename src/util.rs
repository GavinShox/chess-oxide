use crate::errors::FenParseError;
use crate::movegen::{PieceColour, PieceType, Square};
use crate::{log_and_return_error, BoardState};

#[inline]
pub fn notation_to_index(n: &str) -> Result<usize, FenParseError> {
    if n.len() != 2
        || n.chars().nth(0).unwrap() < 'a'
        || n.chars().nth(0).unwrap() > 'h'
        || n.chars().nth(1).unwrap() < '1'
        || n.chars().nth(1).unwrap() > '8'
    {
        log_and_return_error!(FenParseError(format!(
            "Invalid notation ({}) when converting to index:",
            n
        )))
    }
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
        _ => unreachable!(), // see error checking at start of function
    };
    let rank_digit = rank.to_digit(10).unwrap();
    Ok(file_offset + rank_starts[(rank_digit - 1) as usize])
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

pub fn rank_notation_to_indexes_unchecked(r: char) -> [usize; 8] {
    let rank_num = r.to_digit(10).unwrap() as usize;
    let rank_starts = [56, 48, 40, 32, 24, 16, 8, 0]; // 1st to 8th rank starting indexes
    let mut indexes = [0; 8];
    for (i, j) in indexes.iter_mut().enumerate() {
        *j = rank_starts[rank_num - 1] + i;
    }
    indexes
}

pub fn file_notation_to_indexes_unchecked(f: char) -> [usize; 8] {
    let file_offset = match f {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => unreachable!(),
    };
    let mut indexes = [0; 8];
    for (i, j) in indexes.iter_mut().enumerate() {
        *j = file_offset + i * 8;
    }
    indexes
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notation_to_index() {
        assert_eq!(notation_to_index("a1").unwrap(), 56);
        assert_eq!(notation_to_index("h8").unwrap(), 7);
        assert_eq!(notation_to_index("d4").unwrap(), 35);
        assert!(notation_to_index("i9").is_err());
        assert!(notation_to_index("a9").is_err());
        assert!(notation_to_index("z1").is_err());
    }

    #[test]
    fn test_index_to_notation() {
        assert_eq!(index_to_notation(56), "a1");
        assert_eq!(index_to_notation(7), "h8");
        assert_eq!(index_to_notation(35), "d4");
    }

    #[test]
    fn test_index_to_file_notation() {
        assert_eq!(index_to_file_notation(0), 'a');
        assert_eq!(index_to_file_notation(7), 'h');
        assert_eq!(index_to_file_notation(35), 'd');
    }

    #[test]
    fn test_index_to_rank_notation() {
        assert_eq!(index_to_rank_notation(0), '8');
        assert_eq!(index_to_rank_notation(7), '8');
        assert_eq!(index_to_rank_notation(35), '4');
    }

    #[test]
    fn test_rank_notation_to_indexes_unchecked() {
        assert_eq!(
            rank_notation_to_indexes_unchecked('1'),
            [56, 57, 58, 59, 60, 61, 62, 63]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('2'),
            [48, 49, 50, 51, 52, 53, 54, 55]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('3'),
            [40, 41, 42, 43, 44, 45, 46, 47]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('4'),
            [32, 33, 34, 35, 36, 37, 38, 39]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('5'),
            [24, 25, 26, 27, 28, 29, 30, 31]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('6'),
            [16, 17, 18, 19, 20, 21, 22, 23]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('7'),
            [8, 9, 10, 11, 12, 13, 14, 15]
        );
        assert_eq!(
            rank_notation_to_indexes_unchecked('8'),
            [0, 1, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn test_file_notation_to_indexes_unchecked() {
        assert_eq!(
            file_notation_to_indexes_unchecked('a'),
            [0, 8, 16, 24, 32, 40, 48, 56]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('b'),
            [1, 9, 17, 25, 33, 41, 49, 57]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('c'),
            [2, 10, 18, 26, 34, 42, 50, 58]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('d'),
            [3, 11, 19, 27, 35, 43, 51, 59]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('e'),
            [4, 12, 20, 28, 36, 44, 52, 60]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('f'),
            [5, 13, 21, 29, 37, 45, 53, 61]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('g'),
            [6, 14, 22, 30, 38, 46, 54, 62]
        );
        assert_eq!(
            file_notation_to_indexes_unchecked('h'),
            [7, 15, 23, 31, 39, 47, 55, 63]
        );
    }

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
}
