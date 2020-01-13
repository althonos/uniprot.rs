#![allow(unused_imports)]

extern crate chrono;
extern crate quick_xml;
extern crate url;

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
/// println!("{:?}", parser.next())
///
/// ```
pub fn parse<B: BufRead>(reader: B) -> UniprotParser<B> {
    UniprotParser::new(reader)
}


#[cfg(test)]
#[test]
fn test_swissprot() {

    const SPROT: &str = "/pub/databases/uniprot/current_release/knowledgebase/complete/uniprot_sprot.xml.gz";

    // open FTP and connect anonymously
    let mut client = ftp::FtpStream::connect("ftp.ebi.ac.uk:21").unwrap();
    client.login("anonymous", "anonymous").unwrap();

    // decode gzip stream
    let gzipped = client.get(SPROT).unwrap();
    let dec = libflate::gzip::Decoder::new(gzipped).unwrap();
    for x in crate::parse(std::io::BufReader::new(dec)) {
        println!("{:?}", x.unwrap().accessions);
    }
}
