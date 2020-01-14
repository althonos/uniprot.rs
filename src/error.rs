use std::error::Error as StdError;
use std::num::ParseIntError;
use std::str::ParseBoolError;

use err_derive::Error;
use url::ParseError as ParseUrlError;
use quick_xml::Error as XmlError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "xml error: {}", 0)]
    XmlError(#[error(source)] XmlError),
    #[error(display = "parser error: {}", 0)]
    ParseIntError(#[error(source)] ParseIntError),
    #[error(display = "parser error: {}", 0)]
    ParseBoolError(#[error(source)] ParseBoolError),
    #[error(display = "parser error: {}", 0)]
    ParseUrlError(#[error(source)] ParseUrlError),
    #[error(display = "missing element `{}` in `{}`", 0, 1)]
    MissingElement(&'static str, &'static str),
    #[error(display = "missing attribute `{}` in `{}`", 0, 1)]
    MissingAttribute(&'static str, &'static str),
    #[error(display = "duplicate element `{}` in `{}`", 0, 1)]
    DuplicateElement(&'static str, &'static str),
}

// pub type Error = XmlError;

pub type Result<T> = std::result::Result<T, Error>;
