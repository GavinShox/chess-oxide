use rand::Rng;

use static_init::dynamic;

use crate::magic;
use crate::movegen::*;
use crate::position::Position;

// static table, to ensure all positions that are equal have the same hashes for the duration of the program
#[dynamic]
static ZOBRIST_HASH_TABLE: ZobristHashTable = ZobristHashTable::with_polyglot_magic();

// using 64 bit hashes
pub type PositionHash = u64;

// zobrist hash of full Position, used to initialise a position hash
pub fn pos_hash(pos: &Position) -> PositionHash {
    ZOBRIST_HASH_TABLE.polyglot_full_position_hash(pos)
}

// increment the zobrist hash of a Position, can be used when moves are made instead of calling pos_hash on the whole position every move
pub fn pos_next_hash(
    last_movegen_flags: &MovegenFlags,
    new_movegen_flags: &MovegenFlags,
    last_hash: PositionHash,
    mv: &Move,
) -> PositionHash {
    ZOBRIST_HASH_TABLE.polyglot_next_hash(last_movegen_flags, new_movegen_flags, last_hash, mv)
}

// add BoardState information into a zobrist Position hash
pub fn board_state_hash(
    current_hash: PositionHash,
    occurrences: u8,
    halfmove_count: u32,
) -> PositionHash {
    ZOBRIST_HASH_TABLE.board_state_hash(current_hash, occurrences, halfmove_count)
}

struct ZobristHashTable {
    pos_table: [[PositionHash; 12]; 64],
    en_passant_table: [PositionHash; 8], // 8 possible files that an en passant move can be made
    white_to_move: PositionHash,
    white_castle_long: PositionHash,
    black_castle_long: PositionHash,
    white_castle_short: PositionHash,
    black_castle_short: PositionHash,
    halfmove_count: [PositionHash; 100],
    occurrences: [PositionHash; 3],
}
impl ZobristHashTable {
    #[allow(dead_code)]
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut pos_table: [[PositionHash; 12]; 64] = [[0; 12]; 64];
        for i in &mut pos_table {
            for j in i {
                *j = rng.gen();
            }
        }
        let mut en_passant_table: [PositionHash; 8] = [0; 8];
        for i in &mut en_passant_table {
            *i = rng.gen();
        }
        let white_to_move = rng.gen();
        let white_castle_long = rng.gen();
        let black_castle_long = rng.gen();
        let white_castle_short = rng.gen();
        let black_castle_short = rng.gen();
        let mut halfmove_count: [PositionHash; 100] = [0; 100];
        for i in &mut halfmove_count {
            *i = rng.gen();
        }
        let mut occurrences: [PositionHash; 3] = [0; 3];
        for i in &mut occurrences {
            *i = rng.gen();
        }
        Self {
            pos_table,
            en_passant_table,
            white_to_move,
            white_castle_long,
            black_castle_long,
            white_castle_short,
            black_castle_short,
            halfmove_count,
            occurrences,
        }
    }

    pub const fn with_polyglot_magic() -> Self {
        Self {
            pos_table: magic::POLYGLOT_MAGIC_POS_TABLE,
            en_passant_table: magic::POLYGLOT_MAGIC_EN_PASSANT_TABLE,
            white_to_move: magic::POLYGLOT_MAGIC_WHITE_TO_MOVE,
            white_castle_long: magic::POLYGLOT_MAGIC_WHITE_CASTLE_LONG,
            black_castle_long: magic::POLYGLOT_MAGIC_BLACK_CASTLE_LONG,
            white_castle_short: magic::POLYGLOT_MAGIC_WHITE_CASTLE_SHORT,
            black_castle_short: magic::POLYGLOT_MAGIC_BLACK_CASTLE_SHORT,
            halfmove_count: magic::MAGIC_HALFMOVE_COUNT_TABLE,
            occurrences: magic::MAGIC_OCCURRENCES_TABLE,
        }
    }

    fn polyglot_next_hash(
        &self,
        last_movegen_flags: &MovegenFlags,
        new_movegen_flags: &MovegenFlags,
        last_hash: PositionHash,
        mv: &Move,
    ) -> PositionHash {
        let mut hash = last_hash;
        let side = mv.piece.pcolour;
        let mut piece = mv.piece;
        hash ^= self.get_piece_hash(mv.piece, mv.from); // remove the moving piece from position
        if let Some(idx) = last_movegen_flags.polyglot_en_passant {
            hash ^= self.en_passant_table[idx % 8] // remove existing en passant index if it exists
        }
        match mv.move_type {
            MoveType::Promotion(ptype, capture) => {
                // remove piece to be captured
                if let Some(c) = capture {
                    hash ^= self.get_piece_hash(
                        Piece {
                            ptype: c,
                            pcolour: !side,
                        },
                        mv.to,
                    )
                }
                piece = Piece {
                    pcolour: side,
                    ptype,
                }
            } // set piece to promoted type
            MoveType::Capture(p) => {
                hash ^= self.get_piece_hash(
                    Piece {
                        ptype: p,
                        pcolour: !side,
                    },
                    mv.to,
                )
            } // remove captured piece
            MoveType::EnPassant(idx) => {
                // remove captured pawn
                let pawn = Piece {
                    ptype: PieceType::Pawn,
                    pcolour: !side,
                };
                hash ^= self.get_piece_hash(pawn, idx);
            }
            MoveType::Castle(c) => {
                let rook = Piece {
                    ptype: PieceType::Rook,
                    pcolour: side,
                };
                hash ^= self.get_piece_hash(rook, c.rook_from); // remove rook from its starting position
                hash ^= self.get_piece_hash(rook, c.rook_to); // set rook to new position
            }
            _ => {}
        }

        // set polyglot en passant index
        if new_movegen_flags.polyglot_en_passant.is_some() {
            hash ^= self.en_passant_table[new_movegen_flags.polyglot_en_passant.unwrap() % 8];
        }

        if last_movegen_flags.black_castle_long
            && (mv.from == LONG_BLACK_ROOK_START || mv.to == LONG_BLACK_ROOK_START)
        {
            hash ^= self.black_castle_long;
        }
        if last_movegen_flags.black_castle_short
            && (mv.from == SHORT_BLACK_ROOK_START || mv.to == SHORT_BLACK_ROOK_START)
        {
            hash ^= self.black_castle_short;
        }
        if last_movegen_flags.white_castle_long
            && (mv.from == LONG_WHITE_ROOK_START || mv.to == LONG_WHITE_ROOK_START)
        {
            hash ^= self.white_castle_long;
        }
        if last_movegen_flags.white_castle_short
            && (mv.from == SHORT_WHITE_ROOK_START || mv.to == SHORT_WHITE_ROOK_START)
        {
            hash ^= self.white_castle_short;
        }
        // reset castling flags on first king move (including castling which sets both flags false for the moving side)
        if piece.ptype == PieceType::King {
            if piece.pcolour == PieceColour::White {
                if last_movegen_flags.white_castle_long {
                    hash ^= self.white_castle_long
                }
                if last_movegen_flags.white_castle_short {
                    hash ^= self.white_castle_short
                }
            } else {
                if last_movegen_flags.black_castle_long {
                    hash ^= self.black_castle_long
                }
                if last_movegen_flags.black_castle_short {
                    hash ^= self.black_castle_short
                }
            }
        }
        hash ^= self.get_piece_hash(piece, mv.to); // set moving piece in new position
        hash ^= self.white_to_move; // switch sides
        hash
    }

    fn polyglot_full_position_hash(&self, pos: &Position) -> PositionHash {
        let mut hash = 0;
        for (i, s) in pos.pos64.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    hash ^= self.get_piece_hash(*p, i);
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
        if pos.side == PieceColour::White {
            hash ^= self.white_to_move;
        }
        if pos.movegen_flags.polyglot_en_passant.is_some() {
            hash ^= self.en_passant_table[pos.movegen_flags.polyglot_en_passant.unwrap() % 8];
        }

        hash
    }

    #[allow(dead_code)]
    fn next_hash(
        &self,
        last_movegen_flags: &MovegenFlags,
        last_hash: PositionHash,
        mv: &Move,
    ) -> PositionHash {
        let mut hash = last_hash;
        let side = mv.piece.pcolour;
        let mut piece = mv.piece;
        hash ^= self.get_piece_hash(mv.piece, mv.from); // remove the moving piece from position
        if let Some(idx) = last_movegen_flags.en_passant {
            hash ^= self.en_passant_table[idx % 8] // remove existing en passant index
        }
        match mv.move_type {
            MoveType::Promotion(ptype, capture) => {
                // remove piece to be captured
                if let Some(c) = capture {
                    hash ^= self.get_piece_hash(
                        Piece {
                            ptype: c,
                            pcolour: !side,
                        },
                        mv.to,
                    )
                }
                piece = Piece {
                    pcolour: side,
                    ptype,
                }
            } // set piece to promoted type
            MoveType::DoublePawnPush => hash ^= self.en_passant_table[mv.to % 8], // set en passant index
            MoveType::Capture(p) => {
                hash ^= self.get_piece_hash(
                    Piece {
                        ptype: p,
                        pcolour: !side,
                    },
                    mv.to,
                )
            } // remove captured piece
            MoveType::EnPassant(idx) => {
                // remove captured pawn
                let pawn = Piece {
                    ptype: PieceType::Pawn,
                    pcolour: !side,
                };
                hash ^= self.get_piece_hash(pawn, idx);
            }
            MoveType::Castle(c) => {
                let rook = Piece {
                    ptype: PieceType::Rook,
                    pcolour: side,
                };
                hash ^= self.get_piece_hash(rook, c.rook_from); // remove rook from its starting position
                hash ^= self.get_piece_hash(rook, c.rook_to); // set rook to new position
            }
            _ => {}
        }

        if last_movegen_flags.black_castle_long
            && (mv.from == LONG_BLACK_ROOK_START || mv.to == LONG_BLACK_ROOK_START)
        {
            hash ^= self.black_castle_long;
        }
        if last_movegen_flags.black_castle_short
            && (mv.from == SHORT_BLACK_ROOK_START || mv.to == SHORT_BLACK_ROOK_START)
        {
            hash ^= self.black_castle_short;
        }
        if last_movegen_flags.white_castle_long
            && (mv.from == LONG_WHITE_ROOK_START || mv.to == LONG_WHITE_ROOK_START)
        {
            hash ^= self.white_castle_long;
        }
        if last_movegen_flags.white_castle_short
            && (mv.from == SHORT_WHITE_ROOK_START || mv.to == SHORT_WHITE_ROOK_START)
        {
            hash ^= self.white_castle_short;
        }
        // reset castling flags on first king move (including castling which sets both flags false for the moving side)
        if piece.ptype == PieceType::King {
            if piece.pcolour == PieceColour::White {
                if last_movegen_flags.white_castle_long {
                    hash ^= self.white_castle_long
                }
                if last_movegen_flags.white_castle_short {
                    hash ^= self.white_castle_short
                }
            } else {
                if last_movegen_flags.black_castle_long {
                    hash ^= self.black_castle_long
                }
                if last_movegen_flags.black_castle_short {
                    hash ^= self.black_castle_short
                }
            }
        }
        hash ^= self.get_piece_hash(piece, mv.to); // set moving piece in new position
        hash ^= self.white_to_move; // switch sides
        hash
    }

    #[allow(dead_code)]
    fn full_position_hash(&self, pos: &Position) -> PositionHash {
        let mut hash = 0;
        for (i, s) in pos.pos64.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    hash ^= self.get_piece_hash(*p, i);
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
        if pos.side == PieceColour::White {
            hash ^= self.white_to_move;
        }

        hash
    }

    fn board_state_hash(
        &self,
        current_hash: PositionHash,
        occurrences: u8,
        halfmove_count: u32,
    ) -> PositionHash {
        current_hash
            ^ self.get_occurrences_hash(occurrences)
            ^ self.get_halfmove_count_hash(halfmove_count)
    }

    #[inline(always)]
    const fn get_halfmove_count_hash(&self, halfmove_count: u32) -> PositionHash {
        self.halfmove_count[halfmove_count as usize]
    }

    #[inline(always)]
    const fn get_occurrences_hash(&self, occurrences: u8) -> PositionHash {
        match occurrences {
            1 => self.occurrences[0],
            2 => self.occurrences[1],
            3 => self.occurrences[2],
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    const fn get_piece_hash(&self, piece: Piece, square_idx: usize) -> PositionHash {
        // unsafe {
        //     *self.pos_table.get_unchecked(square_idx).get_unchecked(Self::get_piece_idx(piece))
        // }
        self.pos_table[square_idx][Self::get_piece_idx(piece)]
    }

    #[inline(always)]
    const fn get_piece_idx(piece: Piece) -> usize {
        match piece.pcolour {
            PieceColour::White => match piece.ptype {
                PieceType::Pawn => 0,
                PieceType::Knight => 1,
                PieceType::Bishop => 2,
                PieceType::Rook => 3,
                PieceType::Queen => 4,
                PieceType::King => 5,
            },
            PieceColour::Black => match piece.ptype {
                PieceType::Pawn => 6,
                PieceType::Knight => 7,
                PieceType::Bishop => 8,
                PieceType::Rook => 9,
                PieceType::Queen => 10,
                PieceType::King => 11,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute_zobrist_hash_from_fen(fen_str: &str) -> u64 {
        // BoardState uses this module to compute the hash
        let bs = crate::BoardState::from(fen_str.parse::<crate::fen::FEN>().unwrap());

        ZOBRIST_HASH_TABLE.polyglot_full_position_hash(bs.position())
    }

    #[test]
    fn test_zobrist_hashes() {
        let test_cases = vec![
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                0x463b96181691fc9c,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
                0x823c9b50fd114196,
            ),
            (
                "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
                0x0756b94461c50fb0,
            ),
            (
                "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2",
                0x662fafb965db29d4,
            ),
            (
                "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
                0x22a48b5a8e47ff78,
            ),
            (
                "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR b kq - 0 3",
                0x652a607ca3f242c1,
            ),
            (
                "rnbq1bnr/ppp1pkpp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR w - - 0 4",
                0x00fdd303c946bdd9,
            ),
            (
                "rnbqkbnr/p1pppppp/8/8/PpP4P/8/1P1PPPP1/RNBQKBNR b KQkq c3 0 3",
                0x3c8123ea7b067637,
            ),
            (
                "rnbqkbnr/p1pppppp/8/8/P6P/R1p5/1P1PPPP1/1NBQKBNR b Kkq - 0 4",
                0x5c3f9b829b279560,
            ),
        ];

        for (fen, expected_hash) in test_cases {
            let computed_hash = compute_zobrist_hash_from_fen(fen);
            assert_eq!(computed_hash, expected_hash, "FEN: {}", fen);
        }
    }
}
