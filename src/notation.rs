use crate::board;
use crate::errors::PGNParseError;
use crate::movegen::Move;

pub struct Notation<'a> {
    bs: &'a board::BoardState,
    piece: Option<char>,
    dis_file: Option<char>, // for disambiguating moves if required
    dis_rank: Option<char>, // for disambiguating moves if required
    capture: bool,
    to_file: char,
    to_rank: char,
    promotion: Option<char>,
    check: bool,
    checkmate: bool,
}

impl Notation<'_> {
    // (private) new uninitialised Notation struct
    fn new<'a>(bs: &'a board::BoardState) -> Notation<'a> {
        Notation {
            bs,
            piece: None,
            dis_file: None,
            dis_rank: None,
            capture: false,
            to_file: ' ',
            to_rank: ' ',
            promotion: None,
            check: false,
            checkmate: false,
        }
    }

    pub fn to_string(&self) -> String {
        let mut notation = String::new();
        if let Some(piece) = self.piece {
            notation.push(piece);
        }
        if let Some(dis_rank) = self.dis_rank {
            notation.push(dis_rank);
        }
        if let Some(dis_file) = self.dis_file {
            notation.push(dis_file);
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

    pub fn to_move(&self) -> Result<&Move, PGNParseError> {
        todo!()
    }

    pub fn from_mv<'a>(
        bs: &'a board::BoardState,
        mv: &'a Move,
    ) -> Result<Notation<'a>, PGNParseError> {
        let legal_moves = match bs.get_legal_moves() {
            Ok(moves) => moves,
            Err(e) => {
                let err = PGNParseError::NotationParseError(format!(
                    "Error getting legal moves BoardState->: {}",
                    e
                ));
                log::error!("{}", err.to_string());
                return Err(err);
            }
        };
        if !legal_moves.contains(mv) {
            let err = PGNParseError::NotationParseError(format!("Move not legal: {:?}", mv));
            log::error!("{}", err.to_string());
            return Err(err);
        }
        // TODO rest
        Ok(Notation::new(bs))
    }

    // from move but doesnt need boardstate so it will always use all disambiguation
    pub fn from_mv_full_disambiguation(
        mv: &Move,
    ) -> Result<Notation, PGNParseError> {
        todo!()
    }

    pub fn from_str<'a>(
        bs: &'a board::BoardState,
        notation_str: &str,
    ) -> Result<Notation<'a>, PGNParseError> {
        // min length is 2 (e.g. 'e4'), max length is 8 if all disambiguating notation is used and position is a check (e.g. 'Qd5xRd1+')
        let str_len = notation_str.len();
        if str_len < 2 || str_len > 8 {
            let err =
                PGNParseError::NotationParseError(format!("Invalid notation length ({})", str_len));
            log::error!("{}", err.to_string());
            return Err(err);
        }

        let mut chars = notation_str.chars();
        let mut rank_file_chars: Vec<char> = Vec::new();
        let mut piece_char: Option<char> = None;
        let mut capture = false;
        let mut promotion: Option<char> = None;
        let mut check = false;
        let mut checkmate = false;

        while let Some(c) = chars.next() {
            if c.is_ascii_uppercase() {
                // handle piece char
                if piece_char.is_none() {
                    piece_char = Some(c);
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple uppercase piece chars ({})",
                        notation_str
                    ));
                    log::error!("{}", err.to_string());
                    return Err(err);
                }
            } else if c == 'x' {
                // handle captures
                // capture checked first before rank and file as 'x' is ascii lowercase
                if !capture {
                    capture = true;
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple capture chars ({})",
                        notation_str
                    ));
                    log::error!("{}", err.to_string());
                    return Err(err);
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
                        Some(c) => match c {
                            'Q' | 'R' | 'B' | 'N' => {
                                promotion = Some(c);
                            }
                            _ => {
                                let err = PGNParseError::NotationParseError(format!(
                                    "Invalid promotion piece ({})",
                                    c
                                ));
                                log::error!("{}", err.to_string());
                                return Err(err);
                            }
                        },
                        None => {
                            let err = PGNParseError::NotationParseError(format!(
                                "Invalid notation, no promotion piece after '=' ({})",
                                notation_str
                            ));
                            log::error!("{}", err.to_string());
                            return Err(err);
                        }
                    }
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple promotion chars ({})",
                        notation_str
                    ));
                    log::error!("{}", err.to_string());
                    return Err(err);
                }
            } else if c == '+' {
                if !check {
                    check = true;
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple check chars ({})",
                        notation_str
                    ));
                    log::error!("{}", err.to_string());
                    return Err(err);
                }
                break;
            } else if c == '#' {
                if !checkmate {
                    checkmate = true;
                } else {
                    let err = PGNParseError::NotationParseError(format!(
                        "Invalid notation, multiple checkmate chars ({})",
                        notation_str
                    ));
                    log::error!("{}", err.to_string());
                    return Err(err);
                }
                break;
            } else {
                let err = PGNParseError::NotationParseError(format!(
                    "Invalid character in notation ({})",
                    c
                ));
                log::error!("{}", err.to_string());
                return Err(err);
            }
        }

        // create new uninitialised Notation struct
        let mut notation = Self::new(bs);

        // set piece char if it is valid
        if let Some(piece) = piece_char {
            if notation.is_valid_piece(piece) {
                notation.piece = Some(piece);
            } else {
                let err =
                    PGNParseError::NotationParseError(format!("Invalid piece char ({})", piece));
                log::error!("{}", err.to_string());
                return Err(err);
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
                "Invalid move notation ({})",
                notation_str
            ));
            log::error!("{}", err.to_string());
            return Err(err);
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
            log::error!("{}", err.to_string());
            return Err(err);
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
                log::error!("{}", err.to_string());
                return Err(err);
            }
        }

        // set boolean flags
        notation.capture = capture;
        notation.check = check;
        notation.checkmate = checkmate;

        Ok(notation)
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
        let bs = board::BoardState::new_starting();
        let notation = Notation::new(&bs);

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
        let bs = board::BoardState::new_starting();
        let mut notation = Notation::new(&bs);
        notation.piece = Some('N');
        notation.to_file = 'f';
        notation.to_rank = '3';
        notation.capture = true;

        assert_eq!(notation.to_string(), "Nxf3");
    }

    // TODO: implement from_move test
    // #[test]
    // fn test_notation_from_mv() {
    //     let bs = board::BoardState::new();
    //     let mv = Move::new(0, 0, 0, 0); // Replace with a valid move
    //     let notation = Notation::from_mv(&bs, &mv);

    //     assert!(notation.is_ok());
    // }

    #[test]
    fn test_notation_from_str() -> Result<(), PGNParseError> {
        let bs = board::BoardState::new_starting();
        let notation_str = "Qf3xf5+";
        let notation = Notation::from_str(&bs, notation_str);
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
