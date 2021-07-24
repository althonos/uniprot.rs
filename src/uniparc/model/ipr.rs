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
        assert_eq!(event.local_name(), b"ipr");

        let name = extract_attribute(event, "name")?
            .ok_or(Error::MissingAttribute("name", "signatureSequenceMatch"))?
            .unescape_and_decode_value(reader)?;
        let id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "signatureSequenceMatch"))?
            .unescape_and_decode_value(reader)?;

        reader.read_to_end(event.local_name(), buffer)?;
        Ok(InterproReference {
            name,
            id
        })
    }
}
