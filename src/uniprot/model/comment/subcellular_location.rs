use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

#[derive(Debug, Default, Clone)]
/// The subcellular location (and optionally the topology and orientation) of a molecule.
pub struct SubcellularLocation {
    pub locations: Vec<String>,    // TODO: EvidenceString, minOccurs = "1"
    pub topologies: Vec<String>,   // TODO: EvidenceString,
    pub orientations: Vec<String>, // TODO: EvidenceString,
}

impl FromXml for SubcellularLocation {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"subcellularLocation");

        let mut subloc = SubcellularLocation::default();
        parse_inner! {event, reader, buffer,
            e @ b"location" => {
                subloc.locations.push(parse_text!(e, reader, buffer));
            },
            e @ b"topology" => {
                subloc.topologies.push(parse_text!(e, reader, buffer));
            },
            e @ b"orientation" => {
                subloc.orientations.push(parse_text!(e, reader, buffer));
            }
        }

        if subloc.locations.is_empty() {
            return Err(Error::MissingElement("location", "SubcellularLocation"));
        }

        Ok(subloc)
    }
}
