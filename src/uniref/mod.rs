//! Data types for the UniRef databases.

use std::io::BufRead;

mod model;

#[doc(inline)]
pub use self::model::*;

/// The sequential parser type for UniRef entries.
pub type SequentialParser<B> = super::parser::SequentialParser<B, UniRef>;

#[cfg(feature = "threading")]
/// The threaded parser type for UniRef entries.
pub type ThreadedParser<B> = super::parser::ThreadedParser<B, UniRef>;

/// The parser type for UniRef entries.
pub type Parser<B> = super::parser::Parser<B, UniRef>;

/// Parse a UniRef database XML file.
///
/// # Examples:
///
/// Connect to the EBI FTP server and parse the complete UniRef50 data dump
/// using the [`ftp`](https://crates.io/crates/ftp) crate.
///
/// ```rust,no_run
/// let mut client = ftp::FtpStream::connect("ftp.uniprot.org:21").unwrap();
/// client.login("anonymous", "anonymous").unwrap();
///
/// let f = client.get("/pub/databases/uniprot/uniref/uniref50/uniref50.xml.gz").unwrap();
/// let dec = libflate::gzip::Decoder::new(f).unwrap();
/// let mut parser = uniprot::uniref::parse(std::io::BufReader::new(dec));
///
/// println!("{:#?}", parser.next());
/// ```
///
/// Retrieve a single protein entry directly from the UniProt website using
/// [`ureq`](https://crates.io/crates/ureq) to perform the HTTP request.
///
/// ```rust
/// let api_url = "https://www.uniprot.org/uniref/UniRef90_P99999.xml";
///
/// let req = ureq::get(&api_url).set("Accept", "application/xml");
/// let dec = libflate::gzip::Decoder::new(req.call().unwrap().into_reader()).unwrap();
/// let reader = std::io::BufReader::new(dec);
/// let entry = uniprot::uniref::parse(reader).next().unwrap().unwrap();
///
/// println!("{:?}", entry);
/// ```
pub fn parse<B: BufRead+Send+'static>(reader: B) -> Parser<B> {
    Parser::new(reader)
}

/// Parse a single UniRef entry.
pub fn parse_entry<B: BufRead+Send+'static>(reader: B) -> <Parser<B> as Iterator>::Item {
    SequentialParser::parse_entry(reader)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::error::Error;
    use quick_xml::Error as XmlError;

    #[test]
    fn parse_uniref50() {
        let f = std::fs::File::open("tests/uniref50.xml").unwrap();
        let entries = super::parse(std::io::BufReader::new(f))
            .collect::<Result<Vec<_>, _>>()
            .expect("entries should parse successfully");
        assert_eq!(entries.len(), 59);
    }

    mod sequential {
        use super::*;

        #[test]
        fn parse_single_entry() {
            let f = std::fs::File::open("tests/uniref50.xml").unwrap();
            SequentialParser::new(std::io::BufReader::new(f))
                .next()
                .expect("an entry should be parsed")
                .expect("the entry should be parsed successfully");
        }

        #[test]
        fn fail_unexpected_eof() {
            let txt = &b"<UniRef><entry id=\"UniRef50_A0A5A9P0L4\" updated=\"2019-12-18\">"[..];
            let err = SequentialParser::new(std::io::Cursor::new(txt))
                .next()
                .expect("should raise an error")
                .unwrap_err();

            match err {
                Error::Xml(XmlError::UnexpectedEof(_)) => (),
                other => panic!("unexpected error: {:?}", other),
            }
        }

        #[test]
        fn fail_unexpected_root() {
            let txt = &b"<something><entry>"[..];
            let err = SequentialParser::new(std::io::Cursor::new(txt))
                .next()
                .expect("should raise an error")
                .unwrap_err();
            match err {
                Error::UnexpectedRoot(r) => assert_eq!(r, "something"),
                other => panic!("unexpected error: {:?}", other),
            }
        }
    }

    #[cfg(feature = "threading")]
    mod threaded {
        use super::*;

        #[test]
        fn parse_single_entry() {
            let f = std::fs::File::open("tests/uniref50.xml").unwrap();
            ThreadedParser::new(std::io::BufReader::new(f))
                .next()
                .expect("an entry should be parsed")
                .expect("the entry should be parsed successfully");
        }

        #[test]
        fn fail_unexpected_eof() {
            let txt = &b"<UniRef><entry id=\"UniRef50_A0A5A9P0L4\" updated=\"2019-12-18\">"[..];
            let err = ThreadedParser::new(std::io::Cursor::new(txt))
                .next()
                .expect("should not yield `None`")
                .unwrap_err();

            match err {
                Error::Xml(XmlError::UnexpectedEof(_)) => (),
                other => panic!("unexpected error: {:?}", other),
            }
        }

        #[test]
        fn fail_unexpected_root() {
            let txt = &b"<something><entry>"[..];
            let err = SequentialParser::new(std::io::Cursor::new(txt))
                .next()
                .expect("should raise an error")
                .unwrap_err();
            match err {
                Error::UnexpectedRoot(r) => assert_eq!(r, "something"),
                other => panic!("unexpected error: {:?}", other),
            }
        }
    }
}
