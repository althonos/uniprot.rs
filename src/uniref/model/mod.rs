//! Object model of the UniRef XML database.
//!
//! The model was derived from the latest [UniRef XML schema], with some
//! small changes to make the type hierarchy fit with the Rust conventions.
//!
//! [UniRef XML schema]: https://www.uniprot.org/docs/uniref.xsd

mod member;
mod reference;

pub use self::member::Member;
pub use self::reference::Reference;
pub use crate::common::date::Date;
pub use crate::common::property::Property;
pub use crate::common::sequence::Sequence;

use std::io::BufRead;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::DerefMut;
use crate::parser::utils::decode_attribute;
use crate::parser::FromXml;
use crate::parser::UniprotDatabase;
use quick_xml::events::BytesStart;
use quick_xml::Reader;
use crate::error::Error;

// ---------------------------------------------------------------------------

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
            e @ b"representativeMember" => {
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

// ---------------------------------------------------------------------------

/// A UniRef database.
#[derive(Debug, Clone)]
pub struct UniRef {
    entries: Vec<Entry>
}

impl UniRef {
    /// Create a new UniRef database with the given entries.
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl Deref for UniRef {
    type Target = Vec<Entry>;
    fn deref(&self) -> &Vec<Entry> {
        &self.entries
    }
}

impl DerefMut for UniRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl FromIterator<Entry> for UniRef {
    fn from_iter<T: IntoIterator<Item=Entry>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl From<Vec<Entry>> for UniRef {
    fn from(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl From<UniRef> for Vec<Entry> {
    fn from(uniref: UniRef) -> Self {
        uniref.entries
    }
}

impl UniprotDatabase for UniRef {
    type Entry = Entry;
    const ROOTS: &'static [&'static [u8]] = &[ b"UniRef", b"UniRef50", b"UniRef90", b"UniRef100"];
}
