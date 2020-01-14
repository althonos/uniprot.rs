use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;

#[derive(Debug, Default, Clone)]
pub struct Property {
    pub ty: String,
    pub value: String,
}

impl Property {
    pub fn new(ty: String, value: String) -> Self {
        Self { ty, value }
    }
}

impl FromXml for Property {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"property");

        let attr = attributes_to_hashmap(event)?;
        reader.read_to_end(b"property", buffer)?;

        let ty = attr.get(&b"type"[..])
            .ok_or(Error::MissingAttribute("type", "property"))?
            .unescape_and_decode_value(reader)?;
        let value = attr.get(&b"value"[..])
            .ok_or(Error::MissingAttribute("value", "property"))?
            .unescape_and_decode_value(reader)?;

        Ok(Property::new(ty, value))
    }
}
