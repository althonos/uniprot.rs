use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::decode_attribute;
use crate::parser::FromXml;

use super::Property;

/// A UniRef database reference.
#[derive(Debug, Clone)]
pub struct Reference {
    pub id: ShortString,
    pub ty: ShortString,
    pub properties: Vec<Property>,
}

impl FromXml for Reference {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"dbReference");

        // decode attributes
        let id = decode_attribute(event, reader, "id", "reference")?;
        let ty = decode_attribute(event, reader, "type", "reference")?;

        let mut properties = Vec::new();
        parse_inner! {event, reader, buffer,
            e @ b"property" => {
                properties.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(Reference { id, ty, properties })
    }
}
