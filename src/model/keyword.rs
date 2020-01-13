use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

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
        keyword.id = attr.get(&b"id"[..])
            .expect("ERR: could not find required `id` on `keyword`")
            .unescape_and_decode_value(reader)?;

        Ok(keyword)
    }
}
