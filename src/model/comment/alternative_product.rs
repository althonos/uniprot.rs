use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

#[derive(Debug, Default, Clone)]
pub struct AlternativeProduct {
    pub events: Vec<Event>,
    pub isoforms: Vec<Isoform>,
}

#[derive(Debug, Clone)]
pub enum Event {
    AlternativeSplicing,
    AlternativeInitiation,
    AlternativePromoter,
    RibosomalFrameshifting,
}

impl FromXml for Event {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        _reader: &mut Reader<B>,
        _buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"event");

        match event.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"type").unwrap_or_default())
            .transpose()?
            .as_ref()
            .map(|a| &*a.value)
        {
            Some(b"alternative splicing") => Ok(Event::AlternativeSplicing),
            Some(b"alternative initiation") => Ok(Event::AlternativeInitiation),
            Some(b"alternative promoter") => Ok(Event::AlternativePromoter),
            Some(b"ribosomal frameshifting") => Ok(Event::RibosomalFrameshifting),
            None => return Err(Error::MissingAttribute("type", "event")),
            Some(other) => return Err(
                Error::invalid_value("type", "event", String::from_utf8_lossy(other))
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct Isoform {
    pub ids: Vec<String>,
    pub names: Vec<String>,
    pub sequence: IsoformSequence,
    pub texts: Vec<String>,
}

impl Isoform {
    pub fn new(sequence: IsoformSequence) -> Self {
        Self {
            ids: Default::default(),
            names: Default::default(),
            sequence,
            texts: Default::default(),
        }
    }
}

impl FromXml for Isoform {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"isoform");

        let mut ids = Vec::new();
        let mut names = Vec::new();
        let mut texts = Vec::new();
        let mut optseq: Option<IsoformSequence> = None;

        parse_inner!{event, reader, buffer,
            b"id" => {
                ids.push(reader.read_text(b"id", buffer)?);
            },
            b"name" => {
                names.push(reader.read_text(b"name", buffer)?);
            },
            b"text" => {
                texts.push(reader.read_text(b"text", buffer)?);
            },
            e @ b"sequence" => {
                let seq = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = optseq.replace(seq) {
                    return Err(Error::DuplicateElement("sequence", "isoform"));
                }
            }
        }

        let mut isoform = optseq
            .map(Isoform::new)
            .ok_or(Error::MissingElement("sequence", "isoform"))?;
        isoform.names = names;
        isoform.ids = ids;
        isoform.texts = texts;

        Ok(isoform)
    }
}

#[derive(Debug, Clone)]
pub struct IsoformSequence {
    pub ty: IsoformSequenceType,
    pub reference: Option<String>,
}

impl IsoformSequence {
    pub fn new(ty: IsoformSequenceType) -> Self {
        Self::with_reference(ty, None)
    }

    pub fn with_reference<R>(ty: IsoformSequenceType, reference: R) -> Self
    where
        R: Into<Option<String>>
    {
        Self {
            ty,
            reference: reference.into()
        }
    }
}

impl FromXml for IsoformSequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"sequence");

        use self::IsoformSequenceType::*;

        let attr = attributes_to_hashmap(event)?;
        let mut seq = match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"not described") => IsoformSequence::new(NotDescribed),
            Some(b"described") => IsoformSequence::new(Described),
            Some(b"displayed") => IsoformSequence::new(Displayed),
            Some(b"external") => IsoformSequence::new(External),
            None => return Err(Error::MissingAttribute("type", "sequence")),
            Some(other) => return Err(
                Error::invalid_value("type", "sequence", String::from_utf8_lossy(other))
            ),
        };

        // extract optional reference
        seq.reference = attr.get(&b"ref"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?;

        // read to end
        reader.read_to_end(b"sequence", buffer)?;
        Ok(seq)
    }
}

#[derive(Debug, Clone)]
pub enum IsoformSequenceType {
    NotDescribed,
    Described,
    Displayed,
    External
}
