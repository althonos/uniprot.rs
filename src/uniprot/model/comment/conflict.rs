use std::borrow::Cow;
use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::FromXml;

#[derive(Debug, Clone)]
pub struct Conflict {
    pub ty: ConflictType,
    pub reference: Option<String>,
    pub sequence: Option<ConflictSequence>,
}

impl Conflict {
    pub fn new(ty: ConflictType) -> Self {
        Self {
            ty,
            reference: Default::default(),
            sequence: Default::default(),
        }
    }
}

impl FromXml for Conflict {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"conflict");

        // create a new `Conflict` with the right type
        let mut conflict =
            decode_attribute(event, reader, "type", "conflict").map(Conflict::new)?;

        // extract optional reference
        conflict.reference = extract_attribute(event, "type")?
            .map(|x| x.decode_and_unescape_value(reader))
            .transpose()?
            .map(Cow::into_owned);

        // extract `sequence` element
        parse_inner! {event, reader, buffer,
            e @ b"sequence" => {
                let sequence = FromXml::from_xml(&e, reader, buffer)?;
                if conflict.sequence.replace(sequence).is_some() {
                    return Err(Error::DuplicateElement("sequence", "conflict"));
                }
            }
        }

        Ok(conflict)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictType {
    Frameshift,
    ErroneousInitiation,
    ErroneousTermination,
    ErroneousGeneModelPrediction,
    ErroneousTranslation,
    MiscellaneousDiscrepancy,
}

impl FromStr for ConflictType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ConflictType::*;
        match s {
            "frameshift" => Ok(Frameshift),
            "erroneous initiation" => Ok(ErroneousInitiation),
            "erroneous termination" => Ok(ErroneousTermination),
            "erroneous gene model prediction" => Ok(ErroneousGeneModelPrediction),
            "erroneous translation" => Ok(ErroneousTranslation),
            "miscellaneous discrepancy" => Ok(MiscellaneousDiscrepancy),
            other => Err(InvalidValue::from(other)),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ConflictSequence {
    pub id: String,
    pub resource: Resource,
    pub version: Option<usize>,
}

impl ConflictSequence {
    pub fn new(id: String, resource: Resource) -> Self {
        Self::with_version(id, resource, None)
    }

    pub fn with_version<N>(id: String, resource: Resource, version: N) -> Self
    where
        N: Into<Option<usize>>,
    {
        Self {
            id,
            resource,
            version: version.into(),
        }
    }
}

impl FromXml for ConflictSequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"sequence");

        let id = extract_attribute(event, "id")?
            .ok_or(Error::MissingAttribute("id", "sequence"))?
            .decode_and_unescape_value(reader)
            .map(Cow::into_owned)?;
        let version = extract_attribute(event, "version")?
            .map(|x| x.decode_and_unescape_value(reader))
            .transpose()?
            .map(|s| usize::from_str(&s))
            .transpose()?;
        let res = decode_attribute(event, reader, "resource", "sequence")?;

        reader.read_to_end_into(event.name(), buffer)?;
        Ok(ConflictSequence::with_version(id, res, version))
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Embl,
    EmblCds,
}

impl FromStr for Resource {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EMBL" => Ok(Resource::Embl),
            "EMBL-CDS" => Ok(Resource::EmblCds),
            other => Err(InvalidValue::from(other)),
        }
    }
}
