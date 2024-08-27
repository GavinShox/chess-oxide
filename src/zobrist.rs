use rand::Rng;

use crate::movegen::*;

pub type PositionHash = u64;

struct ZobristHashTable {
    pos_table: [[PositionHash; 12]; 64],
    en_passant_table: [PositionHash; 8], // 8 possible files that an en passant move can be made
    black_to_move: PositionHash,
    white_castle_long: PositionHash,
    black_castle_long: PositionHash,
    white_castle_short: PositionHash,
    black_castle_short: PositionHash,
    halfmove_count: [PositionHash; 100],
    occurrences: [PositionHash; 3]
}
impl ZobristHashTable {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut pos_table: [[PositionHash; 12]; 64] = [[0; 12]; 64];
        for i in 0..64 {
            for j in 0..12 {
                pos_table[i][j] = rng.gen();
            }
        }
        let mut en_passant_table: [PositionHash; 8] = [0; 8];
        for i in 0..8 {
            en_passant_table[i] = rng.gen();
        }
        let black_to_move = rng.gen();
        let white_castle_long = rng.gen();
        let black_castle_long = rng.gen();
        let white_castle_short = rng.gen();
        let black_castle_short = rng.gen();
        let mut halfmove_count: [PositionHash; 100] = [0; 100];
        for i in 0..100 {
            halfmove_count[i] = rng.gen();
        }
        let mut occurrences: [PositionHash; 3] = [0; 3];
        for i in 0..3 {
            occurrences[i] = rng.gen();
        }
        Self {
            pos_table,
            en_passant_table,
            black_to_move,
            white_castle_long,
            black_castle_long,
            white_castle_short,
            black_castle_short,
            halfmove_count,
            occurrences
        }
    }

    pub fn next(&self, current_hash: PositionHash, mv: &Move) -> PositionHash {
        todo!() //https://www.chessprogramming.org/Zobrist_Hashing
    }

    #[inline]
    pub fn get_piece_hash(&self, piece: &Piece, square_idx: usize) -> PositionHash {
        unsafe {
            *self.pos_table.get_unchecked(square_idx).get_unchecked(Self::get_piece_idx(piece))
        }
        //self.pos_table[square_idx][Self::get_piece_idx(piece)]
    }

    #[inline]
    fn get_piece_idx(piece: &Piece) -> usize {
        match piece.pcolour {
            PieceColour::White => match piece.ptype {
                PieceType::Pawn => 0,
                PieceType::Knight => 1,
                PieceType::Bishop => 2,
                PieceType::Rook => 3,
                PieceType::Queen => 4,
                PieceType::King => 5,
                PieceType::None => {
                    unreachable!("PieceType::None in get_piece_idx()")
                }
            },
            PieceColour::Black => match piece.ptype {
                PieceType::Pawn => 6,
                PieceType::Knight => 7,
                PieceType::Bishop => 8,
                PieceType::Rook => 9,
                PieceType::Queen => 10,
                PieceType::King => 11,
                PieceType::None => {
                    unreachable!("PieceType::None in get_piece_idx()")
                }
            },
            PieceColour::None => {
                unreachable!("PieceColour::None in get_piece_idx()")
            }
        }
    }
}
