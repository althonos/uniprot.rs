//! Object model of the UniRef XML database.
//!
//! The model was derived from the latest [UniRef XML schema], with some
//! small changes to make the type hierarchy fit with the Rust conventions.
//!
//! [UniRef XML schema]: https://www.uniprot.org/docs/uniref.xsd

mod member;
mod reference;
mod sequence;

pub use self::member::Member;
pub use self::reference::Reference;
pub use self::sequence::Sequence;
pub use crate::common::date::Date;
pub use crate::common::property::Property;

use std::io::BufRead;
use crate::parser::utils::decode_attribute;
use crate::parser::FromXml;
use quick_xml::events::BytesStart;
use quick_xml::Reader;
use crate::error::Error;

/// A UniRef entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub updated: Date,
    pub name: String,
    pub properties: Vec<Property>,
    pub representative_member: Member,
    pub members: Vec<Member>,
}

impl FromXml for Entry {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"entry");

        // parse attributes
        let id = decode_attribute(event, reader, "id", "reference")?;
        let updated = decode_attribute(event, reader, "updated", "entry")?;

        // parse fields
        let mut name = None;
        let mut representative_member = None;
        let mut properties = Vec::new();
        let mut members = Vec::new();
        parse_inner! {event, reader, buffer,
            b"name" => {
                if name.replace(reader.read_text(b"name", buffer)?).is_some() {
                    return Err(Error::DuplicateElement("name", "entry"));
                }
            },
            e @ b"representativemember" => {
                if representative_member.replace(FromXml::from_xml(&e, reader, buffer)?).is_some() {
                    return Err(Error::DuplicateElement("representativemember", "entry"));
                }
            },
            e @ b"member" => {
                members.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"property" => {
                properties.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(Entry {
            id,
            updated,
            name: name.ok_or(Error::MissingElement("name", "entry"))?,
            properties,
            representative_member: representative_member
                .ok_or(Error::MissingElement("representativemember", "entry"))?,
            members,
        })
    }
}
