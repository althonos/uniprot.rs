use std::io::BufRead;

use crate::error::Error;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;
use quick_xml::events::BytesStart;
use quick_xml::Reader;

/// A protein sequence.
#[derive(Debug, Clone)]
pub struct Sequence {
    pub sequence: String,
    pub length: usize,
    pub checksum: u64,
}

impl FromXml for Sequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"sequence");

        // decode attributes
        let length = decode_attribute(event, reader, "length", "sequence")?;
        let checksum = extract_attribute(event, "checksum")?
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| u64::from_str_radix(&x, 16))
            .ok_or(Error::MissingAttribute("checksum", "sequence"))??;

        // extract `sequence` element
        let sequence = reader.read_text(b"sequence", buffer)?;
        Ok(Sequence {
            sequence,
            length,
            checksum,
        })
    }
}
