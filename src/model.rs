// extern crate chrono;
//
// use chrono::Utc;
// use chrono::NaiveDate;
//
// /// Describes a collection of UniProtKB entries.
// struct Uniprot {
//     entries: Vec<Entry>,
//     copyright: Option<Copyright>,
// }
//
// type Copyright = String;
//

use url::Url;

// // ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes a UniProtKB entry.
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
    // evidence: Vec<Evidence>,
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
}

#[derive(Debug, Clone)]
pub enum Dataset {
    SwissProt,
    TrEmbl,
}

#[derive(Debug, Default, Clone)]
pub struct Sequence {
    pub value: String,
    pub length: usize,
    pub mass: usize,
    pub checksum: u64,
    // modified: NaiveDate,
    pub version: usize,
    pub precursor: Option<bool>,
    pub fragment: Option<FragmentType>
}

#[derive(Debug, Clone)]
pub enum FragmentType {
    Single,
    Multiple,
}

//
// // ---------------------------------------------------------------------------
//

#[derive(Debug, Clone, Default)]
/// Describes the names for the protein and parts thereof.
pub struct Protein {
    pub name: ProteinNameGroup,
    pub domains: Vec<ProteinNameGroup>,
    pub components: Vec<ProteinNameGroup>,
}

#[derive(Debug, Clone, Default)]
pub struct ProteinNameGroup {
    pub recommended: Option<ProteinName>,
    pub alternative: Vec<ProteinName>,
    pub submitted: Vec<ProteinName>,
    pub allergen: Option<String>,     // FIXME: type should be evidence text?
    pub biotech: Option<String>,
    pub cd_antigen: Vec<String>,
    pub inn: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProteinName {
    pub full: String,
    pub short: Vec<String>,
    pub ec_number: Vec<String>,
}

#[derive(Debug, Clone)]
/// Describes the evidence for the protein's existence.
pub enum ProteinExistence {
    ProteinLevelEvidence,
    TranscriptLevelEvidence,
    HomologyInferred,
    Predicted,
    Uncertain,
}

impl Default for ProteinExistence {
    fn default() -> Self {
        ProteinExistence::Uncertain
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
/// Describes a gene.
pub struct Gene {
    pub names: Vec<GeneName>,
}

#[derive(Debug, Clone)]
/// Describes different types of gene designations.
pub struct GeneName {
    pub value: String,
    pub ty: GeneNameType,
    pub evidence: Vec<usize>,
}

impl GeneName {
    pub fn new(value: String, ty: GeneNameType) -> Self {
        Self::new_with_evidence(value, ty, Vec::new())
    }

    pub fn new_with_evidence(value: String, ty: GeneNameType, evidence: Vec<usize>) -> Self {
        Self {
            value,
            ty,
            evidence
        }
    }
}

#[derive(Debug, Clone)]
pub enum GeneNameType {
    Primary,
    Synonym,
    OrderedLocus,
    Orf
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
/// Describes the source organism.
pub struct Organism {
    // fields
    pub names: Vec<OrganismName>,
    pub db_references: Vec<DbReference>,
    pub lineages: Vec<OrganismLineage>,
    // attributes
    pub evidences: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum OrganismName {
    Common(String),
    Full(String),
    Scientific(String),
    Synonym(String),
    Abbreviation(String),
}

#[derive(Debug, Default, Clone)]
pub struct OrganismLineage {
    pub taxons: Vec<String>,
}

// // ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes non-nuclear gene locations (organelles and plasmids).
pub struct GeneLocation {
    // name: Vec<Status>,
    pub ty: GeneLocationType,
    pub evidences: Vec<usize>,
    pub names: Vec<GeneLocationName>,
}

impl GeneLocation {
    pub fn new(ty: GeneLocationType) -> Self {
        Self {
            ty,
            evidences: Default::default(),
            names: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GeneLocationType {
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

#[derive(Debug, Clone)]
pub struct GeneLocationName {
    pub value: String,
    pub status: GeneLocationStatus
}

#[derive(Debug, Clone)]
pub enum GeneLocationStatus {
    Known,
    Unknown,
}

impl Default for GeneLocationStatus {
    fn default() -> Self {
        GeneLocationStatus::Known
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
/// Describes a citation and a summary of its content.
pub struct Reference {
    pub citation: Vec<Citation>,
    pub evidence: Vec<usize>,
    pub key: usize,
    pub scope: Vec<String>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone)]
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
    pub editors: Vec<CitationName>,
    /// Describes the authors of a citation.
    pub authors: Vec<CitationName>,
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

#[derive(Debug, Clone)]
pub enum CitationName {
    /// Describes the author of a citation when these are represented by a consortium.
    Consortium(String),
    /// Describes the authors of a citation when these are individuals.
    Person(String),
}

#[derive(Debug, Clone)]
/// Describes the source of the sequence according to the citation.
pub enum Source {
    Strain {
        value: String,
        evidences: Vec<usize>,
    },
    Plasmid {
        value: String,
        evidences: Vec<usize>,
    },
    Transposon {
        value: String,
        evidences: Vec<usize>,
    },
    Tissue {
        value: String,
        evidences: Vec<usize>,
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Describes different types of general annotations.
pub struct Comment {
    // fields
    pub molecule: Option<Molecule>,
    // location: Vec<Location>,
    pub text: Vec<String>,              // FIXME: type should be evidence text?
    pub ty: CommentType,
    // evidence: Vec<usize>,            // TODO: extract evidence attribute
}

impl Comment {
    pub fn new(ty: CommentType) -> Self {
        Self {
            ty,
            molecule: Default::default(),
            text: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommentType {
    Allergen,
    AlternativeProduct(AlternativeProduct),
    Biotechnology,
    BiophysicochemicalProperties(BiophysicochemicalProperties),
    CatalyticActivity(CatalyticActivity),
    Caution,
    Cofactor(Vec<Cofactor>),
    DevelopmentalStage,
    Disease(Option<Disease>),
    Domain,
    DisruptionPhenotype,
    ActivityRegulation,
    Function,
    Induction,
    Miscellaneous,
    Pathway,
    Pharmaceutical,
    Polymorphism,
    Ptm,
    RnaEditing(Vec<FeatureLocation>), // FIXME: possible dedicated type
    Similarity,
    SubcellularLocation(Vec<SubcellularLocation>),
    SequenceCaution(Conflict),
    Subunit,
    TissueSpecificity,
    ToxicDose,
    OnlineInformation(OnlineInformation),
    MassSpectrometry(MassSpectrometry),
    Interaction(Interaction),
}

#[derive(Debug, Default, Clone)]
pub struct MassSpectrometry {
    pub mass: Option<f64>,
    pub error: Option<String>,
    pub method: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Cofactor {
    pub name: String,
    pub db_reference: DbReference,
    pub evidences: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct OnlineInformation {
    pub name: Option<String>,
    pub links: Vec<Url>
}

#[derive(Debug, Clone)]
pub struct CatalyticActivity {
    pub reaction: Reaction,
    pub physiological_reactions: Vec<PhysiologicalReaction>
}

impl CatalyticActivity {
    pub fn new(reaction: Reaction) -> Self {
        Self {
            reaction,
            physiological_reactions: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reaction {
    pub text: String,
    pub db_references: Vec<DbReference>,
    pub evidences: Vec<usize>,
}

impl Reaction {
    pub fn new(text: String) -> Self {
        Self {
            text,
            db_references: Default::default(),
            evidences: Default::default()
        }
    }
}

#[derive(Debug, Clone)]
/// Describes a physiological reaction.
pub struct PhysiologicalReaction {
    pub db_reference: DbReference,
    pub evidences: Vec<usize>,
    pub direction: PhysiologicalReactionDirection,
}

#[derive(Debug, Clone)]
pub enum PhysiologicalReactionDirection {
    LeftToRight,
    RightToLeft
}

#[derive(Debug, Clone)]
pub struct Disease {
    pub id: String,
    pub name: String,
    pub description: String,
    pub acronym: String,
    pub db_reference: DbReference,
}

#[derive(Debug, Default, Clone)]
pub struct BiophysicochemicalProperties {
    pub absorption: Option<Absorption>,
    pub kinetics: Option<Kinetics>,
    pub ph_dependence: Option<String>, // TODO: EvidenceString
    pub redox_potential: Option<String>,  // TODO: EvidenceString
    pub temperature_dependence: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Absorption {
    pub max: Option<String>, // FIXME: evidence string
    pub min: Option<String>, // FIXME: evidence string
    pub text: Option<String>, // FIXME: evidence string
}

#[derive(Debug, Default, Clone)]
pub struct Kinetics {
    pub km: Vec<String>, // FIXME: evidence string
    pub vmax: Vec<String>, // FIXME: evidence string
    pub text: Option<String>  // FIXME: evidence string
}

#[derive(Debug, Default, Clone)]
/// Describes the subcellular location and optionally the topology and orientation of a molecule.
pub struct SubcellularLocation {
    pub locations: Vec<String>, // TODO: EvidenceString, minOccurs = "1"
    pub topologies: Vec<String>, // TODO: EvidenceString,
    pub orientations: Vec<String>, // TODO: EvidenceString,
}

#[derive(Debug, Default, Clone)]
pub struct AlternativeProduct {
    pub events: Vec<AlternativeProductEvent>,
    pub isoforms: Vec<Isoform>,
}

#[derive(Debug, Clone)]
pub enum AlternativeProductEvent {
    AlternativeSplicing,
    AlternativeInitiation,
    AlternativePromoter,
    RibosomalFrameshifting,
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

#[derive(Debug, Clone)]
pub enum IsoformSequenceType {
    NotDescribed,
    Described,
    Displayed,
    External
}

#[derive(Debug, Clone)]
pub struct Interaction {
    pub interactants: (Interactant, Interactant),
    pub organisms_differ: bool,
    pub experiments: usize,
}

#[derive(Debug, Clone)]
pub struct Interactant {
    pub interactant_id: String,
    pub id: Option<String>,
    pub label: Option<String>,
}

impl Interactant {
    pub fn new(interactant_id: String) -> Self {
        Self {
            interactant_id,
            id: Default::default(),
            label: Default::default(),
        }
    }
}

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
    pub resource: ConflictSequenceResource,
    pub version: Option<usize>,
}

impl ConflictSequence {
    pub fn new(id: String, resource: ConflictSequenceResource) -> Self {
        Self::with_version(id, resource, None)
    }

    pub fn with_version<N>(id: String, resource: ConflictSequenceResource, version: N) -> Self
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

#[derive(Debug, Clone)]
pub enum ConflictSequenceResource {
    Embl,
    EmblCds,
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct DbReference {
    pub molecule: Option<Molecule>,
    pub property: Vec<Property>,
    pub ty: String,
    pub id: String,
    pub evidences: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct Molecule {
    pub id: Option<String>,
    pub text: Option<String>,
}

impl Molecule {
    pub fn with_id<S>(id: S) -> Self
    where
        S: Into<Option<String>>
    {
        Self {
            id: id.into(),
            text: None,
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct Property {
    pub ty: String,
    pub value: String,
}

impl Property {
    pub fn new(ty: String, value: String) -> Self {
        Self { ty, value }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct Keyword {
    pub id: String,
    pub value: String,
    pub evidence: Vec<usize>,
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Feature {
    // fields
    pub original: Option<String>,
    pub variation: Vec<String>,
    pub location: FeatureLocation,

    // attributes
    pub ty: FeatureType,
    pub id: Option<String>,
    pub description: Option<String>,
    pub evidences: Vec<usize>,
    pub reference: Option<String>,
}

impl Feature {
    pub fn new(ty: FeatureType, location: FeatureLocation) -> Self {
        Self {
            original: Default::default(),
            variation: Default::default(),
            location,
            ty,
            id: Default::default(),
            description: Default::default(),
            evidences: Default::default(),
            reference: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FeatureType {
    ActiveSite,
    BindingSite,
    CalciumBindingRegion,
    Chain,
    CoiledCoilRegion,
    CompositionallyBiasedRegion,
    CrossLink,
    DisulfideBond,
    DnaBindingRegion,
    Domain,
    GlycosylationSite,
    Helix,
    InitiatorMethionine,
    LipidMoietyBindingRegion,
    MetalIonBindingSite,
    ModifiedResidue,
    MutagenesisSite,
    NonConsecutiveResidues,
    NonTerminalResidue,
    NucleotidePhosphateBindingRegion,
    Peptide,
    Propeptide,
    RegionOfInterest,
    Repeat,
    NonStandardAminoAcid,
    SequenceConflict,
    SequenceVariant,
    ShortSequenceMotif,
    SignalPeptide,
    Site,
    SpliceVariant,
    Strand,
    TopologicalDomain,
    TransitPeptide,
    TransmembraneRegion,
    Turn,
    UnsureResidue,
    ZincFingerRegion,
    IntramembraneRegion
}

#[derive(Debug, Clone)]
pub enum FeatureLocation {
    Range(FeaturePosition, FeaturePosition),
    Position(FeaturePosition)
}

#[derive(Debug, Clone)]
pub struct FeaturePosition {
    pub pos: Option<usize>,
    pub status: FeaturePositionStatus,
    pub evidence: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum FeaturePositionStatus {
    Certain,
    Uncertain,
    LessThan,
    GreaterThan,
    Unknown,
}

impl Default for FeaturePositionStatus {
    fn default() -> Self {
        FeaturePositionStatus::Certain
    }
}
