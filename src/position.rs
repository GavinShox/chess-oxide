use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;
use std::vec;

use rand::Rng;

use crate::fen::FEN;
use crate::mailbox;
use crate::movegen::*;
use crate::zobrist;
use crate::zobrist::PositionHash;

const ATTACK_MAP_START_SIZE: usize = 100;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Pos64([Square; 64]);

impl Index<usize> for Pos64 {
    type Output = Square;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Pos64 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Deref for Pos64 {
    type Target = [Square; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Pos64 {
    fn default() -> Self {
        Self([Square::Empty; 64])
    }
}

impl Pos64 {
    // is a pawn of colour 'pawn_colour' is either side of square at index i, used for setting polyglot en passant flag
    #[inline(always)]
    pub fn polyglot_is_pawn_beside(&self, i: usize, pawn_colour: PieceColour) -> bool {
        let piece = Piece {
            pcolour: pawn_colour,
            ptype: PieceType::Pawn,
        };
        let left = mailbox::next_mailbox_number(i, -1);
        // valid mailbox index
        if left >= 0 {
            if let Square::Piece(p) = &self[left as usize] {
                if p == &piece {
                    return true;
                }
            }
        }
        let right = mailbox::next_mailbox_number(i, 1);
        // valid mailbox index
        if right >= 0 {
            if let Square::Piece(p) = &self[right as usize] {
                if p == &piece {
                    return true;
                }
            }
        }
        false
    }

    // returns the index of the rook closest to the 'a' file (queen side)
    pub fn a_side_outer_rook_idx(&self, colour: PieceColour) -> usize {
        // first rook encountered will be the closest to 'a' file indexes 0 or 56
        self.iter()
            .position(|s| {
                if let Square::Piece(p) = s {
                    p.ptype == PieceType::Rook && p.pcolour == colour
                } else {
                    false
                }
            })
            .unwrap()
    }

    // returns the index of the rook closest to the 'h' file (king side)
    pub fn h_side_outer_rook_idx(&self, colour: PieceColour) -> usize {
        // first rook encountered in reverse will be the closest to 'a' file indexes 63 or 7
        self.iter()
            .rposition(|s| {
                if let Square::Piece(p) = s {
                    p.ptype == PieceType::Rook && p.pcolour == colour
                } else {
                    false
                }
            })
            .unwrap()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AttackMap(Vec<Move>);

impl AttackMap {
    fn new() -> Self {
        //Self(Vec::new())
        Self(Vec::with_capacity(ATTACK_MAP_START_SIZE))
    }

    fn new_no_alloc() -> Self {
        Self(Vec::with_capacity(0))
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl MoveMap for AttackMap {
    fn add_move(&mut self, mv: &Move) {
        self.0.push(*mv);
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub pos64: Pos64,
    pub side: PieceColour,
    pub movegen_flags: MovegenFlags,
    in_check: bool,
    attack_map: AttackMap, // map of moves from attacking side
    wking_idx: usize,
    bking_idx: usize,
}

impl Position {
    // new board with starting Position
    pub fn new_starting() -> Self {
        let mut pos: Pos64 = Pos64::default();

        let movegen_flags = MovegenFlags::default_starting();

        pos[0] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Rook,
        });
        pos[1] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Knight,
        });
        pos[2] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Bishop,
        });
        pos[3] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Queen,
        });
        pos[4] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::King,
        });
        pos[5] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Bishop,
        });
        pos[6] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Knight,
        });
        pos[7] = Square::Piece(Piece {
            pcolour: PieceColour::Black,
            ptype: PieceType::Rook,
        });
        for i in 8..16 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::Black,
                ptype: PieceType::Pawn,
            });
        }
        for i in 16..48 {
            pos[i] = Square::Empty;
        }
        for i in 48..56 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::White,
                ptype: PieceType::Pawn,
            });
        }
        pos[56] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Rook,
        });
        pos[57] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Knight,
        });
        pos[58] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Bishop,
        });
        pos[59] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Queen,
        });
        pos[60] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::King,
        });
        pos[61] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Bishop,
        });
        pos[62] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Knight,
        });
        pos[63] = Square::Piece(Piece {
            pcolour: PieceColour::White,
            ptype: PieceType::Rook,
        });

        let mut new = Self {
            pos64: pos,
            side: PieceColour::White,
            in_check: false,
            movegen_flags,
            attack_map: AttackMap::new(),
            wking_idx: 60,
            bking_idx: 4,
        };
        new.gen_maps();
        new
    }

    // derive a chess960 position from a number between 0 and 959 using fischer random numbering scheme
    // https://en.wikipedia.org/wiki/Fischer_random_chess_numbering_scheme
    pub fn new_chess960_number_derive(mut number: usize) -> Self {
        assert!(number < 960); // todo should proper error handling be done
        let mut pieces = vec![PieceType::King; 8]; // placeholder piecetypes

        // get dark square bishop index first as this number derivation scheme
        // is from perspective of white's starting pieces so these are light squares from white's perspective
        let dark_sq_idxs = vec![1, 3, 5, 7];
        pieces[dark_sq_idxs[number % 4]] = PieceType::Bishop;
        number /= 4;

        // get light square bishop index
        let light_sq_idxs = vec![0, 2, 4, 6];
        pieces[light_sq_idxs[number % 4]] = PieceType::Bishop;
        number /= 4;

        // set queen position
        let mut queen_offset = number % 6;
        number /= 6;
        // we skip over queen_offset number of placeholder piecetypes (kings) before assigning queen
        for piece in pieces.iter_mut() {
            if *piece == PieceType::King {
                if queen_offset == 0 {
                    *piece = PieceType::Queen;
                    break;
                }
                queen_offset -= 1;
            }
        }

        // set knights based off below table
        const KNIGHT_OFFSETS: [(usize, usize); 10] = [
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 3),
            (2, 4),
            (3, 4),
        ];
        let mut knight_offset_1 = KNIGHT_OFFSETS[number].0;
        let mut knight_offset_2 = KNIGHT_OFFSETS[number].1;
        // will both be guaranteed overwritten, rust compiler doesn't allow these to be uninitialised
        let mut knight_piece_1_idx = usize::MAX;
        let mut knight_piece_2_idx = usize::MAX;
        for (i, piece) in pieces.iter().enumerate() {
            if *piece == PieceType::King {
                if knight_offset_1 == 0 {
                    knight_piece_1_idx = i;
                    break;
                }
                knight_offset_1 -= 1;
            }
        }
        for (i, piece) in pieces.iter().enumerate() {
            if *piece == PieceType::King {
                if knight_offset_2 == 0 {
                    knight_piece_2_idx = i;
                    break;
                }
                knight_offset_2 -= 1;
            }
        }
        pieces[knight_piece_1_idx] = PieceType::Knight;
        pieces[knight_piece_2_idx] = PieceType::Knight;

        // finally rooks and king in last 3 spots
        let mut king_rooks = vec![PieceType::Rook, PieceType::King, PieceType::Rook];
        for piece in pieces.iter_mut() {
            if *piece == PieceType::King {
                *piece = king_rooks.pop().unwrap();
            }
        }

        Self::new_from_piecetypes(pieces)
    }

    pub fn new_chess960_random() -> Self {
        Self::new_chess960_number_derive(rand::thread_rng().gen_range(0..960))
    }

    // takes a chess960 Vec<PieceType> back rank and generates a position
    fn new_from_piecetypes(pieces: Vec<PieceType>) -> Self {
        let king_start = pieces.iter().position(|&x| x == PieceType::King).unwrap();
        let long_rook_start = pieces.iter().position(|&x| x == PieceType::Rook).unwrap();
        let short_rook_start = pieces.iter().rposition(|&x| x == PieceType::Rook).unwrap();

        let movegen_flags = MovegenFlags {
            white_castle_short: true,
            white_castle_long: true,
            black_castle_short: true,
            black_castle_long: true,
            en_passant: None,
            polyglot_en_passant: None,
            long_white_rook_start: 56 + long_rook_start,
            short_white_rook_start: 56 + short_rook_start,
            long_black_rook_start: long_rook_start,
            short_black_rook_start: short_rook_start,
        };

        let mut pos: Pos64 = Pos64::default();

        for i in 0..8 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::Black,
                ptype: pieces[i],
            });
        }
        for i in 8..16 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::Black,
                ptype: PieceType::Pawn,
            });
        }
        for i in 16..48 {
            pos[i] = Square::Empty;
        }
        for i in 48..56 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::White,
                ptype: PieceType::Pawn,
            });
        }
        for i in 56..64 {
            pos[i] = Square::Piece(Piece {
                pcolour: PieceColour::White,
                ptype: pieces[i - 56],
            });
        }

        let mut new = Self {
            pos64: pos,
            side: PieceColour::White,
            in_check: false,
            movegen_flags,
            attack_map: AttackMap::new(),
            wking_idx: 56 + king_start,
            bking_idx: king_start,
        };
        new.gen_maps();
        new
    }

    pub(crate) fn new_from_pub_parts(
        pos64: Pos64,
        side: PieceColour,
        movegen_flags: MovegenFlags,
    ) -> Self {
        let mut new = Self {
            pos64,
            side,
            in_check: false,
            movegen_flags,
            attack_map: AttackMap::new(),
            wking_idx: 0,
            bking_idx: 0,
        };
        new.update_king_idx();
        new.gen_maps();
        new
    }

    // Assumes a legal move, no legality checks are done, so no bounds checking is done here
    pub fn new_position(&self, mv: &Move) -> Self {
        let mut new_pos = self.clone();
        new_pos.set_en_passant_flag(mv);
        new_pos.set_castle_flags(mv);
        new_pos.set_king_position(mv);

        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                // en passant, 'to' square is different from the captured square
                new_pos.pos64[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                if castle_mv.rook_from != castle_mv.rook_to {
                    new_pos.pos64[castle_mv.rook_to] = Square::Piece(Piece {
                        pcolour: self.side,
                        ptype: PieceType::Rook,
                    });
                    new_pos.pos64[castle_mv.rook_from] = Square::Empty;
                }

                if mv.from != mv.to {
                    new_pos.pos64[mv.to] = Square::Piece(Piece {
                        pcolour: self.side,
                        ptype: PieceType::King,
                    });
                    if castle_mv.rook_to != mv.from {
                        new_pos.pos64[mv.from] = Square::Empty;
                    }
                }
                new_pos.toggle_side();
                new_pos.gen_maps();
                return new_pos;
            }
            MoveType::Promotion(ptype, _) => match &mut new_pos.pos64[mv.from] {
                Square::Piece(p) => {
                    p.ptype = ptype;
                }
                Square::Empty => {
                    unreachable!();
                }
            },
            _ => {}
        }

        new_pos.pos64[mv.to] = new_pos.pos64[mv.from];
        new_pos.pos64[mv.from] = Square::Empty;

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos
    }

    #[inline(always)]
    pub fn pos_hash(&self) -> PositionHash {
        zobrist::pos_hash(self)
    }

    #[inline(always)]
    fn update_king_idx(&mut self) {
        for (i, s) in self.pos64.iter().enumerate() {
            if let Square::Piece(p) = s {
                if p.ptype == PieceType::King {
                    if p.pcolour == PieceColour::White {
                        self.wking_idx = i;
                    } else {
                        self.bking_idx = i;
                    }
                }
            }
        }
    }

    // TODO maybe consolidate all movegen flag updates into one place if possible?
    #[inline(always)]
    fn set_king_position(&mut self, mv: &Move) {
        if mv.piece.ptype == PieceType::King {
            if mv.piece.pcolour == PieceColour::White {
                self.wking_idx = mv.to;
            } else {
                self.bking_idx = mv.to;
            }
        }
    }
    // clone function for is_move_legal. Avoids expensive clone attack map
    #[inline(always)]
    fn test_clone(&self) -> Self {
        Self {
            pos64: self.pos64,
            side: self.side,
            in_check: self.in_check,
            movegen_flags: self.movegen_flags,
            // create new attack map with empty vec, because it's not needed for testing legality.
            attack_map: AttackMap::new_no_alloc(),
            wking_idx: self.wking_idx,
            bking_idx: self.bking_idx,
        }
    }

    pub fn is_move_legal(&self, mv: &Move) -> bool {
        if mv.piece.ptype == PieceType::King {
            if let MoveType::Castle(castle_mv) = mv.move_type {
                // can't castle out of check
                if self.in_check {
                    return false;
                }

                let king_square = Square::Piece(Piece {
                    pcolour: self.side,
                    ptype: PieceType::King,
                });

                let rook_square = Square::Piece(Piece {
                    pcolour: self.side,
                    ptype: PieceType::Rook,
                });

                // range ignoring starting king square as it is checked above
                let king_squares = if mv.from < mv.to {
                    // mv.to +1 for inclusive range while having same type in both branches
                    mv.from + 1..mv.to + 1
                } else {
                    // non inclusive range ignoring mv.from
                    mv.to..mv.from
                };

                let mut test_pos = self.test_clone();

                if mv.from != mv.to {
                    for i in king_squares {
                        test_pos.pos64[i] = king_square;
                        let last_king_idx = test_pos.get_king_idx();
                        test_pos.pos64[last_king_idx] = Square::Empty;
                        test_pos.set_king_idx(i);

                        if movegen_in_check(&test_pos.pos64, i) {
                            return false;
                        }
                    }
                }

                // only needed for chess960 positions where moving your rook can open a discovered check
                // example position: (wKe1, wRb1, bKe8, bRa1) white castles a-side (long)
                test_pos.pos64[castle_mv.rook_to] = rook_square;
                if castle_mv.rook_from != mv.to {
                    test_pos.pos64[castle_mv.rook_from] = Square::Empty;
                }
                if movegen_in_check(&test_pos.pos64, test_pos.get_king_idx()) {
                    return false;
                }

                return true;
            }
        }

        let mut test_pos = self.test_clone();
        test_pos.set_king_position(mv);

        if let MoveType::EnPassant(ep_capture) = mv.move_type {
            test_pos.pos64[ep_capture] = Square::Empty;
        }

        test_pos.pos64[mv.to] = test_pos.pos64[mv.from];
        test_pos.pos64[mv.from] = Square::Empty;

        !movegen_in_check(&test_pos.pos64, test_pos.get_king_idx())
    }

    #[inline(always)]
    fn toggle_side(&mut self) {
        self.side = if self.side == PieceColour::White {
            PieceColour::Black
        } else {
            PieceColour::White
        };
    }

    #[inline(always)]
    fn get_king_idx(&self) -> usize {
        if self.side == PieceColour::White {
            self.wking_idx
        } else {
            self.bking_idx
        }
    }

    #[inline(always)]
    fn set_king_idx(&mut self, idx: usize) {
        if self.side == PieceColour::White {
            self.wking_idx = idx;
        } else {
            self.bking_idx = idx;
        }
    }

    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        self.in_check
    }

    pub fn get_pseudo_legal_moves(&self) -> &Vec<Move> {
        &self.attack_map.0
    }

    pub fn get_legal_moves(&self) -> Vec<&Move> {
        let mut legal_moves = Vec::with_capacity(self.attack_map.0.len());
        for mv in &self.attack_map.0 {
            if self.is_move_legal(mv) {
                legal_moves.push(mv);
            }
        }
        legal_moves
    }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    #[inline(always)]
    fn set_en_passant_flag(&mut self, mv: &Move) {
        if mv.move_type == MoveType::DoublePawnPush {
            // if the pawn is beside an enemy pawn, set the polyglot en passant flag
            if self.pos64.polyglot_is_pawn_beside(mv.to, !self.side) {
                self.movegen_flags.polyglot_en_passant = Some(mv.to);
            } else {
                self.movegen_flags.polyglot_en_passant = None;
            }
            self.movegen_flags.en_passant = Some(mv.to);
        } else {
            self.movegen_flags.en_passant = None;
        }
    }

    #[inline(always)]
    fn set_castle_flags(&mut self, mv: &Move) {
        if mv.piece.ptype == PieceType::King {
            if mv.piece.pcolour == PieceColour::White {
                self.movegen_flags.white_castle_long = false;
                self.movegen_flags.white_castle_short = false;
            } else {
                self.movegen_flags.black_castle_long = false;
                self.movegen_flags.black_castle_short = false;
            }
        }
        if mv.from == self.movegen_flags.long_black_rook_start {
            self.movegen_flags.black_castle_long = false;
        } else if mv.from == self.movegen_flags.long_white_rook_start {
            self.movegen_flags.white_castle_long = false;
        } else if mv.from == self.movegen_flags.short_black_rook_start {
            self.movegen_flags.black_castle_short = false;
        } else if mv.from == self.movegen_flags.short_white_rook_start {
            self.movegen_flags.white_castle_short = false;
        }

        // if a rook is captured
        if mv.to == self.movegen_flags.long_black_rook_start {
            self.movegen_flags.black_castle_long = false;
        } else if mv.to == self.movegen_flags.long_white_rook_start {
            self.movegen_flags.white_castle_long = false;
        } else if mv.to == self.movegen_flags.short_black_rook_start {
            self.movegen_flags.black_castle_short = false;
        } else if mv.to == self.movegen_flags.short_white_rook_start {
            self.movegen_flags.white_castle_short = false;
        }
    }

    pub(crate) fn gen_maps(&mut self) {
        self.attack_map.clear();

        let pos64 = &self.pos64;
        let movegen_flags = &self.movegen_flags;
        for (i, s) in pos64.iter().enumerate() {
            if let Square::Piece(p) = s {
                if p.pcolour != self.side {
                    continue;
                }
                movegen(pos64, movegen_flags, *p, i, &mut self.attack_map);
            }
        }
        self.in_check = movegen_in_check(&self.pos64, self.get_king_idx());
    }
}

impl From<FEN> for Position {
    fn from(fen: FEN) -> Self {
        Self::new_from_pub_parts(fen.pos64(), fen.side(), fen.movegen_flags())
    }
}
