#![feature(test)]

extern crate ftp;
extern crate test;
extern crate uniprot;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Cursor;

use test::Bencher;
use quick_xml::events::Event;

#[bench]
fn bench_read(b: &mut Bencher) {
    let r = File::open("uniprot.xml").map(BufReader::new).unwrap();
    let mut txt = r.lines()
        .take(100442)
        .collect::<Result<String, _>>()
        .unwrap();
    txt.push_str("</uniprot>");

    b.iter(|| {
        Cursor::new(&txt)
            .lines()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();
    });
    b.bytes = txt.as_bytes().len() as u64;
}

#[bench]
fn bench_quickxml(b: &mut Bencher) {
    let r = File::open("uniprot.xml").map(BufReader::new).unwrap();
    let mut txt = r.lines()
        .take(100442)
        .collect::<Result<String, _>>()
        .unwrap();
    txt.push_str("</uniprot>");

    b.iter(|| {
        let mut r = quick_xml::Reader::from_reader(Cursor::new(&txt));
        let mut events = Vec::new();
        let mut buffer = Vec::new();

        loop {
            match r.read_event(&mut buffer) {
                Err(err) => panic!("{}", err),
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(event) => match event {
                    Event::Start(_) | Event::End(_) | Event::Text(_) => {
                        events.push(event.into_owned());
                    }
                    _ => (),
                }
            }
        }
     });
    b.bytes = txt.as_bytes().len() as u64;
}

#[bench]
fn bench_sequential_parser(b: &mut Bencher) {

    let r = File::open("uniprot.xml").map(BufReader::new).unwrap();
    let mut txt = r.lines()
        .take(100442)
        .collect::<Result<String, _>>()
        .unwrap();
    txt.push_str("</uniprot>");

    b.iter(|| {
        for entry in uniprot::parser::SequentialParser::new(Cursor::new(&txt)) {
            entry.unwrap();
        }
    });

    b.bytes = txt.as_bytes().len() as u64;
}

#[bench]
fn bench_threaded_parser(b: &mut Bencher) {

    let r = File::open("uniprot.xml").map(BufReader::new).unwrap();
    let mut txt = r.lines()
        .take(100442)
        .collect::<Result<String, _>>()
        .unwrap();
    txt.push_str("</uniprot>");

    b.iter(|| {
        for entry in uniprot::parser::ThreadedParser::new(Cursor::new(&txt)) {
            entry.unwrap();
        }
    });

    b.bytes = txt.as_bytes().len() as u64;
}
