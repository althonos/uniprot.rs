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

use super::super::db_reference::DbReference;

#[derive(Debug, Clone)]
pub struct CatalyticActivity {
    pub reaction: Reaction,
    pub physiological_reactions: Vec<PhysiologicalReaction>
}

impl CatalyticActivity {
    pub fn new(reaction: Reaction) -> Self {
        Self {
            reaction,
            physiological_reactions: Vec::new()
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Reaction {
    pub text: String,
    pub db_references: Vec<DbReference>,
    pub evidences: Vec<usize>,
}

impl Reaction {
    pub fn new(text: String) -> Self {
        Self {
            text,
            db_references: Default::default(),
            evidences: Default::default()
        }
    }
}

impl FromXml for Reaction {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"reaction");

        let attr = attributes_to_hashmap(event)?;
        let mut db_references = Vec::new();
        let mut opttext = None;

        parse_inner!{event, reader, buffer,
            b"text" => {
                let text = reader.read_text(b"text", buffer)?;
                if let Some(_) = opttext.replace(text) {
                    return Err(Error::DuplicateElement("text", "reaction"));
                }
            },
            e @ b"dbReference" => {
                db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        let mut reaction = opttext.map(Reaction::new)
            .ok_or(Error::MissingAttribute("text", "reaction"))?;
        reaction.db_references = db_references;
        reaction.evidences = get_evidences(reader, &attr)?;

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
        debug_assert_eq!(event.local_name(), b"physiologicalReaction");

        use self::Direction::*;

        let attr = attributes_to_hashmap(event)?;
        let evidences = get_evidences(reader, &attr)?;
        let direction = decode_attribute(
            event,
            reader,
            "direction",
            "physiologicalReaction"
        )?;

        let mut optdbref = None;
        parse_inner!{event, reader, buffer,
            e @ b"dbReference" => {
                let dbref = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = optdbref.replace(dbref) {
                    return Err(Error::DuplicateElement("dbReference", "reaction"));
                }
            }
        }

        let db_reference = optdbref
            .ok_or(Error::MissingAttribute("dbReference", "PhysiologicalReaction"))?;
        Ok(PhysiologicalReaction { db_reference, direction, evidences })
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Direction {
    LeftToRight,
    RightToLeft
}

impl FromStr for Direction {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left-to-right" => Ok(Direction::LeftToRight),
            "right-to-left" => Ok(Direction::RightToLeft),
            other => Err(InvalidValue::from(other))
        }
    }
}
