
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
