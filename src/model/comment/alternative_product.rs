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
