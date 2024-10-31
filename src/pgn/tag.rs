use std::{fmt, str::FromStr};

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
    // REQUIRED TAGS
    Event(String),
    Site(String),
    Date(String),
    Round(String),
    White(String),
    Black(String),
    Result(String),
    // OPTIONAL TAGS
    Eco(String),
    SetUp(String),
    FEN(String),
    Termination(String),
    Annotator(String),
    CustomTag(CustomTag),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(value) => write!(f, "[Event \"{}\"]", value),
            Self::Site(value) => write!(f, "[Site \"{}\"]", value),
            Self::Date(value) => write!(f, "[Date \"{}\"]", value),
            Self::Round(value) => write!(f, "[Round \"{}\"]", value),
            Self::White(value) => write!(f, "[White \"{}\"]", value),
            Self::Black(value) => write!(f, "[Black \"{}\"]", value),
            Self::Result(value) => write!(f, "[Result \"{}\"]", value),
            Self::Eco(value) => write!(f, "[ECO \"{}\"]", value),
            Self::SetUp(value) => write!(f, "[SetUp \"{}\"]", value),
            Self::FEN(value) => write!(f, "[FEN \"{}\"]", value),
            Self::Termination(value) => write!(f, "[Termination \"{}\"]", value),
            Self::Annotator(value) => write!(f, "[Annotator \"{}\"]", value),
            Self::CustomTag(ct) => write!(f, "[{} \"{}\"]", ct.name, ct.value),
        }
    }
}

impl FromStr for Tag {
    type Err = PGNParseError;

    fn from_str(tag: &str) -> Result<Tag, PGNParseError> {
        let tag_str = tag.trim_matches(['[', ']']).trim();
        let mut parts = tag_str.splitn(2, ' ').map(str::trim);

        let name = if let Some(name) = parts.next() {
            name
        } else {
            let err = PGNParseError::InvalidTag(format!("Tag {} has invalid name", tag));
            log_and_return_error!(err)
        };

        let value = if let Some(value) = parts.next() {
            value.trim_matches('"')
        } else {
            let err = PGNParseError::InvalidTag(format!("Tag {} has invalid value", tag));
            log_and_return_error!(err)
        };

        match name {
            "Event" => Ok(Self::Event(value.to_string())),
            "Site" => Ok(Self::Site(value.to_string())),
            "Date" => Ok(Self::Date(value.to_string())),
            "Round" => Ok(Self::Round(value.to_string())),
            "White" => Ok(Self::White(value.to_string())),
            "Black" => Ok(Self::Black(value.to_string())),
            "Result" => Ok(Self::Result(value.to_string())),
            "ECO" => Ok(Self::Eco(value.to_string())),
            "SetUp" => Ok(Self::SetUp(value.to_string())),
            "FEN" => Ok(Self::FEN(value.to_string())),
            "Termination" => Ok(Self::Termination(value.to_string())),
            "Annotator" => Ok(Self::Annotator(value.to_string())),
            c => Ok(Self::CustomTag(CustomTag::new(c, value))),
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
