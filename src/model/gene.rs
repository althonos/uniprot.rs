use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

#[derive(Debug, Clone, Default)]
/// Describes a gene.
pub struct Gene {
    pub names: Vec<Name>,
}

impl FromXml for Gene {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"gene");

        let mut gene = Gene::default();
        parse_inner!{event, reader, buffer,
            e @ b"name" => {
                gene.names.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(gene)
    }
}

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
            evidence
        }
    }
}

impl FromXml for Name {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"name");

        let attr = attributes_to_hashmap(event)?;
        let name = reader.read_text(event.local_name(), buffer)?;
        let evidence = get_evidences(reader, &attr)?;
        let ty = match attr.get(&b"type"[..]).map(|a| &*a.value) {
            Some(b"primary") => NameType::Primary,
            Some(b"synonym") => NameType::Synonym,
            Some(b"ordered locus") => NameType::OrderedLocus,
            Some(b"ORF") => NameType::Orf,
            None => return Err(Error::MissingAttribute("type", "name")),
            Some(other) => return Err(Error::invalid_value("type", "name", String::from_utf8_lossy(other))),
        };

        Ok(Self::new_with_evidence(name, ty, evidence))
    }
}

#[derive(Debug, Clone)]
pub enum NameType {
    Primary,
    Synonym,
    OrderedLocus,
    Orf
}
