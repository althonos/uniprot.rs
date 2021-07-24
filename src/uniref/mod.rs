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
/// # Example:
/// ```rust,no_run
/// let mut client = ftp::FtpStream::connect("ftp.uniprot.org:21").unwrap();
/// client.login("anonymous", "anonymous").unwrap();
///
/// let f = client.get("/pub/databases/uniprot/uniref/uniref50/uniref50.xml.gz").unwrap();
/// let dec = libflate::gzip::Decoder::new(f).unwrap();
/// let mut parser = uniprot::uniref::parse(std::io::BufReader::new(dec));
///
/// println!("{:#?}", parser.next())
/// ```
pub fn parse<B: BufRead>(reader: B) -> Parser<B> {
    Parser::new(reader)
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
