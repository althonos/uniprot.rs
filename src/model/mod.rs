pub mod comment;
pub mod db_reference;
pub mod feature;
pub mod feature_location;
pub mod gene;
pub mod gene_location;
pub mod keyword;
pub mod molecule;
pub mod organism;
pub mod property;
pub mod protein;
pub mod reference;
pub mod sequence;

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
use self::db_reference::DbReference;
use self::sequence::Sequence;
use self::feature::Feature;
use self::gene::Gene;
use self::gene_location::GeneLocation;
use self::organism::Organism;
use self::reference::Reference;
use self::protein::Protein;
use self::protein::ProteinExistence;
use self::keyword::Keyword;

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
    // pub evidences: Vec<Evidence>,
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
        }
    }

    pub(crate) fn from_xml_ignoring<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
        ignores: &HashSet<Bytes>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"entry");

        let attr = attributes_to_hashmap(event)?;
        let dataset = match attr.get(&b"dataset"[..]).map(|a| &*a.value) {
            Some(b"Swiss-Prot") => Dataset::SwissProt,
            Some(b"TrEMBL") => Dataset::TrEmbl,
            Some(other) => panic!("ERR: invalid value for `dataset` attribute of `entry`: {:?}", other),
            None => panic!("ERR: missing required `dataset` attribute of `entry`"),
        };

        let mut entry = Entry::new(dataset);
        parse_inner_ignoring!{event, reader, buffer, ignores,
            b"accession" => {
                entry.accessions.push(reader.read_text(b"accession", buffer)?);
            },
            b"name" => {
                entry.names.push(reader.read_text(b"name", buffer)?);
            },
            e @ b"protein" => {
                entry.protein = FromXml::from_xml(&e, reader, buffer)?
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
            b"evidence" => {
                // println!("TODO `evidence` in `entry`");
                reader.read_to_end(b"evidence", buffer)?;
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

impl FromXml for Entry {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        Self::from_xml_ignoring(event, reader, buffer, &Default::default())
    }
}


#[derive(Debug, Clone)]
/// The differents datasets an `Entry` can be part of.
pub enum Dataset {
    SwissProt,
    TrEmbl,
}
