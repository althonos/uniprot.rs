use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;
use crate::parser::utils::decode_attribute;

use super::db_reference::DbReference;

#[derive(Debug, Clone)]
/// Describes a citation and a summary of its content.
pub struct Reference {
    pub key: usize,
    pub citation: Citation,
    pub evidences: Vec<usize>,
    pub scope: Vec<String>,
    pub sources: Vec<Source>,
}

impl Reference {
    pub fn new(citation: Citation, key: usize) -> Self {
        Self {
            key,
            citation,
            evidences: Default::default(),
            scope: Default::default(),
            sources: Default::default(),
        }
    }
}

impl FromXml for Reference {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"reference");

        let mut sources = Vec::new();
        let mut scope = Vec::new();
        let mut optcit = None;

        parse_inner!{event, reader, buffer,
            b"scope" => {
                scope.push(reader.read_text(b"scope", buffer)?);
            },
            e @ b"citation" => {
                let citation = FromXml::from_xml(&e, reader, buffer)?;
                if let Some(_) = optcit.replace(citation) {
                    return Err(Error::DuplicateElement("citation", "reference"));
                }
            },
            e @ b"source" => {
                sources.extend(Vec::<Source>::from_xml(&e, reader, buffer)?);
            }
        }

        let citation = optcit.ok_or(Error::MissingAttribute("citation", "reference"))?;
        let mut reference = Reference::new(citation, 0);

        let attr = attributes_to_hashmap(event)?;
        reference.evidences = get_evidences(reader, &attr)?;
        reference.key = attr.get(&b"key"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .ok_or(Error::MissingAttribute("key", "reference"))?
            .map(|x| usize::from_str(&x))??;

        Ok(reference)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes a single citation.
pub struct Citation {
    // attributes
    pub ty: CitationType,
    // date: Option<NaiveDate>,
    pub name: Option<String>,
    pub volume: Option<String>,
    pub first: Option<String>,
    pub last: Option<String>,
    pub publisher: Option<String>,
    pub city: Option<String>,
    pub db: Option<String>,
    pub number: Option<String>,

    // fields
    /// Describes the title of a citation.
    pub titles: Vec<String>,
    /// Describes the editors of a book (only used for books).
    pub editors: Vec<Creator>,
    /// Describes the authors of a citation.
    pub authors: Vec<Creator>,
    /// Describes the location (URL) of an online journal article
    pub locators: Vec<String>,
    /// Describes cross-references to bibliography databases (MEDLINE, PubMed,
    /// AGRICOLA) or other online resources (DOI).
    pub db_references: Vec<DbReference>,
}

impl Citation {
    pub fn new(ty: CitationType) -> Self {
        Self {
            ty,
            name: None,
            volume: None,
            first: None,
            last: None,
            publisher: None,
            city: None,
            db: None,
            number: None,
            titles: Vec::new(),
            editors: Vec::new(),
            authors: Vec::new(),
            locators: Vec::new(),
            db_references: Vec::new(),
        }
    }
}

impl FromXml for Citation {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"citation");

        use self::CitationType::*;
        use self::Creator::*;

        // extract attributes
        let attr = attributes_to_hashmap(event)?;

        // get citation type
        let ty = decode_attribute(event, reader, "type", "citation")?;

        // create the citation
        let mut citation = Citation::new(ty);

        // update attributes on citation (TODO)
        // citation.date = attr.get(&b"date"[..])
        //     .map(|v| v.unescape_and_decode_value(&mut self.xml))
        //     .transpose()?;
        citation.name = attr.get(&b"name"[..])
            .map(|v| v.unescape_and_decode_value(reader))
            .transpose()?;

        // update citation with children elements
        parse_inner!{event, reader, buffer,
            e @ b"authorList" => {
                parse_inner!{e, reader, buffer,
                    b"person" => {
                        let p = reader.read_text(b"person", buffer)
                            .map(Person)?;
                        citation.authors.push(p);
                    },
                    b"consortium" => {
                        let c = reader.read_text(b"consortium", buffer)
                            .map(Consortium)?;
                        citation.authors.push(c);
                    }
                }
            },
            e @ b"editorList" => {
                parse_inner!{e, reader, buffer,
                    b"person" => {
                        let p = reader.read_text(b"person", buffer)
                            .map(Person)?;
                        citation.editors.push(p);
                    },
                    b"consortium" => {
                        let c = reader.read_text(b"consortium", buffer)
                            .map(Consortium)?;
                        citation.editors.push(c);
                    }
                }
            },
            b"title" => {
                citation.titles.push(reader.read_text(b"title", buffer)?);
            },
            b"locator" => {
                citation.locators.push(reader.read_text(b"locator", buffer)?);
            },
            e @ b"dbReference" => {
                citation.db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(citation)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum CitationType {
    Book,
    JournalArticle,
    OnlineJournalArticle,
    Patent,
    Submission,
    Thesis,
    UnpublishedObservations,
}

impl FromStr for CitationType {
    type Err = crate::error::InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::CitationType::*;
        match s {
            "book" => Ok(Book),
            "journal article" => Ok(JournalArticle),
            "online journal article" => Ok(OnlineJournalArticle),
            "patent" => Ok(Patent),
            "submission" => Ok(Submission),
            "thesis" => Ok(Thesis),
            "unpublished observations" => Ok(UnpublishedObservations),
            other => Err(InvalidValue(String::from(other))),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Creator {
    /// Describes the author of a citation when these are represented by a consortium.
    Consortium(String),
    /// Describes the author of a citation when they are an individual.
    Person(String),
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes the source of the sequence according to the citation.
pub struct Source {
    pub value: String,
    pub ty: SourceType,
    pub evidences: Vec<usize>,
}

impl Source {
    pub fn new(value: String, ty: SourceType) -> Self {
        Self::with_evidences(value, ty, Vec::new())
    }

    pub fn with_evidences(value: String, ty: SourceType, evidences: Vec<usize>) -> Self {
        Self {
            value,
            ty,
            evidences
        }
    }
}

impl FromXml for Vec<Source> {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"source");

        use self::SourceType::*;

        let mut sources = Vec::new();
        parse_inner!{event, reader, buffer,
            e @ b"strain" => {
                let value = reader.read_text(b"strain", buffer)?;
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Strain, evidences));
            },
            e @ b"plasmid" => {
                let value = reader.read_text(b"plasmid", buffer)?;
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Plasmid, evidences));
            },
            e @ b"transposon" => {
                let value = reader.read_text(b"transposon", buffer)?;
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Transposon, evidences));
            },
            e @ b"tissue" => {
                let value = reader.read_text(b"tissue", buffer)?;
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Tissue, evidences));
            }
        }

        Ok(sources)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum SourceType {
    Strain,
    Plasmid,
    Transposon,
    Tissue,
}
