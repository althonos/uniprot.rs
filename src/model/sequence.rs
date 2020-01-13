use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;

#[derive(Debug, Default, Clone)]
pub struct Sequence {
    pub value: String,
    pub length: usize,
    pub mass: usize,
    pub checksum: u64,
    // modified: NaiveDate,
    pub version: usize,
    pub precursor: Option<bool>,
    pub fragment: Option<FragmentType>
}

impl FromXml for Sequence {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"sequence");

        let attr = attributes_to_hashmap(event)?;
        let length = attr.get(&b"length"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .expect("ERR: could not find required `length` in `sequence`")
            .expect("ERR: could not parse `length` as usize");
        let mass = attr.get(&b"mass"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .expect("ERR: could not find required `mass` in `sequence`")
            .expect("ERR: could not parse `mass` as usize");
        let checksum = attr.get(&b"checksum"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| u64::from_str_radix(&x, 16))
            .expect("ERR: could not find required `checksum` in `sequence`")
            .expect("ERR: could not parse `checksum` as hex u64");
        // let modified = TODO
        let version = attr.get(&b"version"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .expect("ERR: could not find required `version` in `sequence`")
            .expect("ERR: could not parse `version` as usize");
        let precursor = attr.get(&b"precursor"[..])
            .map(|x| x.unescape_and_decode_value(reader))
            .transpose()?
            .map(|x| bool::from_str(&x))
            .transpose()
            .expect("ERR: could not parse `precursor` as bool");
        let fragment = match attr.get(&b"fragment"[..]).map(|x| &*x.value) {
            Some(b"single") => Some(FragmentType::Single),
            Some(b"multiple") => Some(FragmentType::Multiple),
            Some(other) => panic!("ERR: invalid `fragment` in `sequence`: {:?}", other),
            None => None,
        };

        let value = reader.read_text(b"sequence", buffer)?;
        Ok(Sequence {
            value,
            length,
            mass,
            checksum,
            version,
            precursor,
            fragment,
        })
    }
}

#[derive(Debug, Clone)]
pub enum FragmentType {
    Single,
    Multiple,
}
