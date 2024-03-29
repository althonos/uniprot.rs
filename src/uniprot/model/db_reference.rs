use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::property::Property;
use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

use super::molecule::Molecule;

#[derive(Debug, Default, Clone)]
/// A database cross-reference.
pub struct DbReference {
    pub molecule: Option<Molecule>,
    pub property: Vec<Property>,
    pub ty: ShortString,
    pub id: ShortString,
    pub evidences: Vec<usize>,
}

impl FromXml for DbReference {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"dbReference");

        let mut db_reference = DbReference::default();
        parse_inner! {event, reader, buffer,
            e @ b"property" => {
                db_reference.property.push(Property::from_xml(&e, reader, buffer)?);
            },
            e @ b"molecule" => {
                let molecule = Molecule::from_xml(&e, reader, buffer)?;
                if db_reference.molecule.replace(molecule).is_some() {
                    return Err(Error::DuplicateElement("molecule", "dbReference"))
                }
            }
        }

        // let attr = attributes_to_hashmap(event)?;
        db_reference.ty = extract_attribute(event, "type")?
            .ok_or(Error::MissingAttribute("type", "dbReference"))?
            .decode_and_unescape_value(reader)?
            .into();
        db_reference.id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "dbReference"))?
            .decode_and_unescape_value(reader)?
            .into();

        Ok(db_reference)
    }
}
