use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

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
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"geneLocation");

        let mut geneloc = decode_attribute(event, reader, "type", "geneLocation").map(Self::new)?;

        geneloc.evidences = get_evidences(reader, &event)?;
        parse_inner! {event, reader, buffer,
            e @ b"name" => {
                geneloc.names.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(geneloc)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl FromStr for LocationType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::LocationType::*;
        match s {
            "apicoplast" => Ok(Apicoplast),
            "chloroplast" => Ok(Chloroplast),
            "organellar chromatophore" => Ok(OrganellarChromatophore),
            "cyanelle" => Ok(Cyanelle),
            "hydrogenosome" => Ok(Hydrogenosome),
            "mitochondrion" => Ok(Mitochondrion),
            "non-photosynthetic plastid" => Ok(NonPhotosyntheticPlasmid),
            "nucleomorph" => Ok(Nucleomorph),
            "plasmid" => Ok(Plasmid),
            "plastid" => Ok(Plastid),
            other => Err(InvalidValue::from(other)),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LocationName {
    pub value: String,
    pub status: LocationStatus,
}

impl LocationName {
    /// Create a new `LocationName` with `status` set to `Known`.
    pub fn new(value: String) -> Self {
        Self::with_status(value, Default::default())
    }

    /// Create a new `LocationName` with a given status.
    pub fn with_status(value: String, status: LocationStatus) -> Self {
        Self { value, status }
    }
}

impl FromXml for LocationName {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"name");

        let value = parse_text!(event, reader, buffer);
        let status = match decode_attribute(event, reader, "status", "name") {
            Err(Error::MissingAttribute(_, _)) => Default::default(),
            Err(err) => return Err(err),
            Ok(status) => status,
        };

        Ok(Self::with_status(value, status))
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Indicates whether the name of a plasmid is known or unknown.
pub enum LocationStatus {
    Known,
    Unknown,
}

impl Default for LocationStatus {
    fn default() -> Self {
        LocationStatus::Known
    }
}

impl FromStr for LocationStatus {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "known" => Ok(LocationStatus::Known),
            "unknown" => Ok(LocationStatus::Unknown),
            other => Err(InvalidValue::from(other)),
        }
    }
}
