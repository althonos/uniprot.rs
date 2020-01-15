//!

use std::collections::HashSet;
use std::io::BufRead;
use std::str::FromStr;

use bytes::Bytes;
use quick_xml::Reader;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;

use super::model::*;
use super::error::Error;

macro_rules! parse_inner {
    ($event:expr, $reader:expr, $buffer:expr, $($rest:tt)*) => ({
        loop {
            use $crate::quick_xml::events::BytesEnd;
            use $crate::quick_xml::events::BytesStart;
            use $crate::quick_xml::events::Event;
            use $crate::quick_xml::Error as XmlError;

            $buffer.clear();
            match $reader.read_event($buffer) {
                Ok(Event::Start(ref x)) => {
                    parse_inner_impl!(x, x.local_name(), $($rest)*);
                    $reader.read_to_end(x.local_name(), &mut Vec::new())?;
                    unimplemented!(
                        "`{}` in `{}`",
                        String::from_utf8_lossy(x.local_name()),
                        String::from_utf8_lossy($event.local_name())
                    );
                }
                Err(e) => {
                    return Err(Error::from(e));
                }
                Ok(Event::Eof) => {
                    let e = String::from_utf8_lossy($event.local_name()).to_string();
                    return Err(Error::from(XmlError::UnexpectedEof(e)));
                }
                Ok(Event::End(ref e)) if e.local_name() == $event.local_name() => {
                    break;
                }
                Ok(Event::End(ref e)) => {
                    let expected = $event.unescaped()
                        .map(|s| String::from_utf8_lossy(s.as_ref()).to_string())?;
                    let found = String::from_utf8_lossy(e.name()).to_string();
                    let e = XmlError::EndEventMismatch { expected, found };
                    return Err(Error::from(e));
                }
                _ => continue,
            }
        }
    })
}

macro_rules! parse_inner_ignoring {
    ($event:expr, $reader:expr, $buffer:expr, $ignores:expr, $($rest:tt)*) => ({
        loop {
            use $crate::quick_xml::events::BytesEnd;
            use $crate::quick_xml::events::BytesStart;
            use $crate::quick_xml::events::Event;
            use $crate::quick_xml::Error as XmlError;

            $buffer.clear();
            match $reader.read_event($buffer) {
                Ok(Event::Start(ref x)) => {
                    if $ignores.contains(x.local_name()) {
                        $reader.read_to_end(x.local_name(), &mut Vec::new())?;
                        continue
                    } else {
                        parse_inner_impl!(x, x.local_name(), $($rest)*);
                    }

                    unimplemented!(
                        "`{}` in `{}`",
                        String::from_utf8_lossy(x.local_name()),
                        String::from_utf8_lossy($event.local_name())
                    );
                }
                Err(e) => {
                    return Err(Error::from(e));
                }
                Ok(Event::Eof) => {
                    let e = String::from_utf8_lossy($event.local_name()).to_string();
                    return Err(Error::from(XmlError::UnexpectedEof(e)));
                }
                Ok(Event::End(_)) => break,
                _ => continue,
            }
        }
    })
}

macro_rules! parse_inner_impl {
    ( $x:ident, $name:expr ) => ();
    ( $x:ident, $name:expr, ) => ();
    ( $x:ident, $name:expr, $e:ident @ $l:expr => $r:expr ) => (
        if $name == $l {
            let $e = $x.clone().into_owned();
            $r;
            continue;
        }
    );
    ( $x:ident, $name:expr, $l:expr => $r:expr ) => (
        if $name == $l {
            $r;
            continue;
        }
    );
    ( $x:ident, $name:expr, $e:ident @ $l:expr => $r:expr, $($rest:tt)*) => (
        parse_inner_impl!( $x, $name, $e @ $l => $r );
        parse_inner_impl!( $x, $name, $($rest)* );
    );
    ( $x:ident, $name:expr, $l:expr => $r:expr, $($rest:tt)*) => (
        parse_inner_impl!( $x, $name, $l => $r );
        parse_inner_impl!( $x, $name, $($rest)* );
    )
}

macro_rules! parse_comment {
    ( $event:ident, $reader:ident, $buffer:ident, $comment:ident ) => {
        parse_comment!{$event, $reader, $buffer, $comment, }
    };
    ( $event:ident, $reader:ident, $buffer:ident, $comment:ident, $($rest:tt)* ) => {
        parse_inner!{$event, $reader, $buffer,
            b"text" => {
                $comment.text.push($reader.read_text(b"text", $buffer)?);
            },
            m @ b"molecule" => {
                $comment.molecule = Molecule::from_xml(&m, $reader, $buffer)
                    .map(Some)?;
            },
            $($rest)*
        }
    }
}


// ---------------------------------------------------------------------------

pub(crate) mod utils {
    use std::io::BufRead;
    use std::str::FromStr;

    use fnv::FnvHashMap;
    use quick_xml::Reader;
    use quick_xml::Error as XmlError;
    use quick_xml::events::attributes::Attribute;
    use quick_xml::events::BytesStart;

    use super::Error;

    type HashMap<K, V> = fnv::FnvHashMap<K, V>;

    pub(crate) fn attributes_to_hashmap<'a>(event: &'a BytesStart<'a>) -> Result<HashMap<&'a [u8], Attribute<'a>>, Error> {
        event.attributes()
            .map(|r| r.map(|a| (a.key, a)).map_err(Error::from))
            .collect()
    }

    pub(crate) fn extract_attribute<'a>(event: &'a BytesStart<'a>, name: &str) -> Result<Option<Attribute<'a>>, Error> {
        event.attributes()
            .find(|r| r.is_err() || r.as_ref().ok().map_or(false, |a| a.key == name.as_bytes()))
            .transpose()
            .map_err(Error::from)
    }

    pub(crate) fn get_evidences<'a, B: BufRead>(reader: &mut Reader<B>, attr: &HashMap<&'a [u8], Attribute<'a>>) -> Result<Vec<usize>, Error> {
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
    pub(crate) fn decode_attribute<'a, B: BufRead, T: FromStr>(
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
}

// ---------------------------------------------------------------------------

/// A parser for the Uniprot XML format that parses entries iteratively.
pub struct UniprotParser<B: BufRead> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
    ignores: HashSet<Bytes>,
}

impl<B: BufRead> UniprotParser<B> {
    /// Make the parser ignore an `entry` element.
    ///
    /// This can be useful to speed-up the parser if you are only interested
    /// in particular elements, and you want the parser to only process these.
    /// *Note that element names must be given as they appear in the XML,
    /// like `organismHost` or `reference`, and not like they appear as fields
    /// of the `Entry` structure.*
    ///
    /// # Example
    /// ```rust
    /// # let reader = std::fs::File::open("tests/uniprot.xml")
    /// #     .map(std::io::BufReader::new).unwrap();
    /// for result in uniprot::parse(reader).ignore("feature") {
    ///     let entry = result.unwrap();
    ///     assert_eq!(entry.features.len(), 0);
    /// }
    /// ```
    pub fn ignore<K: Into<Bytes>>(&mut self, key: K) -> &mut Self {
        self.ignores.insert(key.into());
        self
    }
}

impl<B: BufRead> UniprotParser<B> {
    pub fn new(reader: B) -> UniprotParser<B> {
        let mut buffer = Vec::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // read until we enter the `uniprot` element
        let cache = loop {
            buffer.clear();
            match xml.read_event(&mut buffer) {
                Ok(Event::Start(ref e)) if e.local_name() == b"uniprot" => break None,
                Err(e) => break Some(Err(Error::from(e))),
                Ok(Event::Eof) => {
                    let e = String::from("xml");
                    break Some(Err(Error::from(XmlError::UnexpectedEof(e))));
                }
                _ => (),
            }
        };

        UniprotParser {
            xml,
            buffer,
            cache,
            finished: false,
            ignores: Default::default()
        }
    }
}

impl<B: BufRead> Iterator for UniprotParser<B> {
    type Item = Result<Entry, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        // return cached item if any
        if let Some(item) = self.cache.take() {
            return Some(item);
        }

        // if finished, simply return `None`
        if self.finished {
            return None;
        }

        // enter the next `entry` element
        loop {
            self.buffer.clear();
            match self.xml.read_event(&mut self.buffer) {
                // if an error is raised, return it
                Err(e) => return Some(Err(Error::from(e))),
                // error if reaching EOF
                Ok(Event::Eof) => {
                    let e = String::from("entry");
                    self.finished = true;
                    return Some(Err(Error::from(XmlError::UnexpectedEof(e))));
                }
                // if end of `uniprot` is reached, return no further item
                Ok(Event::End(ref e)) if e.local_name() == b"uniprot" => {
                    self.finished = true;
                    return None;
                },
                // create a new Entry
                Ok(Event::Start(ref e)) if e.local_name() == b"entry" => {
                    return Some(Entry::from_xml_ignoring(
                        &e.clone().into_owned(),
                        &mut self.xml,
                        &mut self.buffer,
                        &self.ignores
                    ));
                },
                _ => (),
            }
        };
    }
}

// -----------------------------------------------------------------------

pub(crate) trait FromXml: Sized {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error>;
}
