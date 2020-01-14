use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

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
                if let Some(_) = optbegin.replace(pos) {
                    return Err(Error::DuplicateElement("begin", "location"));
                }
            },
            e @ b"end" => {
                let pos = Position::from_xml(&e, reader, buffer)?;
                if let Some(_) = optend.replace(pos) {
                    return Err(Error::DuplicateElement("end", "location"));
                }
            },
            e @ b"position" => {
                let pos = Position::from_xml(&e, reader, buffer)?;
                if let Some(_) = optposition.replace(pos) {
                    return Err(Error::DuplicateElement("position", "location"));
                }
            }
        }

        if let Some(pos) = optposition {
            if optbegin.is_some() || optend.is_some() {
                panic!("ERR: cannot have both `begin` or `end` with `position`");
            }
            Ok(FeatureLocation::Position(pos))
        } else {
            let begin = optbegin.ok_or(Error::MissingElement("begin", "location"))?;
            let end = optend.ok_or(Error::MissingElement("end", "location"))?;
            Ok(FeatureLocation::Range(begin, end))
        }
    }
}

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
        let status = match attr.get(&b"status"[..]).map(|a| &*a.value) {
            Some(b"certain") => Status::Certain,
            Some(b"uncertain") => Status::Uncertain,
            Some(b"less than") => Status::Certain,
            Some(b"greater than") => Status::Certain,
            Some(b"unknown") => Status::Certain,
            Some(other) => panic!("ERR: invalid `status` for `position`: {:?}", other),
            None => Status::default(),
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
