use std::borrow::Cow;
use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::extract_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::db_reference::DbReference;

#[derive(Debug, Clone)]
/// A citation, also contain a summary of its content.
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
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"reference");

        let mut sources = Vec::new();
        let mut scope = Vec::new();
        let mut optcit = None;

        parse_inner! {event, reader, buffer,
            e @ b"scope" => {
                scope.push(parse_text!(e, reader, buffer));
            },
            e @ b"citation" => {
                let citation = FromXml::from_xml(&e, reader, buffer)?;
                if optcit.replace(citation).is_some() {
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
        reference.key = attr
            .get(&b"key"[..])
            .map(|a| a.decode_and_unescape_value(reader))
            .ok_or(Error::MissingAttribute("key", "reference"))?
            .map(|x| usize::from_str(&x))??;

        Ok(reference)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// A single citation.
pub struct Citation {
    // attributes
    /// Describe the type of this citation.
    pub ty: CitationType,
    // date: Option<NaiveDate>,
    /// Describes the name of an (online) journal or book.
    pub name: Option<String>,
    /// Describes the volume of a journal or book.
    pub volume: Option<String>,
    /// Describes the first page of an article.
    pub first: Option<String>,
    /// Describes the last page of an article.
    pub last: Option<String>,
    /// Describes the publisher of a book.
    pub publisher: Option<String>,
    /// Describes the city where a book was published.
    pub city: Option<String>,
    /// Describes the database name of submissions.
    pub db: Option<String>,
    /// Describes a patent number.
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
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"citation");

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
        //     .map(|v| v.decode_and_unescape_value(&mut self.xml))
        //     .transpose()?;
        citation.name = attr
            .get(&b"name"[..])
            .map(|v| v.decode_and_unescape_value(reader))
            .transpose()?
            .map(Cow::into_owned);
        citation.db = attr
            .get(&b"db"[..])
            .map(|v| v.decode_and_unescape_value(reader))
            .transpose()?
            .map(Cow::into_owned);

        // update citation with children elements
        parse_inner! {event, reader, buffer,
            e @ b"authorList" => {
                parse_inner!{e, reader, buffer,
                    x @ b"person" => {
                        reader.read_to_end_into(x.name(), buffer)?;
                        let name = extract_attribute(&x, "name")?
                            .ok_or(Error::MissingAttribute("name", "person"))?
                            .decode_and_unescape_value(reader)?
                            .into_owned();
                        citation.authors.push(Person(name));
                    },
                    x @ b"consortium" => {
                        reader.read_to_end_into(x.name(), buffer)?;
                        let name = extract_attribute(&x, "name")?
                            .ok_or(Error::MissingAttribute("name", "consortium"))?
                            .decode_and_unescape_value(reader)?
                            .into_owned();
                        citation.authors.push(Consortium(name));
                    }
                }
            },
            e @ b"editorList" => {
                parse_inner!{e, reader, buffer,
                    x @ b"person" => {
                        reader.read_to_end_into(x.name(), buffer)?;
                        let name = extract_attribute(&x, "name")?
                            .ok_or(Error::MissingAttribute("name", "person"))?
                            .decode_and_unescape_value(reader)?
                            .into_owned();
                        citation.authors.push(Person(name));
                    },
                    x @ b"consortium" => {
                        reader.read_to_end_into(x.name(), buffer)?;
                        let name = extract_attribute(&x, "name")?
                            .ok_or(Error::MissingAttribute("name", "consortium"))?
                            .decode_and_unescape_value(reader)?
                            .into_owned();
                        citation.authors.push(Consortium(name));
                    }
                }
            },
            e @ b"title" => {
                citation.titles.push(parse_text!(e, reader, buffer));
            },
            e @ b"locator" => {
                citation.locators.push(parse_text!(e, reader, buffer));
            },
            e @ b"dbReference" => {
                citation.db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(citation)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The type of a citation.
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
/// A single author in a citation.
pub enum Creator {
    /// The author of a citation when these are represented by a consortium.
    Consortium(String),
    /// The author of a citation when they are an individual.
    Person(String),
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// The source of the protein sequence according to the citation.
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
            evidences,
        }
    }
}

impl FromXml for Vec<Source> {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name().as_ref(), b"source");

        use self::SourceType::*;

        let mut sources = Vec::new();
        parse_inner! {event, reader, buffer,
            e @ b"strain" => {
                let value = parse_text!(e, reader, buffer);
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Strain, evidences));
            },
            e @ b"plasmid" => {
                let value = parse_text!(e, reader, buffer);
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Plasmid, evidences));
            },
            e @ b"transposon" => {
                let value = parse_text!(e, reader, buffer);
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Transposon, evidences));
            },
            e @ b"tissue" => {
                let value = parse_text!(e, reader, buffer);
                let evidences = attributes_to_hashmap(&e)
                    .and_then(|a| get_evidences(reader, &a))?;
                sources.push(Source::with_evidences(value, Tissue, evidences));
            }
        }

        Ok(sources)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The kind of sources where a sequence can originate from.
pub enum SourceType {
    Strain,
    Plasmid,
    Transposon,
    Tissue,
}
