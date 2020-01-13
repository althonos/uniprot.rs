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
