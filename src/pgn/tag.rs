use std::fmt;

use crate::{errors::PGNParseError, log_and_return_error};

#[derive(Debug, Clone)]
pub struct CustomTag {
    name: String,
    value: String,
}

#[derive(Debug, PartialEq, Ord, Eq, PartialOrd, Clone)]
pub enum Tag {
    Event(String),
    Site(String),
    Date(String),
    Round(String),
    White(String),
    Black(String),
    Result(String),
    CustomTag { name: String, value: String },
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

pub fn parse_tag(tag: &str) -> Result<Tag, PGNParseError> {
    let tag_str = tag.trim_matches(&['[', ']']).trim();
    let mut parts = tag_str.splitn(2, ' ').map(str::trim);

    let name = match parts.next() {
        Some(name) => name,
        None => {
            let err = PGNParseError::InvalidTag(format!("Tag {} has invalid name", tag));
            log_and_return_error!(err)
        }
    };
    let value = match parts.next() {
        Some(value) => value.trim_matches('"'),
        None => {
            let err = PGNParseError::InvalidTag(format!("Tag {} has invalid value", tag));
            log_and_return_error!(err)
        }
    };
    match name {
        "Event" => Ok(Tag::Event(value.to_string())),
        "Site" => Ok(Tag::Site(value.to_string())),
        "Date" => Ok(Tag::Date(value.to_string())),
        "Round" => Ok(Tag::Round(value.to_string())),
        "White" => Ok(Tag::White(value.to_string())),
        "Black" => Ok(Tag::Black(value.to_string())),
        "Result" => Ok(Tag::Result(value.to_string())),
        c => Ok(Tag::CustomTag {
            name: c.to_string(),
            value: value.to_string(),
        }),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_tag() {
        let tag_str = "[Event    \"Test Game\"]"; // multiple spaces between tag and value should be parsed correctly
        let tag = parse_tag(tag_str).unwrap();

        match tag {
            Tag::Event(value) => assert_eq!(value, "Test Game"),
            _ => panic!("Parsed tag is not an Event"),
        }
    }
}
