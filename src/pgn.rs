// Implementing standard from <https://ia902908.us.archive.org/26/items/pgn-standard-1994-03-12/PGN_standard_1994-03-12.txt>
use std::fmt;

use crate::board;
use crate::errors::PGNParseError;

#[derive(Debug)]
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
        c => {
            Ok(Tag::CustomTag {
                name: c.to_string(),
                value: value.to_string(),
            })
        },
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
}
