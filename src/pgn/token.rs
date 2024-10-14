use std::ops::Deref;
use std::vec;

use super::notation::*;
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
    fn is_game_termination_marker(&self) -> bool {
        self.value == "1-0" || self.value == "0-1" || self.value == "1/2-1/2" || self.value == "*"
    }
}

#[derive(Debug)]
pub struct Tokens {
    tokens: Vec<Token>,
}
impl Tokens {
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
                tags.push(Tag::from_str(&tag_str)?);
                tag_str.clear();
            } else if in_tag {
                tag_str += &token.value;
            }
        }
        Ok(tags)
    }

    pub fn get_game_termination(&self) -> Option<String> {
        for token in self.tokens.iter() {
            if token.is_game_termination_marker() {
                return Some(token.value.clone());
            }
        }
        None
    }

    pub fn get_move_notations(&self) -> Result<Vec<Notation>, PGNParseError> {
        // for now trim comments, variations and move numbers from the movetext as we won't use them for now
        let mut move_tokens = self.tokens.clone();
        let delimiters = vec![("(", ")"), ("{", "}"), ("[", "]"), ("<", ">")];
        for delimiter in delimiters {
            let mut new_tokens = Vec::new();
            let mut in_delimiter = false;
            for token in move_tokens {
                if token.value == delimiter.0 {
                    in_delimiter = true;
                } else if token.value == delimiter.1 {
                    in_delimiter = false;
                } else if !in_delimiter {
                    new_tokens.push(token.clone());
                }
            }
            move_tokens = new_tokens;
        }
        // truncate at game termination marker
        if let Some(pos) = move_tokens
            .iter()
            .position(|token| token.is_game_termination_marker())
        {
            move_tokens.truncate(pos);
        }
        // remove all single character tokens that are left
        move_tokens.retain(|token| token.value.len() > 1);
        // remove move numbers
        move_tokens.retain(|token| !token.value.chars().all(|c| c.is_ascii_digit()));
        let mut notations = Vec::new();
        for token in move_tokens {
            let notation = Notation::from_str(&token.value)?;
            notations.push(notation);
        }

        Ok(notations)
    }
}
// calling .iter() on Tokens will iterator over the inner Vec
impl Deref for Tokens {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
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
        let tokens = Tokens { tokens: tokens_vec };
        let tags = tokens.get_tags().unwrap();

        assert_eq!(tags.len(), 1);
        match &tags[0] {
            Tag::Event(value) => assert_eq!(value, "Token Test Game"),
            _ => panic!("Parsed tag is not an Event"),
        }
    }

    #[test]
    fn test_tokens_get_move_notations() {
        let tokens_vec = vec![
            Token::new("1"),
            Token::new("."),
            Token::new(" "),
            Token::new("e4"),
            Token::new(" "),
            Token::new("e5"),
            Token::new(" "),
            Token::new("Q1d7+"),
            Token::new("1-0"),
        ];
        let tokens = Tokens { tokens: tokens_vec };
        let notations = tokens.get_move_notations().unwrap();

        assert_eq!(notations.len(), 3);
        assert_eq!(notations[0], Notation::from_str("e4").unwrap());
        assert_eq!(notations[1], Notation::from_str("e5").unwrap());
        assert_eq!(notations[2], Notation::from_str("Q1d7+").unwrap());
        println!("{:?}", notations[2]);
    }
}
