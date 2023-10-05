use crate::common::ShortString;

#[cfg(feature = "url-links")]
use url::Url;

#[derive(Debug, Default, Clone)]
pub struct OnlineInformation {
    pub name: Option<ShortString>,
    #[cfg(feature = "url-links")]
    pub links: Vec<Url>,
    #[cfg(not(feature = "url-links"))]
    pub links: Vec<ShortString>,
}
