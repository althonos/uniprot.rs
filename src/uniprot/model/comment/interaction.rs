use std::borrow::Cow;
use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::super::DbReference;

#[derive(Debug, Clone)]
pub struct Interaction {
    pub interactants: (Interactant, Interactant),
    pub organisms_differ: bool,
    pub experiments: usize,
}

#[derive(Debug, Clone)]
pub struct Interactant {
    pub interactant_id: ShortString,
    pub id: Option<ShortString>,
    pub label: Option<ShortString>,
    pub db_reference: Vec<DbReference>,
}

impl Interactant {
    pub fn new(interactant_id: ShortString) -> Self {
        Self {
            interactant_id,
            id: Default::default(),
            label: Default::default(),
            db_reference: Vec::new(),
        }
    }
}

impl FromXml for Interactant {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"interactant");

        let mut interactant = event
            .attributes()
            .find(|x| {
                x.is_err()
                    || x.as_ref()
                        .map(|a| a.key.as_ref() == b"intactId")
                        .unwrap_or_default()
            })
            .ok_or(Error::MissingAttribute("intactId", "Interactant"))?
            .map_err(Error::from)
            .and_then(|a| a.decode_and_unescape_value(reader).map_err(Error::from))
            .map(From::from)
            .map(Interactant::new)?;

        parse_inner! {event, reader, buffer,
            e @ b"id" => {
                let id = parse_text!(e, reader, buffer);
                if interactant.id.replace(id).is_some() {
                    return Err(Error::DuplicateElement("id", "interaction"));
                }
            },
            e @ b"label" => {
                let label = parse_text!(e, reader, buffer);
                if interactant.label.replace(label).is_some() {
                    return Err(Error::DuplicateElement("label", "interaction"));
                }
            },
            e @ b"dbReference" => {
                interactant.db_reference.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(interactant)
    }
}
