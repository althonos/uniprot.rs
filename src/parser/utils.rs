use std::io::BufRead;
use std::str::FromStr;

use fnv::FnvHashMap;
use quick_xml::Reader;
use quick_xml::Error as XmlError;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesStart;

use super::Error;

// -----------------------------------------------------------------------

type HashMap<K, V> = fnv::FnvHashMap<K, V>;

// -----------------------------------------------------------------------

pub fn attributes_to_hashmap<'a>(event: &'a BytesStart<'a>) -> Result<HashMap<&'a [u8], Attribute<'a>>, Error> {
    event.attributes()
        .map(|r| r.map(|a| (a.key, a)).map_err(Error::from))
        .collect()
}

pub fn extract_attribute<'a>(event: &'a BytesStart<'a>, name: &str) -> Result<Option<Attribute<'a>>, Error> {
    event.attributes()
        .with_checks(false)
        .find(|r| r.is_err() || r.as_ref().ok().map_or(false, |a| a.key == name.as_bytes()))
        .transpose()
        .map_err(Error::from)
}

pub fn get_evidences<'a, B: BufRead>(reader: &mut Reader<B>, attr: &HashMap<&'a [u8], Attribute<'a>>) -> Result<Vec<usize>, Error> {
    attr.get(&b"evidence"[..])
        .map(|a| a.unescape_and_decode_value(reader))
        .transpose()?
        .map(|e| e.split(' ').map(usize::from_str).collect::<Result<Vec<_>, _>>().map_err(Error::from))
        .unwrap_or_else(|| Ok(Vec::new()))
}

/// Decode the attribute `name` from `event.attributes()`.
///
/// This functions uses an `unsafe` block to decode the attribute value
/// *only* when `FromStr::from_str` fails, given that all enum types of
/// this library only accept ASCII values.
pub fn decode_attribute<'a, B: BufRead, T: FromStr>(
    event: &'a BytesStart<'a>,
    reader: &mut Reader<B>,
    name: &'static str,
    element: &'static str,
) -> Result<T, Error> {
    unsafe {
        let a = extract_attribute(event, name)?
            .ok_or(Error::MissingAttribute(name, element))?;

        // perform decoding only on error, since valid enum variants
        // can only be obtained from valid UTF-8 anyway.
        let s = std::str::from_utf8_unchecked(&*a.value);
        T::from_str(s)
            .map_err(|_| match a.unescape_and_decode_value(reader) {
                Ok(s) => Error::invalid_value(name, element, s),
                Err(e) => Error::from(e),
            })
    }
}
