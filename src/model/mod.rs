//! Object model of the UniprotKB XML database.
//!
//! The model was derived from the latest [Uniprot XML schema], with some
//! small changes to make the type hierarchy fit with the Rust conventions.
//!
//! [Uniprot XML schema]: https://www.uniprot.org/docs/uniprot.xsd

pub mod comment;
pub mod evidence;
pub mod feature;
pub mod feature_location;
pub mod gene;
pub mod gene_location;
pub mod organism;
pub mod protein;
pub mod reference;
pub mod sequence;

mod db_reference;
mod keyword;
mod molecule;
mod property;

pub use self::db_reference::DbReference;
pub use self::keyword::Keyword;
pub use self::property::Property;
pub use self::molecule::Molecule;

use std::collections::HashSet;
use std::io::BufRead;
use std::str::FromStr;

use bytes::Bytes;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;

use self::comment::Comment;
use self::evidence::Evidence;
use self::sequence::Sequence;
use self::feature::Feature;
use self::gene::Gene;
use self::gene_location::GeneLocation;
use self::organism::Organism;
use self::reference::Reference;
use self::protein::Protein;
use self::protein::ProteinExistence;

#[derive(Debug, Clone)]
/// A UniProtKB entry.
pub struct Entry {

    // attributes
    pub dataset: Dataset,
    // created: NaiveDate,
    // modified: NaiveDate,
    // version: usize,

    // fields
    pub accessions: Vec<String>,  // minOccurs = 1
    pub names: Vec<String>,       // minOccurs = 1
    pub protein: Protein,
    pub genes: Vec<Gene>,
    pub organism: Organism,
    pub organism_hosts: Vec<Organism>,
    pub gene_location: Vec<GeneLocation>,
    pub references: Vec<Reference>,  // minOccurs = 1
    pub comments: Vec<Comment>,      // nillable
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
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"entry");

        let attr = attributes_to_hashmap(event)?;
        let dataset = match attr.get(&b"dataset"[..]).map(|a| &*a.value) {
            Some(b"Swiss-Prot") => Dataset::SwissProt,
            Some(b"TrEMBL") => Dataset::TrEmbl,
            None => return Err(Error::MissingAttribute("dataset", "entry")),
            Some(other) => return Err(Error::invalid_value("dataset", "entry", String::from_utf8_lossy(other)))
        };

        let mut entry = Entry::new(dataset);
        parse_inner!{event, reader, buffer,
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

#[derive(Debug, Clone)]
/// The differents datasets an `Entry` can be part of.
pub enum Dataset {
    SwissProt,
    TrEmbl,
}
