#[derive(Debug, Default, Clone)]
pub struct Cofactor {
    pub name: String,
    pub db_reference: DbReference,
    pub evidences: Vec<usize>,
}
