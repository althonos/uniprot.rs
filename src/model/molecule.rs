use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::extract_attribute;

#[derive(Debug, Clone)]
/// Describes a molecule by name or unique identifier.
pub enum Molecule {
    Id(String),
    Name(String),
}

impl FromXml for Molecule {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"molecule");

        match extract_attribute(event, "type")? {
            None => reader.read_text(b"molecule", buffer)
                .map(Molecule::Name)
                .map_err(Error::from),
            Some(attr) => {
                reader.read_to_end(b"molecule", buffer)?;
                attr.unescape_and_decode_value(reader)
                    .map(Molecule::Id)
                    .map_err(Error::from)
            }
        }
    }
}
