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
pub fn parse<B: BufRead>(reader: B) -> UniprotParser<B> {
    UniprotParser::new(reader)
}

#[cfg(test)]
mod tests {

    use super::*;

    const SPROT: &str = "http://ftp.ebi.ac.uk/pub/databases/uniprot/current_release/knowledgebase/complete/uniprot_sprot.xml.gz";

    #[test]
    fn parse_swissprot() {
        // connect to the EBI FTP server via HTTP
        let gzipped = reqwest::blocking::get(SPROT)
            .expect("could not connect to EBI FTP server");

        // decode gzip stream
        let dec = libflate::gzip::Decoder::new(gzipped).unwrap();
        for x in crate::parse(std::io::BufReader::new(dec)).take(1000) {
            let entry = x.expect("parsing of entry failed");
            assert!(!entry.accessions.is_empty());
            assert!(!entry.names.is_empty());
        }
    }

    #[test]
    fn parse_with_ignore() {
        //
        let txt = std::fs::read_to_string("tests/uniprot.xml")
            .unwrap();

        // check parsing normally will get some hosts
        let entry = crate::parse(std::io::Cursor::new(&txt))
            .next()
            .expect("an entry should be parsed")
            .expect("the entry should be parsed successfully");
        assert!(!entry.organism_hosts.is_empty());

        // check parsing with `organismHost` in ignore skips hosts
        let entry = crate::parse(std::io::Cursor::new(&txt))
            .ignore("organismHost")
            .next()
            .expect("an entry should be parsed")
            .expect("the entry should be parsed successfully");
        assert!(entry.organism_hosts.is_empty());
    }
}
