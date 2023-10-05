use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::db_reference::DbReference;

#[derive(Debug, Default, Clone)]
/// Describes the source organism.
pub struct Organism {
    pub names: Vec<Name>,
    pub db_references: Vec<DbReference>,
    pub lineages: Vec<Lineage>,
    pub evidences: Vec<usize>,
}

impl FromXml for Organism {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert!(
            event.local_name().as_ref() == b"organism"
                || event.local_name().as_ref() == b"organismHost"
        );

        let mut organism = Organism::default();
        organism.evidences = get_evidences(reader, &event)?;
        parse_inner! {event, reader, buffer,
            e @ b"name" => {
                organism.names.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"dbReference" => {
                organism.db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"lineage" => {
                organism.lineages.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(organism)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Name {
    pub value: String,
    pub ty: NameType,
}

impl Name {
    pub fn new(value: String, ty: NameType) -> Self {
        Self { value, ty }
    }
}

impl FromXml for Name {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"name");

        let value = parse_text!(event, reader, buffer);
        let ty = decode_attribute(event, reader, "type", "name")?;
        Ok(Name::new(value, ty))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameType {
    Common,
    Full,
    Scientific,
    Synonym,
    Abbreviation,
}

impl FromStr for NameType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "common" => Ok(NameType::Common),
            "full" => Ok(NameType::Full),
            "scientific" => Ok(NameType::Scientific),
            "synonym" => Ok(NameType::Synonym),
            "abbreviation" => Ok(NameType::Abbreviation),
            other => Err(InvalidValue::from(other)),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct Lineage {
    pub taxons: Vec<String>,
}

impl FromXml for Lineage {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"lineage");

        let mut lineage = Lineage::default();
        parse_inner! {event, reader, buffer,
            e @ b"taxon" => {
                lineage.taxons.push(parse_text!(e, reader, buffer));
            }
        }

        Ok(lineage)
    }
}
