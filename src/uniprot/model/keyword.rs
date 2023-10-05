use std::borrow::Cow;
use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::extract_attribute;
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

        let mut keyword = Keyword::default();
        keyword.value = parse_text!(event, reader, buffer);
        keyword.evidence = get_evidences(reader, &event)?;
        keyword.id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "keyword"))?
            .decode_and_unescape_value(reader)
            .map(Cow::into_owned)?;

        Ok(keyword)
    }
}
