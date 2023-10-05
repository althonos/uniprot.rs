use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::super::db_reference::DbReference;

#[derive(Debug, Clone)]
pub struct CatalyticActivity {
    pub reaction: Reaction,
    pub physiological_reactions: Vec<PhysiologicalReaction>,
}

impl CatalyticActivity {
    pub fn new(reaction: Reaction) -> Self {
        Self {
            reaction,
            physiological_reactions: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Reaction {
    pub text: ShortString,
    pub db_references: Vec<DbReference>,
    pub evidences: Vec<usize>,
}

impl Reaction {
    pub fn new(text: ShortString) -> Self {
        Self {
            text,
            db_references: Default::default(),
            evidences: Default::default(),
        }
    }
}

impl FromXml for Reaction {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"reaction");

        let mut db_references = Vec::new();
        let mut opttext = None;

        parse_inner! {event, reader, buffer,
            e @ b"text" => {
                let text = parse_text!(e, reader, buffer);
                if opttext.replace(text).is_some() {
                    return Err(Error::DuplicateElement("text", "reaction"));
                }
            },
            e @ b"dbReference" => {
                db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        let mut reaction = opttext
            .map(Reaction::new)
            .ok_or(Error::MissingAttribute("text", "reaction"))?;
        reaction.db_references = db_references;
        reaction.evidences = get_evidences(reader, &event)?;

        Ok(reaction)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes a physiological reaction.
pub struct PhysiologicalReaction {
    pub db_reference: DbReference,
    pub evidences: Vec<usize>,
    pub direction: Direction,
}

impl FromXml for PhysiologicalReaction {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"physiologicalReaction");

        use self::Direction::*;

        let evidences = get_evidences(reader, &event)?;
        let direction = decode_attribute(event, reader, "direction", "physiologicalReaction")?;

        let mut optdbref = None;
        parse_inner! {event, reader, buffer,
            e @ b"dbReference" => {
                let dbref = FromXml::from_xml(&e, reader, buffer)?;
                if optdbref.replace(dbref).is_some() {
                    return Err(Error::DuplicateElement("dbReference", "reaction"));
                }
            }
        }

        let db_reference = optdbref.ok_or(Error::MissingAttribute(
            "dbReference",
            "PhysiologicalReaction",
        ))?;
        Ok(PhysiologicalReaction {
            db_reference,
            direction,
            evidences,
        })
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

impl FromStr for Direction {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left-to-right" => Ok(Direction::LeftToRight),
            "right-to-left" => Ok(Direction::RightToLeft),
            other => Err(InvalidValue::from(other)),
        }
    }
}
