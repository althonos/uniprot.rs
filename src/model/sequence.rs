use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::decode_attribute;

#[derive(Debug, Default, Clone)]
pub struct Sequence {
    pub value: String,
    pub length: usize,
    pub mass: usize,
    pub checksum: u64,
    // modified: NaiveDate,
    pub version: usize,
    pub precursor: Option<bool>,
    pub fragment: Option<FragmentType>
}

impl FromXml for Sequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"sequence");

        let attr = attributes_to_hashmap(event)?;
        let length = attr.get(&b"length"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .ok_or(Error::MissingAttribute("length", "sequence"))??;
        let mass = attr.get(&b"mass"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .ok_or(Error::MissingAttribute("mass", "sequence"))??;
        let checksum = attr.get(&b"checksum"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| u64::from_str_radix(&x, 16))
            .ok_or(Error::MissingAttribute("checksum", "sequence"))??;
        // let modified = TODO
        let version = attr.get(&b"version"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .ok_or(Error::MissingAttribute("version", "sequence"))??;
        let precursor = attr.get(&b"precursor"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| bool::from_str(&x))
            .transpose()?;
        let fragment = match decode_attribute(event, reader, "fragment", "sequence") {
            Ok(fragment) => Some(fragment),
            Err(Error::MissingAttribute(_, _)) => None,
            Err(other) => return Err(other),
        };

        let value = reader.read_text(b"sequence", buffer)?;
        Ok(Sequence {
            value,
            length,
            mass,
            checksum,
            version,
            precursor,
            fragment,
        })
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum FragmentType {
    Single,
    Multiple,
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
