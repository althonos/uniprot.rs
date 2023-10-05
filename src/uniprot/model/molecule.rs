use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

#[derive(Debug, Clone)]
/// Describes a molecule by name or unique identifier.
pub enum Molecule {
    Id(ShortString),
    Name(ShortString),
}

impl FromXml for Molecule {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"molecule");

        match extract_attribute(event, "type")? {
            None => Ok(Molecule::Name(parse_text!(event, reader, buffer))),
            Some(attr) => {
                reader.read_to_end_into(event.name(), buffer)?;
                attr.decode_and_unescape_value(reader)
                    .map_err(Error::from)
                    .map(ShortString::from)
                    .map(Molecule::Id)
            }
        }
    }
}
