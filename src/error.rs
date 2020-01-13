use quick_xml::Error as XmlError;

pub type Error = XmlError;

pub type Result<T> = std::result::Result<T, Error>;
