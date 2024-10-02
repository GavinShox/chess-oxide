// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
use std::fmt;

use crate::board;
use crate::errors::PGNParseError;
use crate::movegen::*;

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

    pub fn to_move(&self) -> Result<Move, PGNParseError> {
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

enum Tag {
    Event(String),
    Site(String),
    Date(String),
    Round(String),
    White(String),
    Black(String),
    Result(String),
    CustomTag { name: String, value: String },
}

struct CustomTag {
    name: String,
    value: String,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Tag::Event(value) => write!(f, "[Event \"{}\"]", value),
            Tag::Site(value) => write!(f, "[Site \"{}\"]", value),
            Tag::Date(value) => write!(f, "[Date \"{}\"]", value),
            Tag::Round(value) => write!(f, "[Round \"{}\"]", value),
            Tag::White(value) => write!(f, "[White \"{}\"]", value),
            Tag::Black(value) => write!(f, "[Black \"{}\"]", value),
            Tag::Result(value) => write!(f, "[Result \"{}\"]", value),
            Tag::CustomTag { name, value } => write!(f, "[{} \"{}\"]", name, value),
        }
    }
}

fn is_pgn_delimiter(prev_char: char, c: char) -> bool {
    assert!(c.is_ascii());
    c.is_ascii_whitespace()
        || c == '.'
        || c == ')'
        || c == '('
        || c == '{'
        || c == '}'
        || c == '['
        || c == ']'
        || c == '*'
        || c == '<'
        || c == '>'
        || c == '"'
        || prev_char.is_ascii_digit() && !c.is_ascii_digit()
}

fn tokenize(pgn: &str) -> Vec<&str> {
    if !pgn.is_ascii() {
        panic!("PGN must be ASCII");
    }
    let mut split_vec = Vec::new();
    let mut last = 0;
    for (index, matched) in
        pgn.match_indices(move |c: char| is_pgn_delimiter(pgn.chars().nth(last).unwrap(), c))
    {
        if last != index {
            split_vec.push(&pgn[last..index]);
        }
        split_vec.push(matched);
        last = index + matched.len();
    }
    if last < pgn.len() {
        split_vec.push(&pgn[last..]);
    }
    split_vec
}

fn parse_tag(tag: &str) -> Result<Tag, PGNParseError> {
    let tag_str = tag.trim_matches(&['[', ']']).trim();
    let parts = tag_str.split_once(' ').unwrap(); // TODO handle error, dont just unwrap
    let name = parts.0.trim();
    let value = parts.1.trim().trim_matches('"');
    match name {
        "Event" => Ok(Tag::Event(value.to_string())),
        "Site" => Ok(Tag::Site(value.to_string())),
        "Date" => Ok(Tag::Date(value.to_string())),
        "Round" => Ok(Tag::Round(value.to_string())),
        "White" => Ok(Tag::White(value.to_string())),
        "Black" => Ok(Tag::Black(value.to_string())),
        "Result" => Ok(Tag::Result(value.to_string())),
        _ => todo!("Handle custom tags"),
    }
}

fn tokens_read_tags(tokens: &Vec<&str>) -> Result<Vec<Tag>, PGNParseError> {
    let mut tags = Vec::new();
    let mut tag_str = String::new();
    let mut in_tag = false;
    for token in tokens {
        if *token == "[" {
            in_tag = true;
            tag_str += token;
        } else if *token == "]" {
            in_tag = false;
            tag_str += token;
            tags.push(parse_tag(&tag_str)?);
            tag_str.clear();
        } else if in_tag {
            tag_str += token;
        }
    }
    Ok(tags)
}

struct PGN {
    pgn_string: String,
    tokens: Vec<String>,
    tags: Vec<Tag>,
    moves: Vec<String>,
}
impl PGN {
    fn new(pgn: &str) -> Self {
        let mut new = Self {
            pgn_string: pgn.to_string(),
            tokens: Vec::new(),
            tags: Vec::new(),
            moves: Vec::new(),
        };

        new
    }

    fn from_board(board: &board::Board) -> Self {
        let mut new = Self {
            pgn_string: String::new(),
            tokens: Vec::new(),
            tags: Vec::new(),
            moves: Vec::new(),
        };

        new
    }

    fn to_board(&self) -> board::Board {
        board::Board::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let pgn_string = "[Event \"Token Test Game\"]\n1. e5 e6";
        let tokens = tokenize(pgn_string);

        let expected_tokens = vec![
            "[".to_string(),
            "Event".to_string(),
            " ".to_string(),
            "\"".to_string(),
            "Token".to_string(),
            " ".to_string(),
            "Test".to_string(),
            " ".to_string(),
            "Game".to_string(),
            "\"".to_string(),
            "]".to_string(),
            "\n".to_string(),
            "1".to_string(),
            ".".to_string(),
            " ".to_string(),
            "e5".to_string(),
            " ".to_string(),
            "e6".to_string(),
        ];

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_with_comments() {
        let pgn_string = "[Event \"Game\"] {This is a comment} 1.e4 e5";
        let tokens = tokenize(pgn_string);

        let expected_tokens = vec![
            "[".to_string(),
            "Event".to_string(),
            " ".to_string(),
            "\"".to_string(),
            "Game".to_string(),
            "\"".to_string(),
            "]".to_string(),
            " ".to_string(),
            "{".to_string(),
            "This".to_string(),
            " ".to_string(),
            "is".to_string(),
            " ".to_string(),
            "a".to_string(),
            " ".to_string(),
            "comment".to_string(),
            "}".to_string(),
            " ".to_string(),
            "1".to_string(),
            ".".to_string(),
            "e4".to_string(),
            " ".to_string(),
            "e5".to_string(),
        ];

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_with_variations() {
        let pgn_string = "[Event \"Game\"] 1.e4 (1. d4 d5) e5";
        let tokens = tokenize(pgn_string);

        let expected_tokens = vec![
            "[".to_string(),
            "Event".to_string(),
            " ".to_string(),
            "\"".to_string(),
            "Game".to_string(),
            "\"".to_string(),
            "]".to_string(),
            " ".to_string(),
            "1".to_string(),
            ".".to_string(),
            "e4".to_string(),
            " ".to_string(),
            "(".to_string(),
            "1".to_string(),
            ".".to_string(),
            " ".to_string(),
            "d4".to_string(),
            " ".to_string(),
            "d5".to_string(),
            ")".to_string(),
            " ".to_string(),
            "e5".to_string(),
        ];

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_parse_tag() {
        let tag_str = "[Event \"Test Game\"]";
        let tag = parse_tag(tag_str).unwrap();

        match tag {
            Tag::Event(value) => assert_eq!(value, "Test Game"),
            _ => panic!("Parsed tag is not an Event"),
        }
    }

    #[test]
    fn test_tokens_read_tags() {
        let tokens = vec![
            "[", "Event", " ", "\"", "Token", " ", "Test", " ", "Game", "\"", "]",
        ];
        let tags = tokens_read_tags(&tokens).unwrap();

        assert_eq!(tags.len(), 1);
        match &tags[0] {
            Tag::Event(value) => assert_eq!(value, "Token Test Game"),
            _ => panic!("Parsed tag is not an Event"),
        }
    }

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
    fn test_notation_from_str() {
        let bs = board::BoardState::new_starting();
        let notation_str = "Qf3xf5+";
        let notation = Notation::from_str(&bs, notation_str);

        assert!(notation.is_ok());
        let notation = notation.unwrap();
        assert_eq!(notation.piece, Some('Q'));
        assert_eq!(notation.to_file, 'f');
        assert_eq!(notation.to_rank, '5');
        assert_eq!(notation.capture, true);
        assert_eq!(notation.check, true);
        assert_eq!(notation.dis_file.is_some(), true);
        assert_eq!(notation.dis_rank.is_some(), true);
        assert_eq!(notation.dis_file.unwrap(), 'f');
        assert_eq!(notation.dis_rank.unwrap(), '3');
    }
}
