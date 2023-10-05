//! Common types for `uniprot` and `uniref`.
pub mod date;
pub mod property;
pub mod sequence;

/// The string type used throughout the library.
#[cfg(feature = "smartstring")]
pub type ShortString = smartstring::alias::String;

/// The string type used throughout the library.
#[cfg(not(feature = "smartstring"))]
pub type ShortString = std::string::String;
