use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

#[derive(Debug, Clone)]
pub struct InterproReference {
    pub name: String,
    pub id: String,
}

impl FromXml for InterproReference {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"ipr");

        let name = extract_attribute(event, "name")?
            .ok_or(Error::MissingAttribute("name", "signatureSequenceMatch"))?
            .decode_and_unescape_value(reader)
            .map(Cow::into_owned)?;
        let id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "signatureSequenceMatch"))?
            .decode_and_unescape_value(reader)
            .map(Cow::into_owned)?;

        reader.read_to_end_into(event.name(), buffer)?;
        Ok(InterproReference { name, id })
    }
}
