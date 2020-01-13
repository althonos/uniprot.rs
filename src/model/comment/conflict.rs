use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

#[derive(Debug, Clone)]
pub struct Conflict {
    pub ty: ConflictType,
    pub reference: Option<String>,
    pub sequence: Option<ConflictSequence>
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
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"conflict");

        use self::ConflictType::*;

        let attr = attributes_to_hashmap(event)?;
        let mut conflict = match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"frameshift") => Conflict::new(Frameshift),
            Some(b"erroneous initiation") => Conflict::new(ErroneousInitiation),
            Some(b"erroneous termination") => Conflict::new(ErroneousTermination),
            Some(b"erroneous gene model prediction") => Conflict::new(ErroneousGeneModelPrediction),
            Some(b"erroneous translation") => Conflict::new(ErroneousTranslation),
            Some(b"miscellaneous discrepancy") => Conflict::new(MiscellaneousDiscrepancy),
            Some(other) => panic!("ERR: invalid `type` in `conflict`: {:?}", other),
            None => panic!("ERR: missing required `type` in `conflict`"),
        };

        // extract optional reference
        conflict.reference = attr.get(&b"ref"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?;

        // extract `sequence` element
        parse_inner!{event, reader, buffer,
            e @ b"sequence" => {
                let sequence = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = conflict.sequence.replace(sequence) {
                    panic!("ERR: duplicate `sequence` in `conflict`");
                }
            }
        }

        Ok(conflict)
    }
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    Frameshift,
    ErroneousInitiation,
    ErroneousTermination,
    ErroneousGeneModelPrediction,
    ErroneousTranslation,
    MiscellaneousDiscrepancy
}

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
        N: Into<Option<usize>>
    {
        Self {
            id,
            resource,
            version: version.into()
        }
    }
}

impl FromXml for ConflictSequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"sequence");

        let attr = attributes_to_hashmap(event)?;
        let id = attr.get(&b"id"[..])
            .expect("ERR: could not find required `id` attr on `sequence`")
            .unescape_and_decode_value(reader)?;
        let version = attr.get(&b"version"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|s| usize::from_str(&s))
            .transpose()
            .expect("ERR: could not decode `version` as usize");
        let resource = match attr.get(&b"resource"[..]).map(|a| &*a.value) {
            Some(b"EMBL") => Resource::Embl,
            Some(b"EMBL-CDS") => Resource::EmblCds,
            Some(other) => panic!("ERR: invalid `resource` in `sequence`: {:?}", other),
            None => panic!("ERR: missing required `resource` in `sequence`"),
        };

        reader.read_to_end(b"sequence", &mut Vec::new());
        Ok(ConflictSequence::with_version(id, resource, version))
    }
}

#[derive(Debug, Clone)]
pub enum Resource {
    Embl,
    EmblCds,
}
