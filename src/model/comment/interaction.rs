use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

#[derive(Debug, Clone)]
pub struct Interaction {
    pub interactants: (Interactant, Interactant),
    pub organisms_differ: bool,
    pub experiments: usize,
}

#[derive(Debug, Clone)]
pub struct Interactant {
    pub interactant_id: String,
    pub id: Option<String>,
    pub label: Option<String>,
}

impl Interactant {
    pub fn new(interactant_id: String) -> Self {
        Self {
            interactant_id,
            id: Default::default(),
            label: Default::default(),
        }
    }
}

impl FromXml for Interactant {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"interactant");

        let mut interactant = event.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"intactId").unwrap_or_default())
            .ok_or(Error::MissingAttribute("intactId", "Interactant"))?
            .and_then(|a| a.unescape_and_decode_value(reader).map(Interactant::new))?;

        parse_inner!{event, reader, buffer,
            b"id" => {
                let id = reader.read_text(b"id", buffer)?;
                if let Some(_) = interactant.id.replace(id) {
                    return Err(Error::DuplicateElement("id", "interaction"));
                }
            },
            b"label" => {
                let label = reader.read_text(b"label", buffer)?;
                if let Some(_) = interactant.label.replace(label) {
                    return Err(Error::DuplicateElement("label", "interaction"));
                }
            }
        }

        Ok(interactant)
    }
}
