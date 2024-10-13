use std::fmt;

use crate::{errors::PGNParseError, log_and_return_error};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct CustomTag {
    name: String,
    value: String,
}
impl CustomTag {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
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
    CustomTag(CustomTag), // TODO add more variants instead of just custom tags, required tags in PGN standard is handled in PGN struct not here
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
            Tag::CustomTag(ct) => write!(f, "[{} \"{}\"]", ct.name, ct.value),
        }
    }
}

impl Tag {
    pub fn from_str(tag: &str) -> Result<Tag, PGNParseError> {
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
            c => Ok(Tag::CustomTag(CustomTag::new(c, value))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_tag() {
        let tag_str = "[Event    \"Test Game\"]"; // multiple spaces between tag and value should be parsed correctly
        let tag = Tag::from_str(tag_str).unwrap();

        match tag {
            Tag::Event(value) => assert_eq!(value, "Test Game"),
            _ => panic!("Parsed tag is not an Event"),
        }
    }
}
