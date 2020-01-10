extern crate chrono;
extern crate quick_xml;

pub mod parser;
pub mod model;

#[cfg(test)]
#[test]
fn test_swissprot() {
    let f = std::fs::File::open("uniprot.xml").unwrap();
    let r = std::io::BufReader::new(f);

    for x in parser::parse(r) {
        println!("{:?}", x.unwrap().accessions);
    }
}
