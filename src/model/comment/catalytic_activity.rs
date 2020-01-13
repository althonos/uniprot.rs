use super::super::db_reference::DbReference;

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
