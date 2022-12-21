use core::panic;
use std::{ mem::MaybeUninit, default, hash::Hasher };
use std::hash::Hash;

use crate::movegen::*;

pub type Pos64 = [Square; 64];
pub type PositionHash = u64;

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub struct DefendMap([bool; 64]);
#[derive(Debug, PartialEq, Clone, Hash)]
pub struct AttackMap(Vec<Move>);

impl AttackMap {
    fn new() -> Self {
        Self(Vec::new())
    }
}

impl DefendMap {
    fn new() -> Self {
        Self([false; 64])
    }
}

impl MoveMap for DefendMap {
    fn add_move(&mut self, mv: &Move) -> () {
        self.0[mv.to] = true;
    }
}

impl MoveMap for AttackMap {
    fn add_move(&mut self, mv: &Move) -> () {
        self.0.push(*mv);
    }
}


#[derive(Debug, Clone, Hash)]
pub struct Position {
    pub position: Pos64,
    pub side: PieceColour,
    pub movegen_flags: MovegenFlags,
    defend_map: DefendMap, // map of squares opposite colour is defending
    attack_map: AttackMap, // map of moves from attacking side
}

impl Position {
    pub fn pos_hash(&self) -> PositionHash {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    // new board with starting Position
    pub fn new_starting() -> Self {
        let mut pos: Pos64 = [Square::Empty; 64];

        let movegen_flags = MovegenFlags {
            white_castle_short: true,
            white_castle_long: true,
            black_castle_short: true,
            black_castle_long: true,
            en_passant: None,
        };

        pos[0] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Rook });
        pos[1] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Knight });
        pos[2] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Bishop });
        pos[3] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Queen });
        pos[4] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::King });
        pos[5] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Bishop });
        pos[6] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Knight });
        pos[7] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Rook });
        for i in 8..16 {
            pos[i] = Square::Piece(Piece { pcolour: PieceColour::Black, ptype: PieceType::Pawn });
        }
        for i in 16..48 {
            pos[i] = Square::Empty;
        }
        for i in 48..56 {
            pos[i] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Pawn });
        }
        pos[56] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Rook });
        pos[57] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Knight });
        pos[58] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Bishop });
        pos[59] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Queen });
        pos[60] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::King });
        pos[61] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Bishop });
        pos[62] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Knight });
        pos[63] = Square::Piece(Piece { pcolour: PieceColour::White, ptype: PieceType::Rook });

        let side = PieceColour::White;

        let mut new = Self {
            position: pos,
            side,
            movegen_flags,
            defend_map: DefendMap::new(),
            attack_map: AttackMap::new(),
        };
        new.gen_maps();
        new.get_legal_moves();

        new
    }

    pub fn new_position_from_fen(fen: &str) -> Self {
        let mut pos: Pos64 = [Square::Empty; 64];
        let fen_vec: Vec<&str> = fen.split(' ').collect();
        assert_eq!(fen_vec.len(), 6);

        // first field of FEN defines the piece positions
        let mut rank_start_idx = 0;
        for rank in fen_vec[0].split('/') {
            let mut i = 0;
            for c in rank.chars() {
                let mut square = Square::Empty;
                match c {
                    'p' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Pawn,
                        });
                    }
                    'P' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Pawn,
                        });
                    }
                    'r' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Rook,
                        });
                    }
                    'R' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Rook,
                        });
                    }
                    'n' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Knight,
                        });
                    }
                    'N' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Knight,
                        });
                    }
                    'b' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Bishop,
                        });
                    }
                    'B' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Bishop,
                        });
                    }
                    'q' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::Queen,
                        });
                    }
                    'Q' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::Queen,
                        });
                    }
                    'k' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::Black,
                            ptype: PieceType::King,
                        });
                    }
                    'K' => {
                        square = Square::Piece(Piece {
                            pcolour: PieceColour::White,
                            ptype: PieceType::King,
                        });
                    }
                    x if x.is_ascii_digit() => {
                        for _ in 0..x.to_digit(10).unwrap() {
                            pos[i + rank_start_idx] = Square::Empty;
                            i += 1;
                        }
                        continue; // skip the below square assignment for pieces
                    }
                    other => {
                        panic!("invalid char in first field: {}", other);
                    }
                }
                pos[i + rank_start_idx] = square;
                i += 1;
            }
            rank_start_idx += 8; // next rank
        }

        // second filed of FEN defines which side it is to move, either 'w' or 'b'
        let mut side = PieceColour::White;
        match fen_vec[1] {
            "w" => {/* already set as white */}
            "b" => {
                side = PieceColour::Black;
            }
            other => {
                panic!("invalid second field: {}", other);
            }
        }

        // initialise movegen flags for the next two FEN fields
        let mut movegen_flags = MovegenFlags {
            white_castle_short: false,
            white_castle_long: false,
            black_castle_short: false,
            black_castle_long: false,
            en_passant: None,
        };

        // third field of FEN defines castling flags
        for c in fen_vec[2].chars() {
            match c {
                'q' => {
                    movegen_flags.black_castle_long = true;
                }
                'Q' => {
                    movegen_flags.white_castle_long = true;
                }
                'k' => {
                    movegen_flags.black_castle_short = true;
                }
                'K' => {
                    movegen_flags.white_castle_short = true;
                }
                '-' => {}
                other => panic!("invalid char in third field: {}", other),
            }
        }

        // fourth field of FEN defines en passant flag, it gives notation of the square the pawn jumped over
        if !(fen_vec[3] == "-") {
            let ep_mv_idx = Self::notation_to_index(fen_vec[3]);
            // in our struct however, we store the idx of the pawn to be captured
            let ep_flag = if side == PieceColour::White {
                ep_mv_idx + ABOVE_BELOW
            } else {
                ep_mv_idx - ABOVE_BELOW
            };
            movegen_flags.en_passant = Some(ep_flag);
        }

        // Last two fields not used here, as the 50 move rule isnt calculated in Position struct

        let mut new = Self {
            position: pos,
            side: side,
            movegen_flags,
            defend_map: DefendMap::new(),
            attack_map: AttackMap::new(),
        };
        new.gen_maps();
        new
    }

    // Assumes a legal move, no legality checks are done, so no bounds checking is done here
    pub fn new_position(&self, mv: &Move) -> Self {
        let mut new_pos = self.clone();
        new_pos.set_en_passant_flag(mv);
        new_pos.set_castle_flags(mv);

        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                // en passant, 'to' square is different from the captured square
                new_pos.position[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                new_pos.position[castle_mv.rook_to] = new_pos.position[castle_mv.rook_from];
                new_pos.position[castle_mv.rook_from] = Square::Empty;
            }
            MoveType::Promotion(ptype) => {
                match &mut new_pos.position[mv.from] {
                    Square::Piece(p) => {
                        p.ptype = ptype;
                    }
                    Square::Empty => {/* should never get here */}
                }
            }
            _ => {}
        }

        new_pos.position[mv.to] = new_pos.position[mv.from];
        new_pos.position[mv.from] = Square::Empty;

        new_pos.toggle_side();
        new_pos.gen_maps();
        new_pos
    }

    // moves piece at i, to j, without changing side, to regen defend maps to determine if the move is legal
    // legality here meaning would the move leave your king in check. Actual piece movement is done in movegen
    fn is_move_legal(&self, mv: &Move) -> bool {
        let mut test_pos = self.clone();
        // doing special move types first, to check if mv is a castling move, and return there first
        match mv.move_type {
            MoveType::EnPassant(ep_capture) => {
                test_pos.position[ep_capture] = Square::Empty;
            }
            MoveType::Castle(castle_mv) => {
                // if any square the king moves from, through or to are defended, move isnt legal
                return !(
                    test_pos.is_defended(castle_mv.king_squares.0) ||
                    test_pos.is_defended(castle_mv.king_squares.1) ||
                    test_pos.is_defended(castle_mv.king_squares.2)
                );
            }
            MoveType::Promotion(_) => {/* the piece the pawn promotes to doesn't effect legality */}
            _ => {}
        }

        // this has to be after the castleing section above, as king cant castle out of check
        test_pos.position[mv.to] = self.position[mv.from];
        test_pos.position[mv.from] = Square::Empty;

        // we only need to gen new defend map
        test_pos.gen_defend_map();

        let result = !test_pos.is_in_check();

        // self.position = original_position;
        // self.defend_map = original_defend_map;
        result
    }

    fn toggle_side(&mut self) -> () {
        self.side = if self.side == PieceColour::White {
            PieceColour::Black
        } else {
            PieceColour::White
        };
    }

    pub fn is_defended(&self, i: usize) -> bool {
        self.defend_map.0[i]
    }

    pub fn is_in_check(&self) -> bool {
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if matches!(p.ptype, PieceType::King) && p.pcolour == self.side {
                        if self.is_defended(i) {
                            return true;
                        }
                        break;
                    }
                }
                Square::Empty => {}
            }
        }
        return false;
    }

    pub fn get_legal_moves(&self) -> Vec<&Move> {
        let mut legal_moves = Vec::new();
        for mv in &self.attack_map.0 {
            if self.is_move_legal(mv) {
                legal_moves.push(mv);
            }
        }
        legal_moves
    }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    fn set_en_passant_flag(&mut self, mv: &Move) -> () {
        if mv.move_type == MoveType::DoublePawnPush {
            self.movegen_flags.en_passant = Some(mv.to);
        } else {
            self.movegen_flags.en_passant = None;
        }
    }

    fn set_castle_flags(&mut self, mv: &Move) -> () {
        match mv.from {
            WHITE_KING_START => {
                self.movegen_flags.white_castle_long = false;
                self.movegen_flags.white_castle_short = false;
            }
            BLACK_KING_START => {
                self.movegen_flags.black_castle_long = false;
                self.movegen_flags.black_castle_short = false;
            }
            LONG_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_long = false;
            }
            LONG_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_long = false;
            }
            SHORT_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_short = false;
            }
            SHORT_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_short = false;
            }
            _ => {}
        }
        // if a rook is captured
        match mv.to {
            LONG_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_long = false;
            }
            LONG_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_long = false;
            }
            SHORT_BLACK_ROOK_START => {
                self.movegen_flags.black_castle_short = false;
            }
            SHORT_WHITE_ROOK_START => {
                self.movegen_flags.white_castle_short = false;
            }
            _ => {}
        }
    }

    fn gen_defend_map(&mut self) -> () {
        //self.defend_map.clear();
        let mut defend_map = DefendMap::new();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        movegen(self, p, i, true, &mut defend_map);
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
        self.defend_map = defend_map;
    }

    pub fn gen_maps(&mut self) -> () {
        let mut defend_map = DefendMap::new();
        let mut attack_map = AttackMap::new();
        // self.defend_map.clear();
        // self.attack_map.clear();
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        movegen(self, p, i, true, &mut defend_map)
                    } else {
                        // for mv in self.movegen(p, i, false, true) {
                        //     self.attack_map.push(mv); 
                        // }
                        movegen(self, p, i, false, &mut attack_map);
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
        self.defend_map = defend_map;
        self.attack_map = attack_map;
    }

    pub fn print_board(&self) {
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

        for (num, j) in self.position.iter().enumerate() {
            match j {
                Square::Piece(p) => {
                    match p.pcolour {
                        PieceColour::White => {
                            match p.ptype {
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
                            }
                        }
                        PieceColour::Black => {
                            match p.ptype {
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
                            }
                        }
                    }
                }
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

    pub fn notation_to_index(n: &str) -> usize {
        let file: char = n.chars().nth(0).unwrap();
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
        let rank_num = i / 8 + 1;
        let rank = char::from_digit(rank_num.try_into().unwrap(), 10).unwrap();
        format!("{}{}", file, rank)
    }

    pub fn move_as_notation(p: &str, mv: &str) -> (usize, usize) {
        let i: usize = Self::notation_to_index(p);
        let j: usize = Self::notation_to_index(mv);

        (i, j)
    }
}