use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use super::Date;
use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

#[derive(Debug, Default, Clone)]
/// The sequence of a protein.
pub struct Sequence {
    pub value: String,
    pub length: usize,
    pub mass: usize,
    pub checksum: u64,
    pub modified: Date,
    pub version: usize,
    pub precursor: Option<bool>,
    pub fragment: Option<FragmentType>,
}

impl FromXml for Sequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"sequence");

        let length = decode_attribute(event, reader, "length", "sequence")?;
        let mass = decode_attribute(event, reader, "mass", "sequence")?;
        let version = decode_attribute(event, reader, "version", "sequence")?;
        let modified = decode_attribute(event, reader, "modified", "sequence")?;
        // let modified = TODO
        let precursor = extract_attribute(event, "precursor")?
            .map(|x| x.decode_and_unescape_value(reader))
            .transpose()?
            .map(|x| bool::from_str(&x))
            .transpose()?;
        let checksum = extract_attribute(event, "checksum")?
            .map(|x| x.decode_and_unescape_value(reader))
            .transpose()?
            .map(|x| u64::from_str_radix(&x, 16))
            .ok_or(Error::MissingAttribute("checksum", "sequence"))??;
        let fragment = match decode_attribute(event, reader, "fragment", "sequence") {
            Ok(fragment) => Some(fragment),
            Err(Error::MissingAttribute(_, _)) => None,
            Err(other) => return Err(other),
        };

        let value = parse_text!(event, reader, buffer);
        Ok(Sequence {
            value,
            length,
            mass,
            checksum,
            modified,
            version,
            precursor,
            fragment,
        })
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A marker indicating whether a protein sequence is fragmented.
pub enum FragmentType {
    Single,
    Multiple,
}

impl Default for FragmentType {
    fn default() -> Self {
        FragmentType::Single
    }
}

impl FromStr for FragmentType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single" => Ok(Self::Single),
            "multiple" => Ok(Self::Multiple),
            other => Err(InvalidValue::from(other)),
        }
    }
}
