use rand::Rng;

use static_init::dynamic;

use crate::{movegen::*, Position};

// static table, to ensure all positions that are equal have the same hashes for the duration of the program
#[dynamic]
static ZOBRIST_HASH_TABLE: ZobristHashTable = ZobristHashTable::new();
// using 64 bit hashes
pub type PositionHash = u64;

// zobrist hash of full Position, used to initialise the board
pub fn pos_hash(pos: &Position) -> PositionHash {
    ZOBRIST_HASH_TABLE.full_position_hash(pos)
}

// increment the zobrist hash of a Position, can be used when moves are made instead of calling pos_hash on the whole position every move
pub fn pos_next_hash(current_hash: PositionHash, mv: &Move) -> PositionHash {
    todo!()
}

// add BoardState information into a zobrist Position hash
pub fn board_state_hash(current_hash: PositionHash, occurrences: u8, halfmove_count: u32) -> PositionHash {
    todo!()
}

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
    fn new() -> Self {
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

    pub fn next_hash(&self, current_hash: PositionHash, mv: &Move) -> PositionHash {
        todo!() //https://www.chessprogramming.org/Zobrist_Hashing
    }

    pub fn full_position_hash(&self, pos: &Position) -> PositionHash {
        let mut hash = 0;
        for (i, s) in pos.pos64.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    hash ^= self.get_piece_hash(p, i);
                }
                Square::Empty => {
                    continue;
                }
            }
        }
        if pos.movegen_flags.white_castle_long {
            hash ^= self.white_castle_long;
        }
        if pos.movegen_flags.black_castle_long {
            hash ^= self.black_castle_long;
        }
        if pos.movegen_flags.white_castle_short {
            hash ^= self.white_castle_short;
        }
        if pos.movegen_flags.black_castle_short {
            hash ^= self.black_castle_short;
        }
        if pos.movegen_flags.en_passant.is_some() {
            hash ^= self.en_passant_table[pos.movegen_flags.en_passant.unwrap() % 8];
        }
        if pos.side == PieceColour::Black {
            hash ^= self.black_to_move;
        }

        hash
    }

    fn board_state_hash(&self, current_hash: PositionHash, occurrences: u8, halfmove_count: u32) -> PositionHash {
        current_hash ^ self.get_occurrences_hash(occurrences) ^ self.halfmove_count[halfmove_count as usize]
    }

    #[inline(always)]
    fn get_halfmove_count_hash(&self, halfmove_count: u32) -> PositionHash {
        self.halfmove_count[halfmove_count as usize]
    }

    #[inline(always)]
    fn get_occurrences_hash(&self, occurrences: u8) -> PositionHash {
        match occurrences {
            1 => self.occurrences[0],
            2 => self.occurrences[1],
            3 => self.occurrences[2],
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn get_piece_hash(&self, piece: &Piece, square_idx: usize) -> PositionHash {
        // unsafe {
        //     *self.pos_table.get_unchecked(square_idx).get_unchecked(Self::get_piece_idx(piece))
        // }
        self.pos_table[square_idx][Self::get_piece_idx(piece)]
    }

    #[inline(always)]
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
