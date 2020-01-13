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
