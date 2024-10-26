use std::fmt;
use std::str::FromStr;

use crate::errors::PGNParseError;
use crate::{board, movegen::*};
use crate::{hash_to_string, log_and_return_error};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Notation {
    piece: Option<char>,
    dis_file: Option<char>, // for disambiguating moves if required
    dis_rank: Option<char>, // for disambiguating moves if required
    capture: bool,
    to_file: char,
    to_rank: char,
    promotion: Option<char>,
    check: bool,
    checkmate: bool,
    castle_str: Option<String>,
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut notation = String::new();

        // return castling string if it exists
        if let Some(cs) = &self.castle_str {
            let mut castle_str = cs.clone();
            if self.checkmate {
                castle_str.push('#');
            } else if self.check {
                castle_str.push('+');
            }
            return write!(f, "{}", castle_str);
        }

        if let Some(piece) = self.piece {
            notation.push(piece);
        }
        if let Some(dis_file) = self.dis_file {
            notation.push(dis_file);
        }
        if let Some(dis_rank) = self.dis_rank {
            notation.push(dis_rank);
        }
        if self.capture {
            notation.push('x');
        }
        notation.push(self.to_file);
        notation.push(self.to_rank);
        if let Some(promotion) = self.promotion {
            notation.push('=');
            notation.push(promotion);
        }
        if self.checkmate {
            notation.push('#');
        } else if self.check {
            notation.push('+');
        }
        write!(f, "{}", notation)
    }
}

impl FromStr for Notation {
    type Err = PGNParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // check that str is valid ascii
        Self::validate_ascii(s)?;

        // min length is 2 (e.g. 'e4'), max length is 8 if all disambiguating notation is used and position is a check (e.g. 'Qd5xRd1+')
        Self::validate_length(s)?;

        // create new uninitialised Notation struct
        let mut notation = Self::new();

        // Handle castling strings and return as it doesn't require further parsing
        if notation.handle_castling_strings(s) {
            return Ok(notation);
        }

        // Parse the notation string
        notation.parse_notation_string(s)?;

        Ok(notation)
    }
}

// CONSTRUCTORS AND RELATED FUNCTIONS
impl Notation {
    // (private) new uninitialised Notation struct
    fn new() -> Notation {
        Notation {
            piece: None,
            dis_file: None,
            dis_rank: None,
            capture: false,
            to_file: ' ',
            to_rank: ' ',
            promotion: None,
            check: false,
            checkmate: false,
            castle_str: None,
        }
    }

    // from move with boardstate context, disambiguaating notation will only be used if required
    pub fn from_mv_with_context(
        bs_context: &board::BoardState,
        mv: &Move,
    ) -> Result<Notation, PGNParseError> {
        let legal_moves = extract_legal_moves(bs_context)?;

        // create new uninitialised Notation struct
        let mut notation = Self::new();

        // check if move is legal and if it results in check or checkmate by generating a new boardstate
        // set check and checkmate flags based off the new boardstate's gamestate
        if legal_moves.contains(mv) {
            let test_bs = bs_context.next_state(mv).unwrap(); // unwrap is safe as move is legal
            match test_bs.get_gamestate() {
                board::GameState::Check => notation.check = true, // SET CHECK FLAG
                board::GameState::Checkmate => notation.checkmate = true, // SET CHECKMATE FLAG
                _ => {}
            }
        } else {
            let err = PGNParseError::NotationParseError(format!("Move not legal: {:?}", mv));
            log_and_return_error!(err);
        }

        // set castling string if it is a castling move and return
        if let MoveType::Castle(cm) = mv.move_type {
            notation.castle_str = Some(match cm.get_castle_side() {
                // SET CASTLE STRING
                CastleSide::Short => "O-O".to_string(),
                CastleSide::Long => "O-O-O".to_string(),
            });
            return Ok(notation); // RETURN ON CASTLE MOVE
        }

        // SET PIECE CHAR
        notation.piece = ptype_to_piece_char(&mv.piece.ptype);

        // SET TO FILE AND TO RANK
        notation.to_file = index_to_file_notation(mv.to);
        notation.to_rank = index_to_rank_notation_unchecked(mv.to);

        // SET CAPTURE FLAG (Normal capture, en passant capture, or promotion capture)
        notation.capture = mv.move_type.is_capture();

        // SET PROMOTION CHAR
        notation.promotion = mv_type_to_promotion_char(&mv.move_type);

        // DISAMBIGUATING MOVES
        // pawn moves that are captures or en passants only need dis_file, otherwise only to_file and to_rank are needed
        if matches!(mv.piece.ptype, PieceType::Pawn) && notation.capture {
            // notation.capture is set above in function
            notation.dis_file = Some(index_to_file_notation(mv.from));
        } else {
            // check if there are any other pieces besides pawns that can move to the same square as the mv.piece
            let same_piece_moves: Vec<&Move> = legal_moves
                .iter()
                .filter(|m| m.piece == mv.piece && m.to == mv.to && m.from != mv.from)
                .collect();
            // if there are other pieces that can move to same square
            if !same_piece_moves.is_empty() {
                // store the current mv.from square file and rank
                let mv_from_file = index_to_file_notation(mv.from);
                let mv_from_rank = index_to_rank_notation_unchecked(mv.from);
                // keep track of whether any of the other moves have the same file or rank as the current mv.from square
                let mut same_file = false;
                let mut same_rank = false;
                // check if any of the other moves have the same file or rank as the current mv.from square
                for other_mv in same_piece_moves {
                    let other_mv_from_file = index_to_file_notation(other_mv.from);
                    let other_mv_from_rank = index_to_rank_notation_unchecked(other_mv.from);
                    if other_mv_from_file == mv_from_file {
                        same_file = true;
                    }
                    if other_mv_from_rank == mv_from_rank {
                        same_rank = true;
                    }
                }
                // disambiguate the move by setting the file, or setting the rank, or setting both if needed in that order
                if !same_file {
                    notation.dis_file = Some(mv_from_file);
                } else if !same_rank {
                    notation.dis_rank = Some(mv_from_rank);
                } else {
                    notation.dis_file = Some(mv_from_file);
                    notation.dis_rank = Some(mv_from_rank);
                }
            }
        }

        Ok(notation)
    }

    fn handle_castling_strings(&mut self, notation_str: &str) -> bool {
        let possible_castle_str = notation_str.trim_end_matches(['+', '#']);
        if possible_castle_str == "O-O" || possible_castle_str == "O-O-O" {
            self.castle_str = Some(possible_castle_str.to_string());
            self.check = notation_str.ends_with('+');
            self.checkmate = notation_str.ends_with('#');
            return true;
        }
        false
    }

    fn parse_notation_string(&mut self, notation_str: &str) -> Result<(), PGNParseError> {
        let mut chars = notation_str.char_indices();
        let mut rank_file_chars: Vec<char> = Vec::new();
        let mut piece_char: Option<char> = None;
        let mut capture = false;
        let mut promotion: Option<char> = None;
        let mut check = false;
        let mut checkmate = false;

        while let Some((i, c)) = chars.next() {
            match c {
                c if c.is_ascii_uppercase() => {
                    Self::handle_piece_char(c, &mut piece_char, notation_str, i)?;
                }
                'x' => {
                    Self::handle_capture_char(&mut capture, notation_str, i)?;
                }
                c if c.is_ascii_lowercase() || c.is_ascii_digit() => {
                    rank_file_chars.push(c);
                }
                '=' => {
                    promotion = Self::handle_promotion_char(&mut chars, notation_str, i)?;
                }
                '+' => {
                    Self::handle_check_char(&mut check, notation_str, i)?;
                }
                '#' => {
                    Self::handle_checkmate_char(&mut checkmate, notation_str, i)?;
                }
                _ => {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid character in notation (char: '{}' at index: {})",
                        c, i
                    ));
                    log_and_return_error!(err)
                }
            }
        }

        // set piece char if it is valid
        self.set_piece_char(piece_char)?;

        // set rank and file chars, checking if there are any disambiguating chars
        self.set_rank_file_chars(&rank_file_chars)?;

        // set promotion char if it is valid
        self.set_promotion_char(promotion)?;

        // set boolean flags
        self.capture = capture;
        self.check = check;
        self.checkmate = checkmate;

        Ok(())
    }

    fn set_piece_char(&mut self, piece_char: Option<char>) -> Result<(), PGNParseError> {
        if let Some(piece) = piece_char {
            if is_valid_piece(piece) {
                self.piece = Some(piece);
            } else {
                let err =
                    PGNParseError::NotationParseError(format!("Invalid piece char ({})", piece));
                log_and_return_error!(err)
            }
        }
        Ok(())
    }

    fn set_promotion_char(&mut self, promotion: Option<char>) -> Result<(), PGNParseError> {
        if let Some(promotion) = promotion {
            if is_valid_promotion(promotion) {
                self.promotion = Some(promotion);
            } else {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid promotion piece char ({})",
                    promotion
                ));
                log_and_return_error!(err)
            }
        }
        Ok(())
    }

    fn set_rank_file_chars(&mut self, rank_file_chars: &[char]) -> Result<(), PGNParseError> {
        let to_file: char;
        let to_rank: char;
        let mut dis_file = None;
        let mut dis_rank = None;
        match rank_file_chars.len() {
            2 => {
                to_file = rank_file_chars[0];
                to_rank = rank_file_chars[1];
            }
            3 => {
                if rank_file_chars[0].is_ascii_lowercase() {
                    dis_file = Some(rank_file_chars[0]); // disambiguating file
                } else {
                    dis_rank = Some(rank_file_chars[0]); // disambiguating rank
                }
                to_file = rank_file_chars[1];
                to_rank = rank_file_chars[2];
            }
            4 => {
                dis_file = Some(rank_file_chars[0]);
                dis_rank = Some(rank_file_chars[1]);
                to_file = rank_file_chars[2];
                to_rank = rank_file_chars[3];
            }
            _ => {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid move notation char(s) ({:?})",
                    rank_file_chars
                ));
                log_and_return_error!(err)
            }
        }
        if !is_valid_file(to_file)
            || !is_valid_rank(to_rank)
            || dis_file.map_or(false, |c| !is_valid_file(c))
            || dis_rank.map_or(false, |c| !is_valid_rank(c))
        {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid rank or file char(s) in vec: ({:?})",
                rank_file_chars
            ));
            log_and_return_error!(err)
        } else {
            self.to_file = to_file;
            self.to_rank = to_rank;
            self.dis_file = dis_file;
            self.dis_rank = dis_rank;
        }
        Ok(())
    }

    fn validate_ascii(notation_str: &str) -> Result<(), PGNParseError> {
        if !notation_str.is_ascii() {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid notation string: ({}) is not valid ascii",
                notation_str
            ));
            log_and_return_error!(err)
        }
        Ok(())
    }

    fn validate_length(notation_str: &str) -> Result<(), PGNParseError> {
        let str_len = notation_str.len();
        if !(2..=8).contains(&str_len) {
            let err =
                PGNParseError::NotationParseError(format!("Invalid notation length ({})", str_len));
            log_and_return_error!(err)
        }
        Ok(())
    }

    fn handle_piece_char(
        c: char,
        piece_char: &mut Option<char>,
        notation_str: &str,
        i: usize,
    ) -> Result<(), PGNParseError> {
        if piece_char.is_none() {
            *piece_char = Some(c);
        } else {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid notation, multiple uppercase piece chars (char: '{}' at index: {})",
                notation_str, i
            ));
            log_and_return_error!(err)
        }
        Ok(())
    }

    fn handle_capture_char(
        capture: &mut bool,
        notation_str: &str,
        i: usize,
    ) -> Result<(), PGNParseError> {
        if !*capture {
            // must be at least 2 more chars after 'x' for a valid capture
            if (notation_str.len() - i) < 3 {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid notation, no rank or file after capture char (char: '{}' at index: {})",
                    notation_str, i
                ));
                log_and_return_error!(err)
            } else {
                *capture = true;
            }
        } else {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid notation, multiple capture chars (char: '{}' at index: {})",
                notation_str, i
            ));
            log_and_return_error!(err)
        }
        Ok(())
    }

    fn handle_promotion_char(
        chars: &mut std::str::CharIndices,
        notation_str: &str,
        i: usize,
    ) -> Result<Option<char>, PGNParseError> {
        let n = chars.next();
        match n {
            Some((_, c)) => match c {
                'Q' | 'R' | 'B' | 'N' => Ok(Some(c)),
                _ => {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid promotion piece (char: '{}' at index: {})",
                        c, i
                    ));
                    log_and_return_error!(err)
                }
            },
            None => {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid notation, no promotion piece after '=' (char: '{}' at index: {})",
                    notation_str, i
                ));
                log_and_return_error!(err)
            }
        }
    }

    fn handle_check_char(
        check: &mut bool,
        notation_str: &str,
        i: usize,
    ) -> Result<(), PGNParseError> {
        if !*check {
            *check = true;
        } else {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid notation, multiple check chars (char: '{}' at index: {})",
                notation_str, i
            ));
            log_and_return_error!(err)
        }
        Ok(())
    }

    fn handle_checkmate_char(
        checkmate: &mut bool,
        notation_str: &str,
        i: usize,
    ) -> Result<(), PGNParseError> {
        if !*checkmate {
            *checkmate = true;
        } else {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid notation, multiple checkmate chars (char: '{}' at index: {})",
                notation_str, i
            ));
            log_and_return_error!(err)
        }
        Ok(())
    }
}

impl Notation {
    // tries to find a move, and disambiguates as best as possible, for use in PGN import format so if it is missing some disambiguating information but the move can still be identified, it is fine
    pub fn to_move_with_context(
        &self,
        bs_context: &board::BoardState,
    ) -> Result<Move, PGNParseError> {
        let legal_moves = extract_legal_moves(bs_context)?;
        let possible_moves = self.filter_possible_moves(legal_moves);
        if possible_moves.len() == 1 {
            Ok(*possible_moves[0])
        } else if possible_moves.len() > 1 {
            let mut dis_file_possible_idxs = None;
            let mut dis_rank_possible_idxs = None;

            if let Some(dis_file_char) = self.dis_file {
                dis_file_possible_idxs = Some(file_notation_to_indexes_unchecked(dis_file_char));
            }
            if let Some(dis_rank_char) = self.dis_rank {
                dis_rank_possible_idxs = Some(rank_notation_to_indexes_unchecked(dis_rank_char));
            }

            let mut possible_dis_moves = Vec::new();
            for mv in &possible_moves {
                if let (Some(dis_file_idxs), Some(dis_rank_idxs)) =
                    (dis_file_possible_idxs, dis_rank_possible_idxs)
                {
                    if dis_file_idxs.contains(&mv.from) && dis_rank_idxs.contains(&mv.from) {
                        possible_dis_moves.push(*mv);
                    }
                } else if let Some(dis_file_idxs) = dis_file_possible_idxs {
                    if dis_file_idxs.contains(&mv.from) {
                        possible_dis_moves.push(*mv);
                    }
                } else if let Some(dis_rank_idxs) = dis_rank_possible_idxs {
                    if dis_rank_idxs.contains(&mv.from) {
                        possible_dis_moves.push(*mv);
                    }
                }
            }

            if possible_dis_moves.len() == 1 {
                return Ok(*possible_dis_moves[0]);
            } else {
                let err = PGNParseError::MoveNotFound(format!(
                    "No legal move found for notation ({}) in BoardState (hash: {}) => Could not use notation to disambiguate between multiple possible moves: {:?}",
                    self,
                    hash_to_string(bs_context.board_hash),
                    possible_moves
                ));
                log_and_return_error!(err)
            }
        } else {
            let err = PGNParseError::MoveNotFound(format!(
                "No legal move found for notation ({}) in BoardState (hash: {})",
                self,
                hash_to_string(bs_context.board_hash)
            ));
            log_and_return_error!(err)
        }
    }

    fn get_piece_type(&self) -> Option<PieceType> {
        match self.piece {
            Some('N') => Some(PieceType::Knight),
            Some('B') => Some(PieceType::Bishop),
            Some('R') => Some(PieceType::Rook),
            Some('Q') => Some(PieceType::Queen),
            Some('K') => Some(PieceType::King),
            Some(_) => {
                unreachable!("Invalid piece char in get_piece_type function")
            }
            None => None,
        }
    }

    fn get_promotion_piece_type(&self) -> Option<PieceType> {
        match self.promotion {
            Some('Q') => Some(PieceType::Queen),
            Some('R') => Some(PieceType::Rook),
            Some('B') => Some(PieceType::Bishop),
            Some('N') => Some(PieceType::Knight),
            Some(_) => {
                unreachable!("Invalid promotion char in get_promotion_piece_type function")
            }
            None => None,
        }
    }

    fn get_castle_side(&self) -> Option<CastleSide> {
        if let Some(castle_str) = &self.castle_str {
            match castle_str.as_str() {
                "O-O" => Some(CastleSide::Short),
                "O-O-O" => Some(CastleSide::Long),
                _ => {
                    unreachable!("Invalid castle string in get_castle_side function");
                }
            }
        } else {
            None
        }
    }

    fn filter_possible_moves<'a>(&self, moves: &'a [Move]) -> Vec<&'a Move> {
        moves
            .iter()
            .filter(|mv| {
                if let Some(castle_side) = self.get_castle_side() {
                    if let MoveType::Castle(cm) = mv.move_type {
                        return cm.get_castle_side() == castle_side;
                    }
                }

                if let Some(piece) = self.get_piece_type() {
                    if mv.piece.ptype != piece {
                        return false;
                    }
                } else {
                    // PAWN HANDLING - no piece char can only be a castle move which is already handled, or a pawn move
                    if mv.piece.ptype != PieceType::Pawn {
                        return false;
                    }
                }

                if self.to_file != index_to_file_notation(mv.to)
                    || self.to_rank != index_to_rank_notation_unchecked(mv.to)
                {
                    return false;
                }
                if self.capture && !mv.move_type.is_capture() {
                    return false;
                }
                if let Some(promotion) = self.get_promotion_piece_type() {
                    if let MoveType::Promotion(promotion_ptype, _) = mv.move_type {
                        if promotion_ptype != promotion {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                // if move passes all checks, return true
                true
            })
            .collect::<Vec<&Move>>()
    }
}

// get legal moves from BoardState, on error return BoardStateError wrapped in PGNParseError
fn extract_legal_moves(bs: &board::BoardState) -> Result<&[Move], PGNParseError> {
    match bs.get_legal_moves() {
        Ok(moves) => Ok(moves),
        Err(e) => {
            let err = PGNParseError::NotationParseError(format!(
                "Error getting legal moves in BoardState: {}",
                e
            ));
            log_and_return_error!(err)
        }
    }
}

#[inline]
fn ptype_to_piece_char(ptype: &PieceType) -> Option<char> {
    match ptype {
        PieceType::Pawn => None,
        PieceType::Knight => Some('N'),
        PieceType::Bishop => Some('B'),
        PieceType::Rook => Some('R'),
        PieceType::Queen => Some('Q'),
        PieceType::King => Some('K'),
    }
}

#[inline]
fn mv_type_to_promotion_char(mv_type: &MoveType) -> Option<char> {
    if let MoveType::Promotion(promotion, _) = mv_type {
        match promotion {
            PieceType::Queen => Some('Q'),
            PieceType::Rook => Some('R'),
            PieceType::Bishop => Some('B'),
            PieceType::Knight => Some('N'),
            _ => unreachable!("Invalid MoveType. Not possible from crate::movegen"),
        }
    } else {
        None
    }
}

#[inline]
fn is_valid_file(file: char) -> bool {
    file.is_ascii_lowercase() && ('a'..='h').contains(&file)
}

#[inline]
fn is_valid_rank(rank: char) -> bool {
    rank.is_ascii_digit() && ('1'..='8').contains(&rank)
}

#[inline]
fn is_valid_piece(piece: char) -> bool {
    let valid_pieces = ['P', 'N', 'B', 'R', 'Q', 'K'];
    piece.is_ascii_uppercase() && valid_pieces.contains(&piece)
}

#[inline]
fn is_valid_promotion(promotion: char) -> bool {
    let valid_promotions = ['Q', 'R', 'B', 'N'];
    promotion.is_ascii_uppercase() && valid_promotions.contains(&promotion)
}

#[inline]
fn index_to_file_notation(i: usize) -> char {
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
fn index_to_rank_notation_unchecked(i: usize) -> char {
    let rank_num = 8 - i / 8;
    char::from_digit(rank_num.try_into().unwrap(), 10).unwrap()
}

fn rank_notation_to_indexes_unchecked(r: char) -> [usize; 8] {
    let rank_num = r.to_digit(10).unwrap() as usize;
    let rank_starts = [56, 48, 40, 32, 24, 16, 8, 0]; // 1st to 8th rank starting indexes
    let mut indexes = [0; 8];
    for (i, j) in indexes.iter_mut().enumerate() {
        *j = rank_starts[rank_num - 1] + i;
    }
    indexes
}

fn file_notation_to_indexes_unchecked(f: char) -> [usize; 8] {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_notation_new() {
        let notation = Notation::new();

        assert!(notation.piece.is_none());
        assert!(notation.dis_file.is_none());
        assert!(notation.dis_rank.is_none());
        assert!(!notation.capture);
        assert_eq!(notation.to_file, ' ');
        assert_eq!(notation.to_rank, ' ');
        assert!(notation.promotion.is_none());
        assert!(!notation.check);
        assert!(!notation.checkmate);
    }

    #[test]
    fn test_notation_to_string() {
        let mut notation = Notation::new();
        notation.piece = Some('N');
        notation.to_file = 'f';
        notation.to_rank = '3';
        notation.capture = true;

        assert_eq!(notation.to_string(), "Nxf3");
    }

    #[test]
    fn test_notation_from_mv_with_context() {
        let bs = board::BoardState::new_starting();
        let mv = Move {
            piece: Piece {
                ptype: PieceType::Knight,
                pcolour: PieceColour::White,
            },
            from: 62,
            to: 45,
            move_type: MoveType::Normal,
        };
        let notation = Notation::from_mv_with_context(&bs, &mv);
        assert!(notation.is_ok());
        assert_eq!(notation.unwrap().to_string(), "Nf3");
    }

    #[test]
    fn test_notation_from_str() -> Result<(), PGNParseError> {
        let notation_str = "Qf3xf5+";
        let notation = Notation::from_str(notation_str);
        match notation {
            Ok(notation) => {
                assert_eq!(notation.piece, Some('Q'));
                assert_eq!(notation.to_file, 'f');
                assert_eq!(notation.to_rank, '5');
                assert_eq!(notation.capture, true);
                assert_eq!(notation.check, true);
                assert_eq!(notation.dis_file.is_some(), true);
                assert_eq!(notation.dis_rank.is_some(), true);
                assert_eq!(notation.dis_file.unwrap(), 'f');
                assert_eq!(notation.dis_rank.unwrap(), '3');
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_notation_from_str_castle() -> Result<(), PGNParseError> {
        let notation_str = "O-O";
        let notation = Notation::from_str(notation_str)?;
        assert_eq!(notation.castle_str, Some("O-O".to_string()));
        assert!(!notation.check);
        assert!(!notation.checkmate);

        let notation_str = "O-O+";
        let notation = Notation::from_str(notation_str)?;
        assert_eq!(notation.castle_str, Some("O-O".to_string()));
        assert!(notation.check);
        assert!(!notation.checkmate);

        let notation_str = "O-O-O#";
        let notation = Notation::from_str(notation_str)?;
        assert_eq!(notation.castle_str, Some("O-O-O".to_string()));
        assert!(!notation.check);
        assert!(notation.checkmate);

        Ok(())
    }

    #[test]
    fn test_notation_from_str_promotion() -> Result<(), PGNParseError> {
        let notation_str = "e8=Q";
        let notation = Notation::from_str(notation_str)?;
        assert_eq!(notation.piece, None);
        assert_eq!(notation.to_file, 'e');
        assert_eq!(notation.to_rank, '8');
        assert_eq!(notation.promotion, Some('Q'));
        assert!(!notation.capture);
        assert!(!notation.check);
        assert!(!notation.checkmate);

        let notation_str = "e8=Q+";
        let notation = Notation::from_str(notation_str)?;
        assert_eq!(notation.piece, None);
        assert_eq!(notation.to_file, 'e');
        assert_eq!(notation.to_rank, '8');
        assert_eq!(notation.promotion, Some('Q'));
        assert!(!notation.capture);
        assert!(notation.check);
        assert!(!notation.checkmate);

        let notation_str = "e8=Q#";
        let notation = Notation::from_str(notation_str)?;
        assert_eq!(notation.piece, None);
        assert_eq!(notation.to_file, 'e');
        assert_eq!(notation.to_rank, '8');
        assert_eq!(notation.promotion, Some('Q'));
        assert!(!notation.capture);
        assert!(!notation.check);
        assert!(notation.checkmate);

        Ok(())
    }

    #[test]
    fn test_notation_from_str_invalid() {
        let notation_str = "Qf9";
        let notation = Notation::from_str(notation_str);
        assert!(notation.is_err());

        let notation_str = "Qz3";
        let notation = Notation::from_str(notation_str);
        assert!(notation.is_err());

        let notation_str = "Qf3x";
        let notation = Notation::from_str(notation_str);
        assert!(notation.is_err());

        let notation_str = "Qf3=";
        let notation = Notation::from_str(notation_str);
        assert!(notation.is_err());

        let notation_str = "Qf3++";
        let notation = Notation::from_str(notation_str);
        assert!(notation.is_err());

        let notation_str = "Qf3##";
        let notation = Notation::from_str(notation_str);
        assert!(notation.is_err());
    }

    #[test]
    fn test_notation_to_move_with_context() {
        let bs = board::BoardState::new_starting();
        let notation = Notation::from_str("Nf3").unwrap();
        let mv = notation.to_move_with_context(&bs);
        assert!(mv.is_ok());
        let mv = mv.unwrap();
        assert_eq!(mv.piece.ptype, PieceType::Knight);
        assert_eq!(mv.from, 62);
        assert_eq!(mv.to, 45);

        let notation = Notation::from_str("e4").unwrap();
        let mv = notation.to_move_with_context(&bs);
        assert!(mv.is_ok());
        let mv = mv.unwrap();
        assert_eq!(mv.piece.ptype, PieceType::Pawn);
        assert_eq!(mv.from, 52);
        assert_eq!(mv.to, 36);
    }

    #[test]
    fn test_index_to_file_notation() {
        assert_eq!(index_to_file_notation(0), 'a');
        assert_eq!(index_to_file_notation(7), 'h');
        assert_eq!(index_to_file_notation(35), 'd');
    }

    #[test]
    fn test_index_to_rank_notation() {
        assert_eq!(index_to_rank_notation_unchecked(0), '8');
        assert_eq!(index_to_rank_notation_unchecked(7), '8');
        assert_eq!(index_to_rank_notation_unchecked(35), '4');
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
}
