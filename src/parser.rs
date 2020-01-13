use std::collections::HashSet;
use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;

use super::model::*;
use super::error::Error;

macro_rules! parse_inner {
    ($start:expr, $reader:expr, $buffer:expr, $( $e:ident @ $l:expr => $r:expr ),*  ) => ({
        loop {
            use $crate::quick_xml::events::BytesEnd;
            use $crate::quick_xml::events::BytesStart;
            use $crate::quick_xml::events::Event;
            use $crate::quick_xml::Error as XmlError;

            $buffer.clear();
            match $reader.read_event($buffer) {
                $( Ok(Event::Start(ref e)) if e.local_name() == $l => {
                    let $e = e.clone().into_owned();
                    $r
                }),*
                Ok(Event::Start(ref e)) => {
                    $reader.read_to_end(e.local_name(), &mut Vec::new())?;
                    // unimplemented!(
                    //     "`{}` in `{}`",
                    //     String::from_utf8_lossy(e.local_name()),
                    //     String::from_utf8_lossy($start.local_name())
                    // );
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(Event::Eof) => {
                    let e = String::from_utf8_lossy($start.local_name());
                    return Err(XmlError::UnexpectedEof(e.to_string()));
                }
                Ok(Event::End(ref e)) if e.local_name() == $start.local_name() => {
                    break;
                }
                _ => continue,
            }
        }
    })
}

// ---------------------------------------------------------------------------

pub(crate) mod utils {
    use std::collections::HashMap;
    use std::io::BufRead;
    use std::str::FromStr;

    use quick_xml::Reader;
    use quick_xml::Error as XmlError;
    use quick_xml::events::attributes::Attribute;
    use quick_xml::events::BytesStart;

    pub(crate) fn attributes_to_hashmap<'a>(b: &'a BytesStart<'a>) -> Result<HashMap<&'a [u8], Attribute<'a>>, XmlError> {
        b.attributes().map(|r| r.map(|a| (a.key, a))).collect()
    }

    pub(crate) fn get_evidences<'a, B: BufRead>(reader: &mut Reader<B>, attr: &HashMap<&'a [u8], Attribute<'a>>) -> Result<Vec<usize>, XmlError> {
        Ok(attr.get(&b"evidence"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?
            .map(|e| e.split(' ').map(usize::from_str).collect::<Result<Vec<_>, _>>())
            .transpose()
            .expect("ERR: could not decode evidence number")
            .unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------

pub struct UniprotParser<B: BufRead> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
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
                Err(e) => break Some(Err(e)),
                Ok(Event::Eof) => {
                    let e = String::from("xml");
                    break Some(Err(XmlError::UnexpectedEof(e)));
                }
                _ => (),
            }
        };

        UniprotParser {
            xml,
            buffer,
            cache,
            finished: false,
        }
    }
}

impl<B: BufRead> Iterator for UniprotParser<B> {
    type Item = Result<Entry, XmlError>;
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
                Err(e) => return Some(Err(e)),
                // error if reaching EOF
                Ok(Event::Eof) => {
                    let e = String::from("entry");
                    return Some(Err(XmlError::UnexpectedEof(e)));
                }
                // if end of `uniprot` is reached, return no further item
                Ok(Event::End(ref e)) if e.local_name() == b"uniprot" => {
                    self.finished = true;
                    return None;
                },
                // create a new Entry
                Ok(Event::Start(ref e)) if e.local_name() == b"entry" => {
                    let event = e.clone().into_owned();
                    return Entry::from_xml(&event, &mut self.xml, &mut self.buffer)
                        .map(Some).transpose();
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
