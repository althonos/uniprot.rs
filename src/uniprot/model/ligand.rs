use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::FromXml;

use super::db_reference::DbReference;

#[derive(Debug, Clone)]
/// Describes a ligand.
pub struct Ligand {
    pub name: String,
    pub db_reference: Option<DbReference>,
    pub label: Option<String>,
    pub note: Option<String>,
}

impl Ligand {
    pub fn new(name: String) -> Self {
        Self {
            name,
            db_reference: None,
            label: None,
            note: None,
        }
    }
}

impl FromXml for Ligand {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"ligand");

        // extract the location and variants
        let mut db_reference: Option<DbReference> = None;
        let mut label: Option<String> = None;
        let mut note: Option<String> = None;
        let mut optname: Option<String> = None;
        parse_inner! {event, reader, buffer,
            e @ b"dbReference" => {
                let dbref = DbReference::from_xml(&e, reader, buffer)?;
                if db_reference.replace(dbref).is_some() {
                    return Err(Error::DuplicateElement("dbReference", "ligand"));
                }
            },
            b"note" => {
                let text = reader.read_text(b"note", buffer)?;
                if note.replace(text).is_some() {
                    return Err(Error::DuplicateElement("note", "ligand"))
                }
            },
            b"label" => {
                let text = reader.read_text(b"label", buffer)?;
                if label.replace(text).is_some() {
                    return Err(Error::DuplicateElement("label", "ligand"))
                }
            },
            b"name" => {
                let text = reader.read_text(b"name", buffer)?;
                if optname.replace(text).is_some() {
                    return Err(Error::DuplicateElement("name", "ligand"))
                }
            }
        }

        // make sure the name was found and return a ligand
        let name = optname.ok_or(Error::MissingAttribute("name", "ligand"))?;
        Ok(Self {
            name,
            label,
            note,
            db_reference,
        })
    }
}