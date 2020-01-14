use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::extract_attribute;

use super::db_reference::DbReference;

#[derive(Debug, Clone)]
/// Describes the evidence for an annotation.
pub struct Evidence {
    pub key: usize,
    pub ty: String,
    pub source: Option<Source>,
    pub imported_from: Option<DbReference>,
}

impl Evidence {
    pub fn new(key: usize, ty: String) -> Self {
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
        debug_assert_eq!(event.local_name(), b"evidence");

        let attr = attributes_to_hashmap(event)?;
        let ty = attr.get(&b"type"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .ok_or(Error::MissingAttribute("type", "evidence"))??;
        let key = attr.get(&b"key"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .ok_or(Error::MissingAttribute("key", "evidence"))
            .map(|s| usize::from_str(&s))??;

        let mut evidence = Self::new(key, ty);
        parse_inner!{event, reader, buffer,
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

#[derive(Debug, Clone)]
pub enum Source {
    DbRef(DbReference),
    Ref(usize)
}

impl FromXml for Source {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"source");

        let mut optdbref = None;
        parse_inner!{event, reader, buffer,
            e @ b"dbReference" => {
                let db_reference = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = optdbref.replace(db_reference) {
                    return Err(Error::DuplicateElement("dbReference", "source"));
                }
            }
        }

        if let Some(db_reference) = optdbref {
            Ok(Source::DbRef(db_reference))
        } else {
            extract_attribute(event, b"ref")?
                .ok_or(Error::MissingAttribute("ref", "source"))
                .and_then(|a| a.unescape_and_decode_value(reader).map_err(Error::from))
                .and_then(|s| usize::from_str(&s).map_err(Error::from))
                .map(Source::Ref)
        }
    }
}
