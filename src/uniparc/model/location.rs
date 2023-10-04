use std::io::BufRead;

use crate::error::Error;
use crate::parser::utils::decode_attribute;
use crate::parser::FromXml;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

#[derive(Debug, Clone)]
pub struct Location {
    pub start: u64,
    pub end: u64,
}

impl Location {
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }
}

impl FromXml for Location {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"lcn");

        let start = decode_attribute(event, reader, "start", "lcn")?;
        let end = decode_attribute(event, reader, "end", "lcn")?;
        reader.read_to_end_into(event.name(), buffer)?;

        Ok(Location::new(start, end))
    }
}
