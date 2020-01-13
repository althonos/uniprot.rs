use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

#[derive(Debug, Default, Clone)]
/// Describes the subcellular location and optionally the topology and orientation of a molecule.
pub struct SubcellularLocation {
    pub locations: Vec<String>, // TODO: EvidenceString, minOccurs = "1"
    pub topologies: Vec<String>, // TODO: EvidenceString,
    pub orientations: Vec<String>, // TODO: EvidenceString,
}

impl FromXml for SubcellularLocation {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"subcellularLocation");

        let mut subloc = SubcellularLocation::default();
        parse_inner!{event, reader, buffer,
            e @ b"location" => {
                subloc.locations.push(reader.read_text(b"location", buffer)?);
            },
            e @ b"topology" => {
                subloc.topologies.push(reader.read_text(b"topology", buffer)?);
            },
            e @ b"orientation" => {
                subloc.orientations.push(reader.read_text(b"orientation", buffer)?);
            }
        }

        if subloc.locations.is_empty() {
            panic!("ERR: missing required `location` in `subcellularLocation`");
        }

        Ok(subloc)
    }
}
