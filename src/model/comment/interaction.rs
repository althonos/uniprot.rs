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
