use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::FromXml;

use super::db_reference::DbReference;

#[derive(Debug, Clone)]
/// Describes a ligand part.
pub struct LigandPart {
    pub name: ShortString,
    pub db_reference: Option<DbReference>,
    pub label: Option<ShortString>,
    pub note: Option<ShortString>,
}

impl LigandPart {
    pub fn new(name: ShortString) -> Self {
        Self {
            name,
            db_reference: None,
            label: None,
            note: None,
        }
    }
}

impl FromXml for LigandPart {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"ligandPart");

        // extract the location and variants
        let mut db_reference: Option<DbReference> = None;
        let mut label: Option<ShortString> = None;
        let mut note: Option<ShortString> = None;
        let mut optname: Option<ShortString> = None;
        parse_inner! {event, reader, buffer,
            e @ b"dbReference" => {
                let dbref = DbReference::from_xml(&e, reader, buffer)?;
                if db_reference.replace(dbref).is_some() {
                    return Err(Error::DuplicateElement("dbReference", "ligandPart"));
                }
            },
            e @ b"note" => {
                let text = parse_text!(e, reader, buffer);
                if note.replace(text).is_some() {
                    return Err(Error::DuplicateElement("note", "ligandPart"))
                }
            },
            e @ b"label" => {
                let text = parse_text!(e, reader, buffer);
                if label.replace(text).is_some() {
                    return Err(Error::DuplicateElement("label", "ligandPart"))
                }
            },
            e @ b"name" => {
                let text = parse_text!(e, reader, buffer);
                if optname.replace(text).is_some() {
                    return Err(Error::DuplicateElement("name", "ligandPart"))
                }
            }
        }

        // make sure the name was found and return a ligand
        let name = optname.ok_or(Error::MissingAttribute("name", "ligandPart"))?;
        Ok(Self {
            name,
            label,
            note,
            db_reference,
        })
    }
}
