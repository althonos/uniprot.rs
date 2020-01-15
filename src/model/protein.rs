use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::get_evidences;
use crate::parser::utils::extract_attribute;

#[derive(Debug, Clone, Default)]
/// Describes the names for the protein and parts thereof.
pub struct Protein {
    pub name: Nomenclature,
    pub domains: Vec<Nomenclature>,
    pub components: Vec<Nomenclature>,
}

impl FromXml for Protein {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        let mut protein = Protein::default();
        parse_inner! {event, reader, buffer,
            e @ b"recommendedName" => {
                protein.name.recommended = FromXml::from_xml(&e, reader, buffer).map(Some)?;
            },
            e @ b"alternativeName" => {
                protein.name.alternative.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"component" => {
                // TODO: proper fix to avoid nested `component` in `component`
                protein.components.push(Self::from_xml(&e, reader, buffer)?.name);
            },
            e @ b"domain" => {
                // TODO: proper fix to avoid nested `domain` in `component`
                protein.domains.push(Self::from_xml(&e, reader, buffer)?.name);
            },
            b"allergenName" => {
                let value = reader.read_text(b"allergenName", buffer)?;
                if let Some(_) = protein.name.allergen.replace(value) {
                    return Err(Error::DuplicateElement("allergen", "protein"));
                }
            },
            b"biotechName" => {
                let value = reader.read_text(b"biotechName", buffer)?;
                if let Some(_) = protein.name.biotech.replace(value) {
                    return Err(Error::DuplicateElement("biotech", "protein"));
                }
            },
            b"cdAntigenName" => {
                let value = reader.read_text(b"cdAntigenName", buffer)?;
                protein.name.cd_antigen.push(value);
            },
            b"innName" => {
                let value = reader.read_text(b"innName", buffer)?;
                protein.name.inn.push(value);

            }
        }

        Ok(protein)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Nomenclature {
    pub recommended: Option<Name>,
    pub alternative: Vec<Name>,
    pub submitted: Vec<Name>,
    pub allergen: Option<String>,     // FIXME: type should be EvidenceString?
    pub biotech: Option<String>,
    pub cd_antigen: Vec<String>,
    pub inn: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Name {
    pub full: String,
    pub short: Vec<String>,
    pub ec_number: Vec<String>,
}

impl FromXml for Name {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        let mut group = Self::default();

        parse_inner!{event, reader, buffer,
            b"fullName" => {
                group.full = reader.read_text(b"fullName", buffer)?;
            },
            b"shortName" => {
                group.short.push(reader.read_text(b"shortName", buffer)?);
            },
            b"ecNumber" => {
                group.ec_number.push(reader.read_text(b"ecNumber", buffer)?);
            }
        };

        Ok(group)
    }
}

#[derive(Debug, Clone)]
/// Describes the evidence for the protein's existence.
pub enum ProteinExistence {
    ProteinLevelEvidence,
    TranscriptLevelEvidence,
    HomologyInferred,
    Predicted,
    Uncertain,
}

impl Default for ProteinExistence {
    fn default() -> Self {
        ProteinExistence::Uncertain
    }
}

impl FromXml for ProteinExistence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"proteinExistence");

        use self::ProteinExistence::*;

        reader.read_to_end(event.local_name(), buffer)?;
        match extract_attribute(event, &b"type"[..])?
            .as_ref()
            .map(|a| &*a.value)
        {
            Some(b"evidence at protein level") => Ok(ProteinLevelEvidence),
            Some(b"evidence at transcript level") => Ok(TranscriptLevelEvidence),
            Some(b"inferred from homology") => Ok(HomologyInferred),
            Some(b"predicted") => Ok(Predicted),
            Some(b"uncertain") => Ok(Uncertain),
            None => return Err(Error::MissingAttribute("type", "proteinExistence")),
            Some(other) => return Err(
                Error::invalid_value("other", "ProteinExistence", String::from_utf8_lossy(other))
            )
        }
    }
}
