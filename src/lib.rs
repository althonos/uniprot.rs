//! [![Star me](https://img.shields.io/github/stars/althonos/uniprot.rs.svg?style=social&label=Star&maxAge=3600)](https://github.com/althonos/uniprot.rs/stargazers)
//!
//! *Rust data structures and parser for the [UniprotKB database(s)].*
//!
//! [UniprotKB database(s)]: https://www.uniprot.org/
//!
//! [![TravisCI](https://img.shields.io/travis/althonos/uniprot.rs/master.svg?maxAge=600&style=flat-square)](https://travis-ci.com/althonos/uniprot.rs/branches)
//! [![Codecov](https://img.shields.io/codecov/c/gh/althonos/uniprot.rs/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/althonos/uniprot.rs)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
//! [![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/uniprot.rs)
//! [![Crate](https://img.shields.io/crates/v/uniprot.svg?maxAge=600&style=flat-square)](https://crates.io/crates/uniprot)
//! [![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/uniprot)
//! [![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/uniprot.rs/blob/master/CHANGELOG.md)
//! [![GitHub issues](https://img.shields.io/github/issues/althonos/uniprot.rs.svg?style=flat-square&maxAge=600)](https://github.com/althonos/uniprot.rs/issues)

#![allow(unused_imports)]

extern crate bytes;
extern crate quick_xml;
extern crate url;
extern crate fnv;
#[cfg(feature = "threading")]
extern crate crossbeam_channel;

#[macro_use]
pub mod parser;
pub mod model;
pub mod error;

#[doc(inline)]
pub use self::parser::UniprotParser;

use std::io::BufRead;

/// Parse a Uniprot database XML file.
///
/// # Example:
/// ```rust
/// let mut client = ftp::FtpStream::connect("ftp.ebi.ac.uk:21").unwrap();
/// client.login("anonymous", "anonymous").unwrap();
///
/// let f = client.get("/pub/databases/uniprot/current_release/knowledgebase/complete/uniprot_sprot.xml.gz").unwrap();
/// let dec = libflate::gzip::Decoder::new(f).unwrap();
/// let mut parser = uniprot::parse(std::io::BufReader::new(dec));
///
/// println!("{:#?}", parser.next())
/// ```
pub fn parse<B: BufRead + Send + 'static>(reader: B) -> UniprotParser<B> {
    UniprotParser::new(reader)
}

#[cfg(test)]
mod tests {

    use quick_xml::Error as XmlError;
    use crate::error::Error;
    use super::*;

    #[test]
    fn parse_swissprot_200() {
        let f = std::fs::File::open("tests/uniprot.xml").unwrap();
        let entries = crate::parse(std::io::BufReader::new(f))
            .collect::<Result<Vec<_>, _>>()
            .expect("entries should parse successfully");
        assert_eq!(entries.len(), 200);
    }

    #[test]
    fn parse_single_entry() {
        let f = std::fs::File::open("tests/uniprot.xml").unwrap();
        crate::parse(std::io::BufReader::new(f))
            .next()
            .expect("an entry should be parsed")
            .expect("the entry should be parsed successfully");
    }

    #[test]
    fn fail_unexpected_eof() {
        let txt = &b"<entry>"[..];
        let err = crate::parse(std::io::Cursor::new(txt))
            .next()
            .expect("should raise an error")
            .unwrap_err();

        match err {
            Error::Xml(XmlError::UnexpectedEof(_)) => (),
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
