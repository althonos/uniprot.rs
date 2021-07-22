use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

#[derive(Debug, Default, Clone)]
pub struct Keyword {
    pub id: String,
    pub value: String,
    pub evidence: Vec<usize>,
}

impl FromXml for Keyword {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"keyword");

        let attr = attributes_to_hashmap(event)?;
        let mut keyword = Keyword::default();

        keyword.value = reader.read_text(b"keyword", buffer)?;
        keyword.evidence = get_evidences(reader, &attr)?;
        keyword.id = attr
            .get(&b"id"[..])
            .ok_or(Error::MissingAttribute("id", "keyword"))?
            .unescape_and_decode_value(reader)?;

        Ok(keyword)
    }
}
