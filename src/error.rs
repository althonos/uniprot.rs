//! Ubiquitous types for error management.

use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::num::ParseIntError;
use std::str::ParseBoolError;

use quick_xml::Error as XmlError;
#[cfg(feature = "url-links")]
use url::ParseError as ParseUrlError;

#[derive(Debug)]
/// The main error type for the [`uniprot`] crate.
///
/// [`uniprot`]: ../index.html
pub enum Error {
    /// The underlying XML parser encountered an error.
    ///
    /// *Any error from the underlying reader will be wrapped in the
    /// [`XmlError::Io`] variant.*
    ///
    /// [`XmlError::Io`]: https://docs.rs/quick-xml/latest/quick_xml/enum.Error.html#variant.Io
    Xml(XmlError),

    /// An integer value could not be parsed successfully.
    ParseInt(ParseIntError),

    /// A boolean value could not be parsed successfully.
    ParseBool(ParseBoolError),

    /// A required element is missing.
    MissingElement(&'static str, &'static str),

    /// A required attribute is missing.
    MissingAttribute(&'static str, &'static str),

    /// An element which should be unique was found more than once.
    DuplicateElement(&'static str, &'static str),

    /// A value could not be parsed successfully.
    InvalidValue(&'static str, &'static str, InvalidValue),

    /// Unexpected root element.
    UnexpectedRoot(String),

    #[cfg(feature = "url-links")]
    /// A `Url` value could not be parsed successfully.
    ParseUrl(ParseUrlError),

    #[cfg(feature = "threading")]
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use self::Error::*;
        match self {
            Xml(e) => e.fmt(f),
            ParseInt(e) => write!(f, "parser error: {}", e),
            ParseBool(e) => write!(f, "parser error: {}", e),
            #[cfg(feature = "url-links")]
            ParseUrl(e) => write!(f, "parser error: {}", e),
            MissingElement(x, y) => write!(f, "missing element `{}` in `{}`", x, y),
            MissingAttribute(x, y) => write!(f, "missing attribute `{}` in `{}`", x, y),
            DuplicateElement(x, y) => write!(f, "duplicate element `{}` in `{}`", x, y),
            InvalidValue(x, y, _) => write!(f, "invalid value for attribute `{}` in `{}`", x, y),
            UnexpectedRoot(root) => write!(f, "unexpected root element `{}`", root),
            #[cfg(feature = "threading")]
            DisconnectedChannel => write!(f, "unexpected threading channel disconnection"),
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::from(XmlError::Io(e))
    }
}

impl From<ParseBoolError> for Error {
    fn from(e: ParseBoolError) -> Self {
        Error::ParseBool(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

#[cfg(feature = "url-links")]
impl From<ParseUrlError> for Error {
    fn from(e: ParseUrlError) -> Self {
        Error::ParseUrl(e)
    }
}

impl From<XmlError> for Error {
    fn from(e: XmlError) -> Self {
        Error::Xml(e)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::Error::*;
        match self {
            Xml(e) => Some(e),
            ParseInt(e) => Some(e),
            ParseBool(e) => Some(e),
            InvalidValue(_, _, e) => Some(e),
            #[cfg(feature = "url-links")]
            ParseUrl(e) => Some(e),
            _ => None,
        }
    }
}

/// The main result type for the [`uniprot`] crate.
///
/// [`uniprot`]: ../index.html
pub type Result<T> = std::result::Result<T, Error>;

// ---------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq, Eq)]
/// The error type for types with constrained values.
pub struct InvalidValue(pub String);

impl Display for InvalidValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "invalid value: {}", &self.0)
    }
}

impl StdError for InvalidValue {}

impl<S: Into<String>> From<S> for InvalidValue {
    fn from(s: S) -> Self {
        InvalidValue(s.into())
    }
}
