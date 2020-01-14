#![allow(unused_imports)]

extern crate bytes;
extern crate quick_xml;
extern crate url;
extern crate fnv;

#[macro_use]
pub mod parser;
pub mod model;
pub mod error;

use std::io::BufRead;
use self::parser::UniprotParser;

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
        // connect to the EBI FTP server via HTTP
        let gzipped = reqwest::blocking::get(SPROT)
            .expect("could not connect to EBI FTP server");

        // decode gzip stream and collect only first entry
        let dec = libflate::gzip::Decoder::new(gzipped).unwrap();
        let mut txt = std::io::BufReader::new(dec).lines()
            .take(110)
            .collect::<Result<String, _>>()
            .unwrap();
        txt.push_str("</uniprot>");

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
