use std::io::BufRead;

use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;
use quick_xml::events::BytesStart;
use quick_xml::Reader;
use crate::error::Error;

/// A single key-value property.
#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    pub value: String,
}

impl FromXml for Property {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"property");

        let key = extract_attribute(event, "type")?
            .ok_or(Error::MissingAttribute("type", "property"))?
            .unescape_and_decode_value(reader)?;
        let value = extract_attribute(event, "value")?
            .ok_or(Error::MissingAttribute("value", "property"))?
            .unescape_and_decode_value(reader)?;

        reader.read_to_end(event.local_name(), buffer)?;
        Ok(Property { key, value })
    }
}
