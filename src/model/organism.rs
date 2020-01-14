use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;

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
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert!(
            event.local_name() == b"organism"
            || event.local_name() == b"organismHost"
        );

        let attr = attributes_to_hashmap(event)?;

        let mut organism = Organism::default();
        organism.evidences = get_evidences(reader, &attr)?;
        parse_inner!{event, reader, buffer,
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

#[derive(Debug, Clone)]
pub enum Name {
    Common(String),
    Full(String),
    Scientific(String),
    Synonym(String),
    Abbreviation(String),
}

impl FromXml for Name {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"name");

        let value = reader.read_text(b"name", buffer)?;
        match extract_attribute(event, &b"type"[..])?.as_ref().map(|a| &*a.value) {
            Some(b"common") => Ok(Name::Common(value)),
            Some(b"full") => Ok(Name::Full(value)),
            Some(b"scientific") => Ok(Name::Scientific(value)),
            Some(b"synonym") => Ok(Name::Synonym(value)),
            Some(b"abbreviation") => Ok(Name::Abbreviation(value)),
            Some(other) => panic!("ERR: invalid value for organism name type: {:?}", other),
            None => return Err(Error::MissingAttribute("type", "name")),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Lineage {
    pub taxons: Vec<String>,
}

impl FromXml for Lineage {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"lineage");

        let mut lineage = Lineage::default();
        parse_inner!{event, reader, buffer,
            b"taxon" => {
                lineage.taxons.push(reader.read_text(b"taxon", buffer)?);
            }
        }

        Ok(lineage)
    }
}
