//! Object model of the UniprotKB XML database.
//!
//! The model was derived from the latest [Uniprot XML schema], with some
//! small changes to make the type hierarchy fit with the Rust conventions.
//!
//! [Uniprot XML schema]: https://www.uniprot.org/docs/uniprot.xsd

pub mod comment;
pub mod feature_location;
pub mod gene;
pub mod gene_location;
pub mod organism;
pub mod protein;
pub mod reference;

mod db_reference;
mod evidence;
mod feature;
mod keyword;
mod ligand;
mod ligand_part;
mod molecule;
mod sequence;

pub use self::db_reference::DbReference;
pub use self::evidence::Evidence;
pub use self::evidence::Source;
pub use self::feature::Feature;
pub use self::feature::FeatureType;
pub use self::keyword::Keyword;
pub use self::ligand::Ligand;
pub use self::ligand_part::LigandPart;
pub use self::molecule::Molecule;
pub use self::sequence::FragmentType;
pub use self::sequence::Sequence;
pub use crate::common::date::Date;
pub use crate::common::property::Property;

use std::collections::HashSet;
use std::io::BufRead;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::decode_attribute;
use crate::parser::FromXml;
use crate::parser::UniprotDatabase;

use self::comment::Comment;
use self::gene::Gene;
use self::gene_location::GeneLocation;
use self::organism::Organism;
use self::protein::Protein;
use self::protein::ProteinExistence;
use self::reference::Reference;

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// A UniProtKB entry.
pub struct Entry {
    // attributes
    pub dataset: Dataset,
    pub created: Date,
    pub modified: Date,
    pub version: usize,
    // fields
    pub accessions: Vec<String>, // minOccurs = 1
    pub names: Vec<String>,      // minOccurs = 1
    pub protein: Protein,
    pub genes: Vec<Gene>,
    pub organism: Organism,
    pub organism_hosts: Vec<Organism>,
    pub gene_location: Vec<GeneLocation>,
    pub references: Vec<Reference>, // minOccurs = 1
    pub comments: Vec<Comment>,     // nillable
    pub db_references: Vec<DbReference>,
    pub protein_existence: ProteinExistence,
    pub keywords: Vec<Keyword>,
    pub features: Vec<Feature>,
    pub evidences: Vec<Evidence>,
    pub sequence: Sequence,
}

impl Entry {
    pub fn new(dataset: Dataset) -> Self {
        Entry {
            dataset,
            created: Default::default(),
            modified: Default::default(),
            version: 1,
            accessions: Default::default(),
            names: Default::default(),
            protein: Default::default(),
            genes: Default::default(),
            organism: Default::default(),
            organism_hosts: Default::default(),
            gene_location: Default::default(),
            references: Default::default(),
            comments: Default::default(),
            db_references: Default::default(),
            protein_existence: Default::default(),
            keywords: Default::default(),
            features: Default::default(),
            sequence: Default::default(),
            evidences: Default::default(),
        }
    }
}

impl FromXml for Entry {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"entry");

        let attr = attributes_to_hashmap(event)?;
        let dataset = match attr.get(&b"dataset"[..]).map(|a| &*a.value) {
            Some(b"Swiss-Prot") => Dataset::SwissProt,
            Some(b"TrEMBL") => Dataset::TrEmbl,
            None => return Err(Error::MissingAttribute("dataset", "entry")),
            Some(other) => {
                return Err(Error::invalid_value(
                    "dataset",
                    "entry",
                    String::from_utf8_lossy(other),
                ))
            }
        };
        let mut entry = Entry::new(dataset);

        entry.modified = decode_attribute(event, reader, "modified", "entry")?;
        entry.created = decode_attribute(event, reader, "created", "entry")?;
        entry.version = decode_attribute(event, reader, "version", "entry")?;
        parse_inner! {event, reader, buffer,
            b"accession" => {
                entry.accessions.push(reader.read_text(b"accession", buffer)?);
            },
            b"name" => {
                entry.names.push(reader.read_text(b"name", buffer)?);
            },
            e @ b"protein" => {
                entry.protein = FromXml::from_xml(&e, reader, buffer)?;
            },
            e @ b"gene" => {
                entry.genes.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"organism" => {
                entry.organism = FromXml::from_xml(&e, reader, buffer)?;
            },
            e @ b"organismHost" => {
                entry.organism_hosts.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"reference" => {
                entry.references.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"comment" => {
                entry.comments.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"dbReference" => {
                entry.db_references.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"proteinExistence" => {
                entry.protein_existence = FromXml::from_xml(&e, reader, buffer)?;
            },
            e @ b"keyword" => {
                entry.keywords.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"feature" => {
                entry.features.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"evidence" => {
                entry.evidences.push(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"sequence" => {
                entry.sequence = Sequence::from_xml(&e, reader, buffer)?;
            },
            e @ b"geneLocation" => {
                entry.gene_location.push(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(entry)
    }
}

// ---------------------------------------------------------------------------

/// A UniProtKB database.
#[derive(Debug, Clone)]
pub struct UniProt {
    entries: Vec<Entry>,
}

impl UniProt {
    /// Create a new UniRef database with the given entries.
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl Deref for UniProt {
    type Target = Vec<Entry>;
    fn deref(&self) -> &Vec<Entry> {
        &self.entries
    }
}

impl DerefMut for UniProt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl FromIterator<Entry> for UniProt {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl From<Vec<Entry>> for UniProt {
    fn from(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl From<UniProt> for Vec<Entry> {
    fn from(uniprot: UniProt) -> Self {
        uniprot.entries
    }
}

impl UniprotDatabase for UniProt {
    type Entry = Entry;
    const ROOTS: &'static [&'static [u8]] = &[b"uniprot"];
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// The differents datasets an `Entry` can be part of.
pub enum Dataset {
    SwissProt,
    TrEmbl,
}
