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
        debug_assert_eq!(event.local_name().as_ref(), b"sequence");

        // decode attributes
        let length = decode_attribute(event, reader, "length", "sequence")?;
        let checksum = extract_attribute(event, "checksum")?
            .map(|x| x.decode_and_unescape_value(reader))
            .transpose()?
            .map(|x| u64::from_str_radix(&x, 16))
            .ok_or(Error::MissingAttribute("checksum", "sequence"))??;

        // extract `sequence` element
        let sequence = parse_text!(event, reader, buffer);
        Ok(Sequence {
            sequence,
            length,
            checksum,
        })
    }
}
