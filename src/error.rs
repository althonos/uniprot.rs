//! Ubiquitous types for error management.

use std::error::Error as StdError;
use std::num::ParseIntError;
use std::str::ParseBoolError;

use err_derive::Error;
use url::ParseError as ParseUrlError;
use quick_xml::Error as XmlError;

#[derive(Debug, Error)]
/// The main error type for the `uniprot` crate.
pub enum Error {
    #[error(display = "xml error: {}", 0)]
    Xml(#[error(source)] XmlError),
    #[error(display = "parser error: {}", 0)]
    ParseInt(#[error(source)] ParseIntError),
    #[error(display = "parser error: {}", 0)]
    ParseBool(#[error(source)] ParseBoolError),
    #[error(display = "parser error: {}", 0)]
    ParseUrl(#[error(source)] ParseUrlError),
    #[error(display = "missing element `{}` in `{}`", 0, 1)]
    MissingElement(&'static str, &'static str),
    #[error(display = "missing attribute `{}` in `{}`", 0, 1)]
    MissingAttribute(&'static str, &'static str),
    #[error(display = "duplicate element `{}` in `{}`", 0, 1)]
    DuplicateElement(&'static str, &'static str),
    #[error(display = "invalid value for attribute `{}` in `{}`", 0, 1)]
    InvalidValue(&'static str, &'static str, #[error(source)] InvalidValue),

    #[cfg(feature = "threading")]
    #[error(display = "unexpected threading channel disconnection")]
    DisconnectedChannel,
}

impl Error {
    pub fn invalid_value<S: Into<String>>(
        name: &'static str,
        elem: &'static str,
        value: S
    ) -> Self {
        Error::InvalidValue(name, elem, InvalidValue(value.into()))
    }
}

/// The main result type for the `uniprot crate`
pub type Result<T> = std::result::Result<T, Error>;

// ---------------------------------------------------------------------------

#[derive(Default, Debug, Clone, Error, PartialEq, Eq)]
#[error(display = "invalid value: {}", 0)]
/// The error type for types with constrained values.
pub struct InvalidValue(pub String);

impl<S: Into<String>> From<S> for InvalidValue {
    fn from(s: S) -> Self {
        InvalidValue(s.into())
    }
}
