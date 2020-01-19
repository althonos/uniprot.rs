use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

use super::super::db_reference::DbReference;

#[derive(Debug, Clone)]
pub struct Disease {
    pub id: String,
    pub name: String,
    pub description: String,
    pub acronym: String,
    pub db_reference: DbReference,
}

impl FromXml for Disease {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"disease");

        let mut optname = None;
        let mut optdesc = None;
        let mut optacro = None;
        let mut optdbref = None;

        let id = event.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"id").unwrap_or_default())
            .ok_or(Error::MissingAttribute("id", "disease"))??
            .unescape_and_decode_value(reader)?;

        parse_inner!{event, reader, buffer,
            b"name" => {
                let name = reader.read_text(b"name", buffer)?;
                if optname.replace(name).is_some() {
                    return Err(Error::DuplicateElement("name", "disease"));
                }
            },
            b"acronym" => {
                let acronym = reader.read_text(b"acronym", buffer)?;
                if optacro.replace(acronym).is_some() {
                    return Err(Error::DuplicateElement("acronym", "disease"));
                }
            },
            b"description" => {
                let description = reader.read_text(b"description", buffer)?;
                if optdesc.replace(description).is_some() {
                    return Err(Error::DuplicateElement("description", "disease"));
                }
            },
            e @ b"dbReference" => {
                let db_reference = FromXml::from_xml(&e, reader, buffer)?;
                if optdbref.replace(db_reference).is_some() {
                    return Err(Error::DuplicateElement("dbReference", "disease"));
                }
            }
        }

        Ok(Disease {
            id,
            name: optname.ok_or(Error::MissingElement("name", "disease"))?,
            description: optdesc.ok_or(Error::MissingElement("description", "disease"))?,
            acronym: optacro.ok_or(Error::MissingElement("acronym", "disease"))?,
            db_reference: optdbref.ok_or(Error::MissingElement("dbReference", "disease"))?,
        })

    }
}
