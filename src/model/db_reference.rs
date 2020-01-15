use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;

use super::molecule::Molecule;
use super::property::Property;

#[derive(Debug, Default, Clone)]
/// A database cross-reference.
pub struct DbReference {
    pub molecule: Option<Molecule>,
    pub property: Vec<Property>,
    pub ty: String,
    pub id: String,
    pub evidences: Vec<usize>,
}

impl FromXml for DbReference {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"dbReference");

        let mut db_reference = DbReference::default();
        parse_inner!{event, reader, buffer,
            e @ b"property" => {
                db_reference.property.push(Property::from_xml(&e, reader, buffer)?);
            },
            e @ b"molecule" => {
                let molecule = Molecule::from_xml(&e, reader, buffer)?;
                if let Some(_) = db_reference.molecule.replace(molecule) {
                    return Err(Error::DuplicateElement("molecule", "dbReference"))
                }
            }
        }

        let attr = attributes_to_hashmap(event)?;
        db_reference.ty = attr.get(&b"type"[..])
            .ok_or(Error::MissingAttribute("type", "dbReference"))?
            .unescape_and_decode_value(reader)?;
        db_reference.id = attr.get(&b"id"[..])
            .ok_or(Error::MissingAttribute("id", "dbReference"))?
            .unescape_and_decode_value(reader)?;

        Ok(db_reference)
    }
}
