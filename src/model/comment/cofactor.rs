use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

use super::super::db_reference::DbReference;

#[derive(Debug, Default, Clone)]
pub struct Cofactor {
    pub name: String,
    pub db_reference: DbReference,
    pub evidences: Vec<usize>,
}

impl FromXml for Cofactor {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(b"cofactor", event.local_name());

        let attr = attributes_to_hashmap(event)?;
        let mut optname = None;
        let mut optdbref = None;

        parse_inner!{event, reader, buffer,
            b"name" => {
                let name = reader.read_text(b"name", buffer)?;
                if let Some(_) = optname.replace(name) {
                    return Err(Error::DuplicateElement("name", "cofactor"));
                }
            },
            e @ b"dbReference" => {
                let dbref = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = optdbref.replace(dbref) {
                    return Err(Error::DuplicateElement("dbReference", "cofactor"));
                }
            }
        }

        let name = optname.ok_or(Error::MissingElement("name", "cofactor"))?;
        let db_reference = optdbref.ok_or(Error::MissingElement("dbReference", "cofactor"))?;
        let evidences = get_evidences(reader, &attr)?;

        Ok(Cofactor { name, db_reference, evidences })
    }
}
