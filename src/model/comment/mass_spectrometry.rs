#[derive(Debug, Default, Clone)]
pub struct MassSpectrometry {
    pub mass: Option<f64>,
    pub error: Option<String>,
    pub method: Option<String>,
}
