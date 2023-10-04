use std::borrow::Cow;
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
        debug_assert_eq!(event.local_name().as_ref(), b"keyword");

        let attr = attributes_to_hashmap(event)?;
        let mut keyword = Keyword::default();

        keyword.value = parse_text!(event, reader, buffer);
        keyword.evidence = get_evidences(reader, &attr)?;
        keyword.id = attr
            .get(&b"id"[..])
            .ok_or(Error::MissingAttribute("id", "keyword"))?
            .decode_and_unescape_value(reader)
            .map(Cow::into_owned)?;

        Ok(keyword)
    }
}
