#[derive(Debug, Default, Clone)]
/// Describes the subcellular location and optionally the topology and orientation of a molecule.
pub struct SubcellularLocation {
    pub locations: Vec<String>, // TODO: EvidenceString, minOccurs = "1"
    pub topologies: Vec<String>, // TODO: EvidenceString,
    pub orientations: Vec<String>, // TODO: EvidenceString,
}
