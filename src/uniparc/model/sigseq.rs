use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::InterproReference;
use super::Location;

#[derive(Debug, Clone)]
pub struct SignatureSequenceMatch {
    pub database: String,
    pub id: String,
    pub interpro: InterproReference,
    pub locations: Vec<Location>,
}

impl FromXml for SignatureSequenceMatch {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        assert_eq!(event.local_name(), b"signatureSequenceMatch");

        let database = extract_attribute(event, "database")?
            .ok_or(Error::MissingAttribute(
                "database",
                "signatureSequenceMatch",
            ))?
            .unescape_and_decode_value(reader)?;
        let id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "signatureSequenceMatch"))?
            .unescape_and_decode_value(reader)?;

        let mut interpro = None;
        let mut locations = Vec::new();
        parse_inner! {event, reader, buffer,
            e @ b"ipr" => {
                if interpro.replace(FromXml::from_xml(&e, reader, buffer)?).is_some() {
                    return Err(Error::DuplicateElement("ipr", "dbReference"));
                }
            },
            e @ b"lcn" => {
                locations.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(SignatureSequenceMatch {
            database,
            id,
            locations,
            interpro: interpro.ok_or(Error::MissingElement("ipr", "dbReference"))?,
        })
    }
}
