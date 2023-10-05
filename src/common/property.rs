use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

#[derive(Debug, Clone)]
/// A single key-value property.
pub struct Property {
    pub ty: ShortString,
    pub value: ShortString,
}

impl Property {
    pub fn new(ty: ShortString, value: ShortString) -> Self {
        Self { ty, value }
    }
}

impl FromXml for Property {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"property");

        reader.read_to_end_into(event.name(), buffer)?;
        let ty = extract_attribute(event, "type")?
            .ok_or(Error::MissingAttribute("type", "property"))?
            .decode_and_unescape_value(reader)?
            .into();
        let value = extract_attribute(event, "value")?
            .ok_or(Error::MissingAttribute("value", "property"))?
            .decode_and_unescape_value(reader)?
            .into();

        Ok(Property::new(ty, value))
    }
}
