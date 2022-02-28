use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

#[derive(Debug, Clone, Default)]
/// Describes a gene.
pub struct Gene {
    pub names: Vec<Name>,
}

impl FromXml for Gene {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"gene");

        let mut gene = Gene::default();
        parse_inner! {event, reader, buffer,
            e @ b"name" => {
                gene.names.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(gene)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes different types of gene designations.
pub struct Name {
    pub value: String,
    pub ty: NameType,
    pub evidence: Vec<usize>,
}

impl Name {
    pub fn new(value: String, ty: NameType) -> Self {
        Self::new_with_evidence(value, ty, Vec::new())
    }

    pub fn new_with_evidence(value: String, ty: NameType, evidence: Vec<usize>) -> Self {
        Self {
            value,
            ty,
            evidence,
        }
    }
}

// ---------------------------------------------------------------------------

impl FromXml for Name {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"name");

        let attr = attributes_to_hashmap(event)?;
        let name = reader.read_text(event.local_name(), buffer)?;
        let evidence = get_evidences(reader, &attr)?;
        let ty = decode_attribute(event, reader, "type", "name")?;

        Ok(Self::new_with_evidence(name, ty, evidence))
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameType {
    Primary,
    Synonym,
    OrderedLocus,
    Orf,
}

impl FromStr for NameType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "primary" => Ok(NameType::Primary),
            "synonym" => Ok(NameType::Synonym),
            "ordered locus" => Ok(NameType::OrderedLocus),
            "ORF" => Ok(NameType::Orf),
            other => Err(InvalidValue::from(other)),
        }
    }
}
