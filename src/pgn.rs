use std::{error::Error, fmt::Display};

use crate::errors::PGNParseError;

enum Tag {
    Event(String),
    Site(String),
    Date(String),
    Round(String),
    White(String),
    Black(String),
    Result(String),
}
impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Tag::Event(value) => write!(f, "[Event \"{}\"]", value),
            Tag::Site(value) => write!(f, "[Site \"{}\"]", value),
            Tag::Date(value) => write!(f, "[Date \"{}\"]", value),
            Tag::Round(value) => write!(f, "[Round \"{}\"]", value),
            Tag::White(value) => write!(f, "[White \"{}\"]", value),
            Tag::Black(value) => write!(f, "[Black \"{}\"]", value),
            Tag::Result(value) => write!(f, "[Result \"{}\"]", value),
        }
    }
}

fn parse_tag(tag: &str) -> Result<Tag, PGNParseError> {
    let tag_str = tag.trim_matches(&['[', ']']).trim();
    let mut parts = tag_str.split(' ');
    let name = parts.next().unwrap();
    let value = parts.next().unwrap().trim_matches('"');
    match name {
        "Event" => Ok(Tag::Event(value.to_string())),
        "Site" => Ok(Tag::Site(value.to_string())),
        "Date" => Ok(Tag::Date(value.to_string())),
        "Round" => Ok(Tag::Round(value.to_string())),
        "White" => Ok(Tag::White(value.to_string())),
        "Black" => Ok(Tag::Black(value.to_string())),
        "Result" => Ok(Tag::Result(value.to_string())),
        _ => panic!("Unknown tag: {}", name),
    }
}

struct PGN {
    tags: Vec<Tag>,
    moves: Vec<String>,
}
