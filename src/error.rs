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
    ParseInt(#[from] ParseIntError),
    #[error("parser error: {0}")]
    ParseBool(#[from] ParseBoolError),
    #[error("parser error: {0}")]
    ParseUrl(#[from] ParseUrlError),
    #[error("missing element `{0}` in `{1}`")]
    MissingElement(&'static str, &'static str),
    #[error("missing attribute `{0}` in `{1}`")]
    MissingAttribute(&'static str, &'static str),
    #[error("duplicate element `{0}` in `{1}`")]
    DuplicateElement(&'static str, &'static str),
    #[error("invalid value for attribute `{0}` in `{1}`")]
    InvalidValue(&'static str, &'static str, #[source] InvalidValue),
    #[cfg(feature = "threading")]
    #[error("unexpected threading channel disconnection")]
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
