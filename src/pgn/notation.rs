use crate::errors::PGNParseError;
use crate::{board, util};
use crate::{hash_to_string, log_and_return_error};
use crate::{movegen::*, BoardState};

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

// CONSTRUCTORS
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
        let legal_moves = match bs_context.get_legal_moves() {
            Ok(moves) => moves,
            Err(e) => {
                let err = PGNParseError::NotationParseError(format!(
                    "Error getting legal moves in BoardState: {}",
                    e
                ));
                log_and_return_error!(err)
            }
        };

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
        notation.piece = match mv.piece.ptype {
            PieceType::Pawn => None,
            PieceType::Knight => Some('N'),
            PieceType::Bishop => Some('B'),
            PieceType::Rook => Some('R'),
            PieceType::Queen => Some('Q'),
            PieceType::King => Some('K'),
        };

        // SET TO FILE AND TO RANK
        notation.to_file = util::index_to_file_notation(mv.to);
        notation.to_rank = util::index_to_rank_notation(mv.to);

        // SET CAPTURE FLAG (Normal capture, en passant capture, or promotion capture)
        notation.capture = mv.move_type.is_capture();

        // SET PROMOTION CHAR
        if let MoveType::Promotion(prom, _) = mv.move_type {
            notation.promotion = Some(match prom {
                PieceType::Queen => 'Q',
                PieceType::Rook => 'R',
                PieceType::Bishop => 'B',
                PieceType::Knight => 'N',
                _ => {
                    // unreachable as move has been legality checked at top of function
                    unreachable!();
                }
            });
        }

        // DISAMBIGUATING MOVES
        // pawn moves that are captures or en passants only need dis_file, otherwise only to_file and to_rank are needed
        if matches!(mv.piece.ptype, PieceType::Pawn) && notation.capture {
            // notation.capture is set above in function
            notation.dis_file = Some(util::index_to_file_notation(mv.from));
        } else {
            // check if there are any other pieces besides pawns that can move to the same square as the mv.piece
            let same_piece_moves: Vec<&Move> = legal_moves
                .iter()
                .filter(|m| m.piece == mv.piece && m.to == mv.to && m.from != mv.from)
                .collect();
            // if there are other pieces that can move to same square
            if !same_piece_moves.is_empty() {
                // store the current mv.from square file and rank
                let mv_from_file = util::index_to_file_notation(mv.from);
                let mv_from_rank = util::index_to_rank_notation(mv.from);
                // keep track of whether any of the other moves have the same file or rank as the current mv.from square
                let mut same_file = false;
                let mut same_rank = false;
                // check if any of the other moves have the same file or rank as the current mv.from square
                for other_mv in same_piece_moves {
                    let other_mv_from_file = util::index_to_file_notation(other_mv.from);
                    let other_mv_from_rank = util::index_to_rank_notation(other_mv.from);
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

    pub fn from_str<'a>(notation_str: &str) -> Result<Notation, PGNParseError> {
        // check that str is valid ascii
        if !notation_str.is_ascii() {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid notation string: ({}) is not valid ascii",
                notation_str
            ));
            log_and_return_error!(err)
        }

        // min length is 2 (e.g. 'e4'), max length is 8 if all disambiguating notation is used and position is a check (e.g. 'Qd5xRd1+')
        let str_len = notation_str.len();
        if str_len < 2 || str_len > 8 {
            let err =
                PGNParseError::NotationParseError(format!("Invalid notation length ({})", str_len));
            log_and_return_error!(err)
        }

        // create new uninitialised Notation struct
        let mut notation = Self::new();

        // Handle castling strings
        // trim check and checkmate chars so that the castle string can be checked in one if statement instead of 3 for each variant
        let possible_castle_str = notation_str.trim_end_matches(&['+', '#']);
        if possible_castle_str == "O-O" || possible_castle_str == "O-O-O" {
            notation.castle_str = Some(possible_castle_str.to_string());
            notation.check = notation_str.ends_with('+');
            notation.checkmate = notation_str.ends_with('#');
            return Ok(notation);
        }

        let mut chars = notation_str.char_indices();
        let mut rank_file_chars: Vec<char> = Vec::new();
        let mut piece_char: Option<char> = None;
        let mut capture = false;
        let mut promotion: Option<char> = None;
        let mut check = false;
        let mut checkmate = false;

        while let Some((i, c)) = chars.next() {
            if c.is_ascii_uppercase() {
                // handle piece char
                if piece_char.is_none() {
                    piece_char = Some(c);
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple uppercase piece chars (char: '{}' at index: {})",
                        notation_str, i
                    ));
                    log_and_return_error!(err)
                }
            } else if c == 'x' {
                // handle captures
                // capture checked first before rank and file as 'x' is ascii lowercase
                if !capture {
                    capture = true;
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple capture chars (char: '{}' at index: {})",
                        notation_str, i
                    ));
                    log_and_return_error!(err)
                }
            } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
                // handle rank and file chars
                rank_file_chars.push(c);
            } else if c == '=' {
                // handle promotions
                if promotion.is_none() {
                    // make sure there is a next char when reading promotion piece and unwrapping, if not, return error
                    let n = chars.next();
                    match n {
                        Some((_, c)) => match c {
                            'Q' | 'R' | 'B' | 'N' => {
                                promotion = Some(c);
                            }
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
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple promotion chars (char: '{}' at index: {})",
                        notation_str, i
                    ));
                    log_and_return_error!(err)
                }
            } else if c == '+' {
                if !check {
                    check = true;
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple check chars (char: '{}' at index: {})",
                        notation_str, i
                    ));
                    log_and_return_error!(err)
                }
                break;
            } else if c == '#' {
                if !checkmate {
                    checkmate = true;
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple checkmate chars (char: '{}' at index: {})",
                        notation_str, i
                    ));
                    log_and_return_error!(err)
                }
                break;
            } else {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid character in notation (char: '{}' at index: {})",
                    c, i
                ));
                log_and_return_error!(err)
            }
        }

        // set piece char if it is valid
        if let Some(piece) = piece_char {
            if notation.is_valid_piece(piece) {
                notation.piece = Some(piece);
            } else {
                let err =
                    PGNParseError::NotationParseError(format!("Invalid piece char ({})", piece));
                log_and_return_error!(err)
            }
        }

        // set rank and file chars, checking if there are any disambiguating chars
        let to_file: char;
        let to_rank: char;
        let mut dis_file = None;
        let mut dis_rank = None;
        if rank_file_chars.len() == 2 {
            to_file = rank_file_chars[0];
            to_rank = rank_file_chars[1];
        } else if rank_file_chars.len() == 3 {
            if rank_file_chars[0].is_ascii_lowercase() {
                dis_file = Some(rank_file_chars[0]); // disambiguating file
            } else {
                dis_rank = Some(rank_file_chars[0]); // disambiguating rank
            }
            to_file = rank_file_chars[1];
            to_rank = rank_file_chars[2];
        } else if rank_file_chars.len() == 4 {
            dis_file = Some(rank_file_chars[0]);
            dis_rank = Some(rank_file_chars[1]);
            to_file = rank_file_chars[2];
            to_rank = rank_file_chars[3];
        } else {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid move notation char(s) ({:?})",
                rank_file_chars
            ));
            log_and_return_error!(err)
        }
        // set rank and file chars if they are all valid
        if !notation.is_valid_file(to_file)
            || !notation.is_valid_rank(to_rank)
            || dis_file.map_or(false, |c| !notation.is_valid_file(c))
            || dis_rank.map_or(false, |c| !notation.is_valid_rank(c))
        {
            let err = PGNParseError::NotationParseError(format!(
                "Invalid rank or file char(s) in vec: ({:?})",
                rank_file_chars
            ));
            log_and_return_error!(err)
        } else {
            notation.to_file = to_file;
            notation.to_rank = to_rank;
            notation.dis_file = dis_file;
            notation.dis_rank = dis_rank;
        }

        // set promotion char if it is valid
        if let Some(promotion) = promotion {
            if notation.is_valid_promotion(promotion) {
                notation.promotion = Some(promotion);
            } else {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid promotion piece char ({})",
                    promotion
                ));
                log_and_return_error!(err)
            }
        }

        // set boolean flags
        notation.capture = capture;
        notation.check = check;
        notation.checkmate = checkmate;

        Ok(notation)
    }
}

impl Notation {
    pub fn to_string(&self) -> String {
        let mut notation = String::new();

        // return castling string if it exists
        if let Some(cs) = &self.castle_str {
            let mut castle_str = cs.clone();
            if self.checkmate {
                castle_str.push('#');
            } else if self.check {
                castle_str.push('+');
            }
            return castle_str;
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
        notation
    }

    // tries to find a move, and disambiguates as best as possible, for use in PGN import format so if it is missing some disambiguating information but the move can still be identified, it is fine
    pub fn to_move(&self, bs: &BoardState) -> Result<Move, PGNParseError> {
        let legal_moves = match bs.get_legal_moves() {
            Ok(moves) => moves,
            Err(e) => {
                let err = PGNParseError::NotationParseError(format!(
                    "Error getting legal moves in BoardState: {}",
                    e
                ));
                log_and_return_error!(err)
            }
        };

        if let Some(castle_str) = &self.castle_str {
            let castle_side = match castle_str.as_str() {
                "O-O" => CastleSide::Short,
                "O-O-O" => CastleSide::Long,
                _ => {
                    let err = PGNParseError::NotationParseError(format!(
                        "Unreachable code: Invalid castle string ({}) in to_move function",
                        &castle_str
                    ));
                    log_and_return_error!(err)
                }
            };
            for mv in legal_moves {
                if let MoveType::Castle(cm) = mv.move_type {
                    if cm.get_castle_side() == castle_side {
                        return Ok(*mv);
                    }
                }
            }
            let err = PGNParseError::MoveNotFound(format!(
                "No legal move found for castle notation ({}) in BoardState (hash: {})",
                &castle_str,
                hash_to_string(bs.board_hash)
            ));
            log_and_return_error!(err)
        } else {
            let possible_moves = legal_moves
                .iter()
                .filter(|mv| {
                    if let Some(piece) = self.piece {
                        if mv.piece.ptype
                            != match piece {
                                'N' => PieceType::Knight,
                                'B' => PieceType::Bishop,
                                'R' => PieceType::Rook,
                                'Q' => PieceType::Queen,
                                'K' => PieceType::King,
                                _ => {
                                    unreachable!("Invalid piece char in to_move function");
                                }
                            }
                        {
                            return false;
                        }
                    } else {
                        // PAWN HANDLING - no piece char can only be a castle move which is already handled, or a pawn move
                        if mv.piece.ptype != PieceType::Pawn {
                            return false;
                        }
                    }

                    if self.to_file != util::index_to_file_notation(mv.to)
                        || self.to_rank != util::index_to_rank_notation(mv.to)
                    {
                        return false;
                    }
                    if self.capture && !mv.move_type.is_capture() {
                        return false;
                    }
                    if let Some(promotion) = self.promotion {
                        if let MoveType::Promotion(promotion_ptype, _) = mv.move_type {
                            if promotion_ptype
                                != match promotion {
                                    'Q' => PieceType::Queen,
                                    'R' => PieceType::Rook,
                                    'B' => PieceType::Bishop,
                                    'N' => PieceType::Knight,
                                    _ => unreachable!("Invalid promotion char in to_move function"),
                                }
                            {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    // if move passes all checks, return true
                    true
                })
                .collect::<Vec<&Move>>();
            if possible_moves.len() == 1 {
                return Ok(*possible_moves[0]);
            } else if possible_moves.len() > 1 {
                let mut dis_file_possible_idxs = None;
                let mut dis_rank_possible_idxs = None;

                if let Some(dis_file_char) = self.dis_file {
                    dis_file_possible_idxs =
                        Some(util::file_notation_to_indexes_unchecked(dis_file_char));
                }
                if let Some(dis_rank_char) = self.dis_rank {
                    dis_rank_possible_idxs =
                        Some(util::rank_notation_to_indexes_unchecked(dis_rank_char));
                }

                let mut possible_dis_moves = Vec::new();
                for mv in &possible_moves {
                    if dis_file_possible_idxs.is_some() && dis_rank_possible_idxs.is_some() {
                        if dis_file_possible_idxs.unwrap().contains(&mv.from)
                            && dis_rank_possible_idxs.unwrap().contains(&mv.from)
                        {
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
                        self.to_string(),
                        hash_to_string(bs.board_hash),
                        possible_moves
                    ));
                    log_and_return_error!(err)
                }
            } else {
                let err = PGNParseError::MoveNotFound(format!(
                    "No legal move found for notation ({}) in BoardState (hash: {})",
                    self.to_string(),
                    hash_to_string(bs.board_hash)
                ));
                log_and_return_error!(err)
            }
        }
    }

    #[inline]
    fn is_valid_file(&self, file: char) -> bool {
        file.is_ascii_lowercase() && file >= 'a' && file <= 'h'
    }

    #[inline]
    fn is_valid_rank(&self, rank: char) -> bool {
        rank.is_ascii_digit() && rank >= '1' && rank <= '8'
    }

    #[inline]
    fn is_valid_piece(&self, piece: char) -> bool {
        let valid_pieces = ['P', 'N', 'B', 'R', 'Q', 'K'];
        piece.is_ascii_uppercase() && valid_pieces.contains(&piece)
    }

    #[inline]
    fn is_valid_promotion(&self, promotion: char) -> bool {
        let valid_promotions = ['Q', 'R', 'B', 'N'];
        promotion.is_ascii_uppercase() && valid_promotions.contains(&promotion)
    }
}

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
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
