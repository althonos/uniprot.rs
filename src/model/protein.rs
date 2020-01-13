#[derive(Debug, Clone, Default)]
/// Describes the names for the protein and parts thereof.
pub struct Protein {
    pub name: Nomenclature,
    pub domains: Vec<Nomenclature>,
    pub components: Vec<Nomenclature>,
}

#[derive(Debug, Clone, Default)]
pub struct Nomenclature {
    pub recommended: Option<Name>,
    pub alternative: Vec<Name>,
    pub submitted: Vec<Name>,
    pub allergen: Option<String>,     // FIXME: type should be EvidenceString?
    pub biotech: Option<String>,
    pub cd_antigen: Vec<String>,
    pub inn: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Name {
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
