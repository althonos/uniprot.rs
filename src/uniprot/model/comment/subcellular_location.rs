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
        debug_assert_eq!(event.local_name(), b"subcellularLocation");

        let mut subloc = SubcellularLocation::default();
        parse_inner! {event, reader, buffer,
            b"location" => {
                subloc.locations.push(reader.read_text(b"location", buffer)?);
            },
            b"topology" => {
                subloc.topologies.push(reader.read_text(b"topology", buffer)?);
            },
            b"orientation" => {
                subloc.orientations.push(reader.read_text(b"orientation", buffer)?);
            }
        }

        if subloc.locations.is_empty() {
            return Err(Error::MissingElement("location", "SubcellularLocation"));
        }

        Ok(subloc)
    }
}
