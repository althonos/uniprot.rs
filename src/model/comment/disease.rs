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
            .expect("ERR: could not find required `id` attr on `disease`")?
            .unescape_and_decode_value(reader)?;

        parse_inner!{event, reader, buffer,
            e @ b"name" => {
                let name = reader.read_text(b"name", buffer)?;
                if let Some(_) = optname.replace(name) {
                    panic!("ERR: duplicate `name` in `disease`");
                }
            },
            e @ b"acronym" => {
                let acronym = reader.read_text(b"acronym", buffer)?;
                if let Some(_) = optacro.replace(acronym) {
                    panic!("ERR: duplicate `acronym` in `disease`");
                }
            },
            e @ b"description" => {
                let description = reader.read_text(b"description", buffer)?;
                if let Some(_) = optdesc.replace(description) {
                    panic!("ERR: duplicate `description` in `disease`");
                }
            },
            e @ b"dbReference" => {
                let db_reference = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = optdbref.replace(db_reference) {
                    panic!("ERR: duplicate `db_reference` in `disease`");
                }
            }
        }

        Ok(Disease {
            id,
            name: optname.expect("ERR: missing `name` in `disease`"),
            description: optdesc.expect("ERR: missing `description` in `disease`"),
            acronym: optacro.expect("ERR: missing `acronym` in `disease`"),
            db_reference: optdbref.expect("ERR: missing `db_reference` in `disease`"),
        })

    }
}
