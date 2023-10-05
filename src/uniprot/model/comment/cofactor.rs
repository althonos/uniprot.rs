use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::super::db_reference::DbReference;

#[derive(Debug, Default, Clone)]
pub struct Cofactor {
    pub name: ShortString,
    pub db_reference: DbReference,
    pub evidences: Vec<usize>,
}

impl FromXml for Cofactor {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"cofactor");

        let mut optname = None;
        let mut optdbref = None;

        parse_inner! {event, reader, buffer,
            e @ b"name" => {
                let name = parse_text!(e, reader, buffer);
                if optname.replace(name).is_some() {
                    return Err(Error::DuplicateElement("name", "cofactor"));
                }
            },
            e @ b"dbReference" => {
                let dbref = FromXml::from_xml(&e, reader, buffer)?;
                if optdbref.replace(dbref).is_some() {
                    return Err(Error::DuplicateElement("dbReference", "cofactor"));
                }
            }
        }

        let name = optname.ok_or(Error::MissingElement("name", "cofactor"))?;
        let db_reference = optdbref.ok_or(Error::MissingElement("dbReference", "cofactor"))?;
        let evidences = get_evidences(reader, event)?;

        Ok(Cofactor {
            name,
            db_reference,
            evidences,
        })
    }
}
