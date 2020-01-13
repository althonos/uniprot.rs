#[derive(Debug, Clone)]
pub struct Disease {
    pub id: String,
    pub name: String,
    pub description: String,
    pub acronym: String,
    pub db_reference: DbReference,
}
