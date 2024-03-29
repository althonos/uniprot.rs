use std::borrow::Cow;
use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

use super::db_reference::DbReference;

#[derive(Debug, Clone)]
/// The evidence for an annotation.
pub struct Evidence {
    pub key: usize,
    pub ty: ShortString,
    pub source: Option<Source>,
    pub imported_from: Option<DbReference>,
}

impl Evidence {
    pub fn new(key: usize, ty: ShortString) -> Self {
        Self {
            key,
            ty,
            source: None,
            imported_from: None,
        }
    }
}

impl FromXml for Evidence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"evidence");

        let key = decode_attribute(event, reader, "key", "evidence")?;
        let ty = extract_attribute(event, "type")?
            .map(|x| x.decode_and_unescape_value(reader))
            .ok_or(Error::MissingAttribute("type", "evidence"))??
            .into();

        let mut evidence = Self::new(key, ty);
        parse_inner! {event, reader, buffer,
            e @ b"source" => {
                let source = FromXml::from_xml(&e, reader, buffer)?;
                if evidence.source.replace(source).is_some() {
                    return Err(Error::DuplicateElement("source", "evidence"));
                }
            },
            e @ b"importedFrom" => {
                parse_inner!{e, reader, buffer,
                    d @ b"dbReference" => {
                        let dbref = FromXml::from_xml(&d, reader, buffer)?;
                        if evidence.imported_from.replace(dbref).is_some() {
                            return Err(Error::DuplicateElement("importedFrom", "evidence"));
                        }
                    }
                }
            }
        }

        Ok(evidence)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// A reference to the source of the data.
pub enum Source {
    /// A cross-reference to another database, such as PubMed.
    DbRef(DbReference),
    /// An internal reference to a source only cited within the entry.
    Ref(usize),
}

impl FromXml for Source {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"source");

        let mut optdbref = None;
        parse_inner! {event, reader, buffer,
            e @ b"dbReference" => {
                let db_reference = FromXml::from_xml(&e, reader, buffer)?;
                if optdbref.replace(db_reference).is_some() {
                    return Err(Error::DuplicateElement("dbReference", "source"));
                }
            }
        }

        if let Some(db_reference) = optdbref {
            Ok(Source::DbRef(db_reference))
        } else {
            decode_attribute(event, reader, "ref", "source").map(Source::Ref)
        }
    }
}
