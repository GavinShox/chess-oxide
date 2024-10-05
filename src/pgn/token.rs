use std::{fmt, ops::Deref};

use super::tag::*;
use crate::errors::PGNParseError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub value: String,
}
impl Token {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Tokens {
    tokens: Vec<Token>,
}
impl Tokens {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn from_vec(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    pub fn from_pgn_str(pgn: &str) -> Self {
        Self {
            tokens: tokenize(pgn),
        }
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>, PGNParseError> {
        let mut tags = Vec::new();
        let mut tag_str = String::new();
        let mut in_tag = false;
        for token in self.tokens.iter() {
            if token.value == "[" {
                in_tag = true;
                tag_str += &token.value;
            } else if token.value == "]" {
                in_tag = false;
                tag_str += &token.value;
                tags.push(parse_tag(&tag_str)?);
                tag_str.clear();
            } else if in_tag {
                tag_str += &token.value;
            }
        }
        Ok(tags)
    }
}
impl Deref for Tokens {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

pub enum TerminationMarker {
    WhiteWins,
    BlackWins,
    Draw,
    InProgress,
}
impl fmt::Display for TerminationMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TerminationMarker::WhiteWins => write!(f, "1-0"),
            TerminationMarker::BlackWins => write!(f, "0-1"),
            TerminationMarker::Draw => write!(f, "1/2-1/2"),
            TerminationMarker::InProgress => write!(f, "*"),
        }
    }
}

fn tokenize(pgn: &str) -> Vec<Token> {
    if !pgn.is_ascii() {
        panic!("PGN must be ASCII");
    }
    let mut split_vec = Vec::new();
    let mut last = 0;
    for (index, matched) in
        pgn.match_indices(move |c: char| is_pgn_delimiter(pgn.chars().nth(last).unwrap(), c))
    {
        if last != index {
            split_vec.push(Token::new(&pgn[last..index]));
        }
        split_vec.push(Token::new(matched));
        last = index + matched.len();
    }
    if last < pgn.len() {
        split_vec.push(Token::new(&pgn[last..]));
    }
    split_vec
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize() {
        let pgn_string = "[Event \"Token Test Game\"]\n1. e5 e6";
        let tokens = tokenize(pgn_string);

        let expected_tokens_str = vec![
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

        let expected_tokens: Vec<Token> =
            expected_tokens_str.iter().map(|s| Token::new(s)).collect();

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_with_comments() {
        let pgn_string = "[Event \"Game\"] {This is a comment} 1.e4 e5";
        let tokens = tokenize(pgn_string);

        let expected_tokens_str = vec![
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

        let expected_tokens: Vec<Token> =
            expected_tokens_str.iter().map(|s| Token::new(s)).collect();

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_with_variations() {
        let pgn_string = "[Event \"Game\"] 1.e4 (1. d4 d5) e5";
        let tokens = tokenize(pgn_string);

        let expected_tokens_str = vec![
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

        let expected_tokens: Vec<Token> =
            expected_tokens_str.iter().map(|s| Token::new(s)).collect();

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_tokens_get_tags() {
        let tokens_vec = vec![
            Token::new("["),
            Token::new("Event"),
            Token::new(" "),
            Token::new("\""),
            Token::new("Token"),
            Token::new(" "),
            Token::new("Test"),
            Token::new(" "),
            Token::new("Game"),
            Token::new("\""),
            Token::new("]"),
        ];
        let tokens = Tokens::from_vec(tokens_vec);
        let tags = tokens.get_tags().unwrap();

        assert_eq!(tags.len(), 1);
        match &tags[0] {
            Tag::Event(value) => assert_eq!(value, "Token Test Game"),
            _ => panic!("Parsed tag is not an Event"),
        }
    }
}
