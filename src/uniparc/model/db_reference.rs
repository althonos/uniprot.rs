use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::decode_opt_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::Date;
use super::Property;

#[derive(Debug, Clone)]
pub struct DbReference {
    // attributes
    pub ty: ShortString,
    pub id: ShortString,
    pub version_i: usize,
    pub active: ShortString,
    pub version: Option<usize>,
    pub created: Option<Date>,
    pub last: Option<Date>,
    // fields
    pub properties: Vec<Property>,
}

impl FromXml for DbReference {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"dbReference");

        let version_i = decode_attribute(event, reader, "version_i", "dbReference")?;
        let version = decode_opt_attribute(event, reader, "version", "dbReference")?;
        let created = decode_opt_attribute(event, reader, "created", "dbReference")?;
        let last = decode_opt_attribute(event, reader, "last", "dbReference")?;

        let ty = extract_attribute(event, "type")?
            .ok_or(Error::MissingAttribute("type", "dbReference"))?
            .decode_and_unescape_value(reader)
            .map(ShortString::from)?;
        let id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "dbReference"))?
            .decode_and_unescape_value(reader)
            .map(ShortString::from)?;
        let active = extract_attribute(event, "active")?
            .ok_or(Error::MissingAttribute("active", "dbReference"))?
            .decode_and_unescape_value(reader)
            .map(ShortString::from)?;

        let mut properties = Vec::new();
        parse_inner! {event, reader, buffer,
            e @ b"property" => {
                properties.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(DbReference {
            ty,
            id,
            version_i,
            version,
            active,
            created,
            last,
            properties,
        })
    }
}
