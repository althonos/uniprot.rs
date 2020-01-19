use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;
use crate::parser::utils::decode_attribute;

#[derive(Debug, Clone)]
pub enum FeatureLocation {
    Range(Position, Position),
    Position(Position)
}

impl FromXml for FeatureLocation {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"location");

        let mut optbegin: Option<Position> = None;
        let mut optend: Option<Position> = None;
        let mut optposition: Option<Position> = None;

        parse_inner!{event, reader, buffer,
            e @ b"begin" => {
                let pos = Position::from_xml(&e, reader, buffer)?;
                if optbegin.replace(pos).is_some() {
                    return Err(Error::DuplicateElement("begin", "location"));
                }
            },
            e @ b"end" => {
                let pos = Position::from_xml(&e, reader, buffer)?;
                if optend.replace(pos).is_some() {
                    return Err(Error::DuplicateElement("end", "location"));
                }
            },
            e @ b"position" => {
                let pos = Position::from_xml(&e, reader, buffer)?;
                if optposition.replace(pos).is_some() {
                    return Err(Error::DuplicateElement("position", "location"));
                }
            }
        }

        if let Some(pos) = optposition {
            if optbegin.is_some() {
                Err(Error::DuplicateElement("begin", "location"))
            } else if optend.is_some() {
                Err(Error::DuplicateElement("end", "location"))
            } else {
                Ok(FeatureLocation::Position(pos))
            }
        } else {
            let begin = optbegin.ok_or(Error::MissingElement("begin", "location"))?;
            let end = optend.ok_or(Error::MissingElement("end", "location"))?;
            Ok(FeatureLocation::Range(begin, end))
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Position {
    pub pos: Option<usize>,
    pub status: Status,
    pub evidence: Vec<usize>,
}

impl FromXml for Position {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert!(
            event.local_name() == b"begin"
            || event.local_name() == b"end"
            || event.local_name() == b"position"
        );

        let attr = attributes_to_hashmap(event)?;
        let status = match decode_attribute(event, reader, "status", "position") {
            Ok(status) => status,
            Err(Error::MissingAttribute(_, _)) => Status::default(),
            Err(other) => return Err(other),
        };
        let evidence = get_evidences(reader, &attr)?;
        let pos = attr.get(&b"position"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .transpose()?;

        reader.read_to_end(event.local_name(), buffer)?;
        Ok(Position { pos, status, evidence })
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Status {
    Certain,
    Uncertain,
    LessThan,
    GreaterThan,
    Unknown,
}

impl Default for Status {
    fn default() -> Self {
        Status::Certain
    }
}

impl FromStr for Status {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "certain" => Ok(Status::Certain),
            "uncertain" => Ok(Status::Uncertain),
            "less than" => Ok(Status::LessThan),
            "greater than" => Ok(Status::GreaterThan),
            "unknown" => Ok(Status::Unknown),
            other => Err(InvalidValue::from(other))
        }
    }
}
