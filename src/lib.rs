//! [![Star me](https://img.shields.io/github/stars/althonos/uniprot.rs.svg?style=social&label=Star&maxAge=3600)](https://github.com/althonos/uniprot.rs/stargazers)
//!
//! *Rust data structures and parser for the [UniprotKB database(s)].*
//!
//! [UniprotKB database(s)]: https://www.uniprot.org/
//!
//! [![TravisCI](https://img.shields.io/travis/com/althonos/uniprot.rs/master.svg?maxAge=600&style=flat-square)](https://travis-ci.com/althonos/uniprot.rs/branches)
//! [![Codecov](https://img.shields.io/codecov/c/gh/althonos/uniprot.rs/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/althonos/uniprot.rs)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
//! [![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/uniprot.rs)
//! [![Crate](https://img.shields.io/crates/v/uniprot.svg?maxAge=600&style=flat-square)](https://crates.io/crates/uniprot)
//! [![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/uniprot)
//! [![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/uniprot.rs/blob/master/CHANGELOG.md)
//! [![GitHub issues](https://img.shields.io/github/issues/althonos/uniprot.rs.svg?style=flat-square&maxAge=600)](https://github.com/althonos/uniprot.rs/issues)
//!
//!
//! # 🔌 Usage
//!
//! The `uniprot::parse` function can be used to obtain an iterator over the entries
//! of a UniprotKB database in XML format (either SwissProt or TrEMBL). It
//! will use the [`uniprot::Parser`], which is either [`SequentialParser`]
//! or [`ThreadedParser`] depending on the compilation features.
//!
//! ```rust
//! extern crate uniprot;
//!
//! let f = std::fs::File::open("tests/uniprot.xml")
//!    .map(std::io::BufReader::new)
//!    .unwrap();
//!
//! for r in uniprot::parse(f) {
//!    let entry = r.unwrap();
//!    // ... process the Uniprot entry ...
//! }
//! ```
//!
//! `uniprot::parse` takes any [`BufRead`] implementor as an input. Additionaly,
//! if compiling with the [`threading`] feature, it will require the input to
//! be [`Send`] and `'static` as well.
//!
//! ## 📦 Decoding Gzip
//!
//! If parsing a Gzipped file, you can use [`flate2::read::GzDecoder`] or
//! [`libflate::gzip::Decoder`] to decode the input stream, and then simply
//! wrap it in a [`BufferedReader`]. Note that [`flate2`] has slightly better
//! performance, but binds to C,, while [`libflate`] is a pure Rust
//! implementation.
//!
//! ## 📧 Downloading from FTP
//!
//! Uniprot is available from the two following locations: [`ftp.ebi.ac.uk`]
//! and [`ftp.uniprot.org`], the former being located in Europe while the
//! latter is in the United States. The `ftp` crate can be used to open
//! a connection and parse the databases on-the-fly: see the
//! [`uniprot::parse`] example to see a code snippet.
//!
//! ## 📧 Downloading from HTTP
//!
//! If FTP is not available, note that the EBI FTP server can also be reached
//! using HTTP at [`http://ftp.ebi.ac.uk`]. This allows using HTTP libraries
//! instead of FTP ones to reach the release files.
//!
//!
//! # 📝 Features
//!
//! ## `threading`
//!
//! _**enabled** by default_.
//!
//! The `threading` feature compiles the parser module in multi-threaded mode.
//! This feature greatly improves parsing speed and efficiency, but removes
//! any guarantee about the order the entries are yielded in.
//!
//!
//! ## 📋 Changelog
//!
//! This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html)
//! and provides a [changelog](https://github.com/althonos/uniprot.rs/blob/master/CHANGELOG.md)
//! in the [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) format.
//!
//! ## 📜 License
//!
//! This library is provided under the open-source
//! [MIT license](https://choosealicense.com/licenses/mit/).
//!
//!
//! [`http://ftp.ebi.ac.uk`]: http://ftp.ebi.ac.uk
//! [`ftp.ebi.ac.uk`]: ftp://ftp.ebi.ac.uk
//! [`ftp.uniprot.org`]: ftp://ftp.uniprot.org
//! [`threading`]: #threading
//! [`flate2`]: https://docs.rs/flate2/
//! [`flate2::read::GzDecoder`]: https://docs.rs/flate2/latest/flate2/read/struct.GzDecoder.html
//! [`libflate`]: https://docs.rs/libflate/
//! [`libflate::gzip::Decoder`]: https://docs.rs/libflate/latest/libflate/gzip/struct.Decoder.html
//! [`BufferedReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
//! [`Entry`]: ./model/struct.Entry.html
//! [`uniprot::parse`]: ./fn.parse.html
//! [`uniprot::Parser`]: ./type.Parser.html
//! [`SequentialParser`]: ./parser/struct.SequentialParser.html
//! [`ThreadedParser`]: ./parser/struct.ThreadedParser.html

#![allow(unused_imports)]

extern crate bytes;
#[cfg(feature = "threading")]
extern crate crossbeam_channel;
extern crate fnv;
#[macro_use]
#[cfg(feature = "threading")]
extern crate lazy_static;
#[cfg(feature = "threading")]
extern crate num_cpus;
extern crate quick_xml;
extern crate thiserror;
extern crate url;

#[macro_use]
pub mod parser;
pub mod model;
pub mod error;

#[doc(inline)]
pub use self::parser::Parser;

use std::io::BufRead;

/// Parse a Uniprot database XML file.
///
/// # Example:
/// ```rust,no_run
/// let mut client = ftp::FtpStream::connect("ftp.uniprot.org:21").unwrap();
/// client.login("anonymous", "anonymous").unwrap();
///
/// let f = client.get("/pub/databases/uniprot/current_release/knowledgebase/complete/uniprot_sprot.xml.gz").unwrap();
/// let dec = libflate::gzip::Decoder::new(f).unwrap();
/// let mut parser = uniprot::parse(std::io::BufReader::new(dec));
///
/// println!("{:#?}", parser.next())
/// ```
pub fn parse<B: BufRead>(reader: B) -> Parser<B> {
    Parser::new(reader)
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

    mod sequential {
        use super::*;
        use crate::parser::SequentialParser;

        #[test]
        fn parse_single_entry() {
            let f = std::fs::File::open("tests/uniprot.xml").unwrap();
            SequentialParser::new(std::io::BufReader::new(f))
                .next()
                .expect("an entry should be parsed")
                .expect("the entry should be parsed successfully");
        }

        #[test]
        fn fail_unexpected_eof() {
            let txt = &b"<entry>"[..];
            let err = SequentialParser::new(std::io::Cursor::new(txt))
                .next()
                .expect("should raise an error")
                .unwrap_err();

            match err {
                Error::Xml(XmlError::UnexpectedEof(_)) => (),
                other => panic!("unexpected error: {:?}", other),
            }
        }

    }


    #[cfg(feature = "threading")]
    mod threaded {
        use super::*;
        use crate::parser::ThreadedParser;

        #[test]
        fn parse_single_entry() {
            let f = std::fs::File::open("tests/uniprot.xml").unwrap();
            ThreadedParser::new(std::io::BufReader::new(f))
                .next()
                .expect("an entry should be parsed")
                .expect("the entry should be parsed successfully");
        }

        #[test]
        fn fail_unexpected_eof() {
            let txt = &b"<entry>"[..];
            let err = ThreadedParser::new(std::io::Cursor::new(txt))
                .next()
                .expect("should raise an error")
                .unwrap_err();

            match err {
                Error::Xml(XmlError::UnexpectedEof(_)) => (),
                other => panic!("unexpected error: {:?}", other),
            }
        }
    }
}
