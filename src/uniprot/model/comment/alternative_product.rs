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

#[derive(Debug, Default, Clone)]
pub struct AlternativeProduct {
    pub events: Vec<Event>,
    pub isoforms: Vec<Isoform>,
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    AlternativeSplicing,
    AlternativeInitiation,
    AlternativePromoter,
    RibosomalFrameshifting,
}

impl FromStr for Event {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Event::*;
        match s {
            "alternative splicing" => Ok(AlternativeSplicing),
            "alternative initiation" => Ok(AlternativeInitiation),
            "alternative promoter" => Ok(AlternativePromoter),
            "ribosomal frameshifting" => Ok(RibosomalFrameshifting),
            other => Err(InvalidValue::from(other)),
        }
    }
}

impl FromXml for Event {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"event");
        reader.read_to_end_into(event.name(), buffer)?;
        decode_attribute(event, reader, "type", "event")
    }
}

// ---------------------------------------------------------------------------

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
        debug_assert_eq!(event.local_name().as_ref(), b"isoform");

        let mut ids = Vec::new();
        let mut names = Vec::new();
        let mut texts = Vec::new();
        let mut optseq: Option<IsoformSequence> = None;

        parse_inner! {event, reader, buffer,
            e @ b"id" => {
                ids.push(parse_text!(e, reader, buffer));
            },
            e @ b"name" => {
                names.push(parse_text!(e, reader, buffer));
            },
            e @ b"text" => {
                texts.push(parse_text!(e, reader, buffer));
            },
            e @ b"sequence" => {
                let seq = FromXml::from_xml(&e, reader, buffer)?;
                if optseq.replace(seq).is_some() {
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

// ---------------------------------------------------------------------------

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
        R: Into<Option<String>>,
    {
        Self {
            ty,
            reference: reference.into(),
        }
    }
}

impl FromXml for IsoformSequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"sequence");

        reader.read_to_end_into(event.name(), buffer)?;

        let reference = extract_attribute(event, "ref")?
            .map(|x| x.decode_and_unescape_value(reader))
            .transpose()?
            .map(Cow::into_owned);
        decode_attribute(event, reader, "type", "sequence")
            .map(|ty| Self::with_reference(ty, reference))
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsoformSequenceType {
    NotDescribed,
    Described,
    Displayed,
    External,
}

impl FromStr for IsoformSequenceType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::IsoformSequenceType::*;
        match s {
            "not described" => Ok(NotDescribed),
            "described" => Ok(Described),
            "displayed" => Ok(Displayed),
            "external" => Ok(External),
            other => Err(InvalidValue::from(other)),
        }
    }
}
