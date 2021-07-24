//! Ubiquitous types for error management.

use std::error::Error as StdError;
use std::io::Error as IoError;
use std::num::ParseIntError;
use std::str::ParseBoolError;

use quick_xml::Error as XmlError;
use thiserror::Error;
use url::ParseError as ParseUrlError;

#[derive(Debug, Error)]
/// The main error type for the [`uniprot`] crate.
///
/// [`uniprot`]: ../index.html
pub enum Error {
    #[error(transparent)]
    /// The underlying XML parser encountered an error.
    ///
    /// *Any error from the underlying reader will be wrapped in the
    /// [`XmlError::Io`] variant.*
    ///
    /// [`XmlError::Io`]: https://docs.rs/quick-xml/latest/quick_xml/enum.Error.html#variant.Io
    Xml(#[from] XmlError),
    #[error("parser error: {0}")]
    /// An integer value could not be parsed successfully.
    ParseInt(#[from] ParseIntError),
    #[error("parser error: {0}")]
    /// A boolean value could not be parsed successfully.
    ParseBool(#[from] ParseBoolError),
    #[error("parser error: {0}")]
    /// A `Url` value could not be parsed successfully.
    ParseUrl(#[from] ParseUrlError),
    #[error("missing element `{0}` in `{1}`")]
    /// A required element is missing.
    MissingElement(&'static str, &'static str),
    #[error("missing attribute `{0}` in `{1}`")]
    /// A required attribute is missing.
    MissingAttribute(&'static str, &'static str),
    #[error("duplicate element `{0}` in `{1}`")]
    /// An element which should be unique was found more than once.
    DuplicateElement(&'static str, &'static str),
    #[error("invalid value for attribute `{0}` in `{1}`")]
    /// A value could not be parsed successfully.
    InvalidValue(&'static str, &'static str, #[source] InvalidValue),
    /// Unexpected root element.
    #[error("unexpected root element `{0}`")]
    UnexpectedRoot(String),
    #[cfg(feature = "threading")]
    #[error("unexpected threading channel disconnection")]
    /// A communication channel between threads was disconnected early.
    DisconnectedChannel,
}

impl Error {
    pub fn invalid_value<S: Into<String>>(
        name: &'static str,
        elem: &'static str,
        value: S,
    ) -> Self {
        Error::InvalidValue(name, elem, InvalidValue(value.into()))
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::from(XmlError::Io(e))
    }
}

/// The main result type for the [`uniprot`] crate.
///
/// [`uniprot`]: ../index.html
pub type Result<T> = std::result::Result<T, Error>;

// ---------------------------------------------------------------------------

#[derive(Default, Debug, Clone, Error, PartialEq, Eq)]
#[error("invalid value: {0}")]
/// The error type for types with constrained values.
pub struct InvalidValue(pub String);

impl<S: Into<String>> From<S> for InvalidValue {
    fn from(s: S) -> Self {
        InvalidValue(s.into())
    }
}
