use core::panic;
use rand::Rng;

use static_init::dynamic;

use crate::movegen::*;

#[dynamic]
static ZOBRIST_HASH_TABLE: ZobristHashTable = ZobristHashTable::new();

const ABOVE_BELOW: usize = 8; // 8 indexes from i is the square directly above/below in the pos64 array

pub type Pos64 = [Square; 64];
pub type PositionHash = u64;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DefendMap([bool; 64]);
#[derive(Debug, PartialEq, Clone)]
pub struct AttackMap(Vec<Move>);

impl AttackMap {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl DefendMap {
    fn new() -> Self {
        Self([false; 64])
    }

    fn clear(&mut self) {
        self.0 = [false; 64];
    }
}

impl MoveMap for DefendMap {
    fn add_move(&mut self, mv: &Move) {
        self.0[mv.to] = true;
    }
}

impl MoveMap for AttackMap {
    fn add_move(&mut self, mv: &Move) {
        self.0.push(*mv);
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub position: Pos64,
    pub side: PieceColour,
    pub movegen_flags: MovegenFlags,
    defend_map: DefendMap, // map of squares opposite colour is defending
    attack_map: AttackMap, // map of moves from attacking side
    wking_idx: usize,
    bking_idx: usize,
}

struct ZobristHashTable {
    pos_table: [[PositionHash; 12]; 64],
    en_passant_table: [PositionHash; 8], // 8 possible files that an en passant move can be made
    black_to_move: PositionHash,
    white_castle_long: PositionHash,
    black_castle_long: PositionHash,
    white_castle_short: PositionHash,
    black_castle_short: PositionHash,
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
        let white_long_castle = rng.gen();
        let black_long_castle = rng.gen();
        let white_short_castle = rng.gen();
        let black_short_castle = rng.gen();
        Self {
            pos_table,
            en_passant_table,
            black_to_move,
            white_castle_long: white_long_castle,
            black_castle_long: black_long_castle,
            white_castle_short: white_short_castle,
            black_castle_short: black_short_castle,
        }
    }

    fn get_piece_hash(&self, piece: &Piece, square_idx: usize) -> PositionHash {
        self.pos_table[square_idx][Self::get_piece_idx(piece)]
    }

    fn get_piece_idx(piece: &Piece) -> usize {
        match piece.pcolour {
            PieceColour::White => {
                match piece.ptype {
                    PieceType::Pawn => 0,
                    PieceType::Knight => 1,
                    PieceType::Bishop => 2,
                    PieceType::Rook => 3,
                    PieceType::Queen => 4,
                    PieceType::King => 5,
                    PieceType::None => { panic!("PieceType::None in get_piece_idx()") }
                }
            }
            PieceColour::Black => {
                match piece.ptype {
                    PieceType::Pawn => 6,
                    PieceType::Knight => 7,
                    PieceType::Bishop => 8,
                    PieceType::Rook => 9,
                    PieceType::Queen => 10,
                    PieceType::King => 11,
                    PieceType::None => { panic!("PieceType::None in get_piece_idx()") }
                }
            }
            PieceColour::None => { panic!("PieceColour::None in get_piece_idx()") }
        }
    }
}

impl Position {
    pub fn pos_hash(&self) -> PositionHash {
        let mut hash = 0;
        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    hash ^= ZOBRIST_HASH_TABLE.get_piece_hash(p, i);
                }
                Square::Empty => {
                    continue;
                }
            }
        }
        if self.movegen_flags.white_castle_long {
            hash ^= ZOBRIST_HASH_TABLE.white_castle_long;
        }
        if self.movegen_flags.black_castle_long {
            hash ^= ZOBRIST_HASH_TABLE.black_castle_long;
        }
        if self.movegen_flags.white_castle_short {
            hash ^= ZOBRIST_HASH_TABLE.white_castle_short;
        }
        if self.movegen_flags.black_castle_short {
            hash ^= ZOBRIST_HASH_TABLE.black_castle_short;
        }
        if self.movegen_flags.en_passant.is_some() {
            hash ^=
                ZOBRIST_HASH_TABLE.en_passant_table
                    [(self.movegen_flags.en_passant.unwrap() % 8) as usize];
        }
        if self.side == PieceColour::Black {
            hash ^= ZOBRIST_HASH_TABLE.black_to_move;
        }

        hash
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
            wking_idx: 60,
            bking_idx: 4,
        };
        new.gen_maps();
        new
    }

    // TODO MOVE TO BOARDSTATE STRUCT, MAKES NO SENSE TO BE IMPLEMENTED HERE TODO
    pub fn from_fen(fen: &str) -> Self {
        let mut pos: Pos64 = [Square::Empty; 64];
        let fen_vec: Vec<&str> = fen.split(' ').collect();

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
        if fen_vec[3] != "-" {
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
            side,
            movegen_flags,
            defend_map: DefendMap::new(),
            attack_map: AttackMap::new(),
            wking_idx: 0,
            bking_idx: 0,
        };
        for (i, s) in new.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.ptype == PieceType::King {
                        if p.pcolour == PieceColour::White {
                            new.wking_idx = i;
                        } else {
                            new.bking_idx = i;
                        }
                    }
                }
                Square::Empty => {}
            }
        }
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

    // TODO maybe consolidate all movegen fag updates into one place if possible?
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
    fn test_clone(&self) -> Self {
        Self {
            position: self.position,
            side: self.side,
            movegen_flags: self.movegen_flags,
            defend_map: self.defend_map,
            // create new attack map with empty vec, because it's not needed for testing legality.
            attack_map: AttackMap::new(),
            wking_idx: self.wking_idx,
            bking_idx: self.bking_idx,
        }
    }

    fn is_move_legal(&self, mv: &Move) -> bool {
        // TODO defend map only used for castling, so maybe look at where defend map is necessary

        // tests before cloning position
        if mv.piece.ptype == PieceType::King {
            if self.is_defended(mv.to) {
                return false;
            }
            if let MoveType::Castle(castle_mv) = mv.move_type {
                // if any square the king moves from, through or to are defended, move isnt legal
                return !(
                    self.is_defended(castle_mv.king_squares.0) ||
                    self.is_defended(castle_mv.king_squares.1) ||
                    self.is_defended(castle_mv.king_squares.2)
                );
            }
        }

        let mut test_pos = self.test_clone();
        test_pos.set_king_position(mv);

        if let MoveType::EnPassant(ep_capture) = mv.move_type {
            test_pos.position[ep_capture] = Square::Empty;
        }

        test_pos.position[mv.to] = self.position[mv.from];
        test_pos.position[mv.from] = Square::Empty;

        !test_pos.is_in_check_legal_check()
    }

    fn toggle_side(&mut self) {
        self.side = if self.side == PieceColour::White {
            PieceColour::Black
        } else {
            PieceColour::White
        };
    }

    pub fn is_defended(&self, i: usize) -> bool {
        self.defend_map.0[i]
    }
    // TODO seperate functions that rely on maps and the leegal check ones that dont. One is an incomplete state and the other is complete
    fn is_in_check_legal_check(&self) -> bool {
        movegen_in_check(&self.position, self.get_king_idx())
    }
    fn get_king_idx(&self) -> usize {
        if self.side == PieceColour::White { self.wking_idx } else { self.bking_idx }
    }

    pub fn is_in_check(&self) -> bool {
        self.is_defended(self.get_king_idx())
        //movegen_in_check(&self.position, side_king, self.side)
        // for (i, s) in self.position.iter().enumerate() {
        //     match s {
        //         Square::Piece(p) => {
        //             if matches!(p.ptype, PieceType::King) && p.pcolour == self.side {
        //                 if self.is_defended(i) {
        //                     return true;
        //                 }
        //                 break;
        //             }
        //         }
        //         Square::Empty => {}
        //     }
        // }
        // return false;
    }

    pub fn get_legal_moves(&self) -> &Vec<Move> {
        // let mut legal_moves = Vec::new();
        // for mv in &self.attack_map.0 {
        //     if self.is_move_legal(mv) {
        //         legal_moves.push(mv);
        //     }
        // }

        // legal_moves
        &self.attack_map.0
    }

    // sets enpassant movegen flag to Some(idx of pawn that can be captured), if the move is a double pawn push
    fn set_en_passant_flag(&mut self, mv: &Move) {
        if mv.move_type == MoveType::DoublePawnPush {
            self.movegen_flags.en_passant = Some(mv.to);
        } else {
            self.movegen_flags.en_passant = None;
        }
    }

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
        match mv.from {
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

    pub fn gen_maps(&mut self) {
        self.defend_map.clear();
        self.attack_map.clear();
        let mut attack_map = AttackMap::new();
        // movegen_pos(&self.position, &self.movegen_flags, self.side, &mut self.attack_map, &mut self.defend_map);

        for (i, s) in self.position.iter().enumerate() {
            match s {
                Square::Piece(p) => {
                    if p.pcolour != self.side {
                        movegen(
                            &self.position,
                            &self.movegen_flags,
                            p,
                            i,
                            true,
                            &mut self.defend_map
                        );
                    } else {
                        movegen(&self.position, &self.movegen_flags, p, i, false, &mut attack_map);
                    }
                }
                Square::Empty => {
                    continue;
                }
            }
        }
        // defend map has to be updated before we can check legal moves, but it is directly updsted above
        // prune illegal moves
        let mut legal_indexes = vec![true; self.attack_map.0.len()];
        for (i, mv) in self.attack_map.0.iter().enumerate() {
            legal_indexes[i] = self.is_move_legal(mv);
        }
        attack_map.0.retain(|&mv| self.is_move_legal(&mv));

        self.attack_map = attack_map;
    }

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
                                PieceType::None => {
                                    print!(" - ");
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
                                PieceType::None => {
                                    print!(" - ");
                                }
                            }
                        }
                        PieceColour::None => {}
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
}
