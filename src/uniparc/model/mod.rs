//! Object model of the UniParc XML database.
//!
//! The model was derived from the latest [UniParc XML schema], with some
//! small changes to make the type hierarchy fit with the Rust conventions.
//!
//! [UniParc XML schema]: https://www.uniprot.org/docs/uniparc.xsd

// ---------------------------------------------------------------------------

mod db_reference;
mod ipr;
mod location;
mod sigseq;

pub use self::location::Location;
pub use self::ipr::InterproReference;
pub use self::db_reference::DbReference;
pub use self::sigseq::SignatureSequenceMatch;
pub use crate::common::date::Date;
pub use crate::common::property::Property;
pub use crate::common::sequence::Sequence;

use std::io::BufRead;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::DerefMut;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::extract_attribute;
use crate::parser::UniprotDatabase;
use crate::parser::FromXml;

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// A UniParc entry.
pub struct Entry {
    // attributes
    pub dataset: String,
    // fields
    pub accession: String,
    pub db_references: Vec<DbReference>,
    pub signature_sequence_matches: Vec<SignatureSequenceMatch>,
    pub sequence: Sequence,
}

impl FromXml for Entry {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"entry");

        let dataset = extract_attribute(event, "dataset")?
            .ok_or(Error::MissingAttribute("dataset", "entry"))?
            .unescape_and_decode_value(reader)?;

        let mut accession = None;
        let mut sequence = None;
        let mut db_references = Vec::new();
        let mut signature_sequence_matches = Vec::new();
        parse_inner! {event, reader, buffer,
            b"accession" => {
                if accession.replace(reader.read_text(b"accession", buffer)?).is_some() {
                    return Err(Error::DuplicateElement("accession", "entry"));
                }
            },
            e @ b"dbReference" => {
                db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"signatureSequenceMatch" => {
                signature_sequence_matches.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"sequence" => {
                if sequence.replace(FromXml::from_xml(&e, reader, buffer)?).is_some() {
                    return Err(Error::DuplicateElement("sequence", "entry"));
                }
            }
        }

        Ok(Entry {
            dataset,
            db_references,
            signature_sequence_matches,
            accession: accession.ok_or(Error::MissingElement("accession", "entry"))?,
            sequence: sequence.ok_or(Error::MissingElement("sequence", "entry"))?,
        })
    }
}

// ---------------------------------------------------------------------------

/// A UniParc database.
#[derive(Debug, Clone)]
pub struct UniParc {
    entries: Vec<Entry>
}

impl UniParc {
    /// Create a new UniRef database with the given entries.
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl Deref for UniParc {
    type Target = Vec<Entry>;
    fn deref(&self) -> &Vec<Entry> {
        &self.entries
    }
}

impl DerefMut for UniParc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl FromIterator<Entry> for UniParc {
    fn from_iter<T: IntoIterator<Item=Entry>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl From<Vec<Entry>> for UniParc {
    fn from(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl From<UniParc> for Vec<Entry> {
    fn from(uniparc: UniParc) -> Self {
        uniparc.entries
    }
}

impl UniprotDatabase for UniParc {
    type Entry = Entry;
    const ROOTS: &'static [&'static [u8]] = &[ b"uniparc"];
}
