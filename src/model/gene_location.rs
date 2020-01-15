use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;

#[derive(Debug, Clone)]
/// Describes non-nuclear gene locations (organelles and plasmids).
pub struct GeneLocation {
    // name: Vec<Status>,
    pub ty: LocationType,
    pub evidences: Vec<usize>,
    pub names: Vec<LocationName>,
}

impl GeneLocation {
    pub fn new(ty: LocationType) -> Self {
        Self {
            ty,
            evidences: Default::default(),
            names: Default::default(),
        }
    }
}

impl FromXml for GeneLocation {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"geneLocation");

        use self::LocationType::*;

        let attr = attributes_to_hashmap(event)?;
        let loctype = match attr.get(&b"type"[..]).map(|a| &*a.value) {
            Some(b"apicoplast") => Apicoplast,
            Some(b"chloroplast") => Chloroplast,
            Some(b"organellar chromatophore") => OrganellarChromatophore,
            Some(b"cyanelle") => Cyanelle,
            Some(b"hydrogenosome") => Hydrogenosome,
            Some(b"mitochondrion") => Mitochondrion,
            Some(b"non-photosynthetic plastid") => NonPhotosyntheticPlasmid,
            Some(b"nucleomorph") => Nucleomorph,
            Some(b"plasmid") => Plasmid,
            Some(b"plastid") => Plastid,
            None => return Err(Error::MissingAttribute("type", "geneLocation")),
            Some(other) => return Err(
                Error::invalid_value("type", "geneLocation", String::from_utf8_lossy(other))
            )
        };

        let mut geneloc = Self::new(loctype);
        geneloc.evidences = get_evidences(reader, &attr)?;
        parse_inner!{event, reader, buffer,
            e @ b"name" => {
                geneloc.names.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(geneloc)
    }
}

#[derive(Debug, Clone)]
pub enum LocationType {
    Apicoplast,
    Chloroplast,
    OrganellarChromatophore,
    Cyanelle,
    Hydrogenosome,
    Mitochondrion,
    NonPhotosyntheticPlasmid,
    Nucleomorph,
    Plasmid,
    Plastid,
}

#[derive(Debug, Clone)]
pub struct LocationName {
    pub value: String,
    pub status: LocationStatus
}

impl LocationName {
    /// Create a new `LocationName` with `status` set to `Known`.
    pub fn new(value: String) -> Self {
        Self::with_status(value, Default::default())
    }

    /// Create a new `LocationName` with a given status.
    pub fn with_status(value: String, status: LocationStatus) -> Self {
        Self {
            value,
            status
        }
    }
}

impl FromXml for LocationName {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"name");

        let value = reader.read_text(b"name", buffer)?;
        let status = match extract_attribute(event, "status")?
            .as_ref()
            .map(|a| &*a.value)
        {
            Some(b"known") => LocationStatus::Known,
            Some(b"unknown") => LocationStatus::Unknown,
            None => LocationStatus::default(),
            Some(other) => return Err(
                Error::invalid_value("status", "name", String::from_utf8_lossy(other))
            )
        };

        Ok(Self::with_status(value, status))
    }
}

#[derive(Debug, Clone)]
pub enum LocationStatus {
    Known,
    Unknown,
}

impl Default for LocationStatus {
    fn default() -> Self {
        LocationStatus::Known
    }
}
