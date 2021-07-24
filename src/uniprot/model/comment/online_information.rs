use url::Url;

#[derive(Debug, Default, Clone)]
pub struct OnlineInformation {
    pub name: Option<String>,
    pub links: Vec<Url>,
}
