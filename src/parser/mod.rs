//!

pub(crate) mod utils;

#[cfg(feature = "threading")]
mod consumer;
#[macro_use]
mod macros;
#[cfg(feature = "threading")]
mod producer;

use std::collections::HashSet;
use std::io::BufRead;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use bytes::Bytes;
#[cfg(feature = "threading")]
use crossbeam_channel::Receiver;
#[cfg(feature = "threading")]
use crossbeam_channel::RecvTimeoutError;
#[cfg(feature = "threading")]
use crossbeam_channel::TryRecvError;
use quick_xml::Reader;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;

use super::model::*;
use super::error::Error;

#[cfg(feature = "threading")]
use self::consumer::Consumer;
#[cfg(feature = "threading")]
use self::producer::Producer;

#[cfg(feature = "threading")]
/// The number of threads used for parsing.
///
/// Note that one extra thread is spawned simply to consume the buffered
/// reader; the other threads will parse the resulting bytes into proper
/// entries.
pub const THREADS: usize = 8;

// ---------------------------------------------------------------------------


// -----------------------------------------------------------------------

pub(crate) trait FromXml: Sized {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error>;
}

// ---------------------------------------------------------------------------

#[cfg(feature = "threading")]
/// A parser for the Uniprot XML format that parses entries iteratively.
pub struct MultiThreadedParser<B: BufRead + Send + 'static> {
    producer: Producer<B>,
    consumers: Vec<Consumer>,
    receiver: Receiver<Result<Entry, Error>>,
    finished: bool,
    started: bool,
}

#[cfg(feature = "threading")]
impl<B: BufRead + Send + 'static> MultiThreadedParser<B> {
    pub fn new(reader: B) -> Self {
        let mut buffer = Vec::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // create the communication channels
        let (s0, r0) = crossbeam_channel::bounded(THREADS);
        let (s1, r1) = crossbeam_channel::bounded(THREADS);
        let (s2, r2) = crossbeam_channel::bounded(THREADS);

        // read until we enter the `uniprot` element
        loop {
            buffer.clear();
            match xml.read_event(&mut buffer) {
                Ok(Event::Start(ref e)) if e.local_name() == b"uniprot" => break,
                Err(e) => {
                    s2.send(Err(Error::from(e)))
                        .expect("channel should still be connected");
                    break;
                }
                Ok(Event::Eof) => {
                    let e = String::from("xml");
                    s2.send(Err(Error::from(XmlError::UnexpectedEof(e))))
                        .expect("channel should still be connected");
                    break;
                }
                _ => (),
            }
        };

        // create the consumer and the workers
        let producer = Producer::new(xml.into_underlying_reader(), s1, r0);
        let mut consumers = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let consumer = Consumer::new(r1.clone(), s2.clone(), s0.clone());
            consumers.push(consumer);
            s0.send(Vec::new()).unwrap();
        }

        // return the parser
        Self {
            producer,
            consumers,
            finished: false,
            started: false,
            receiver: r2,
        }
    }
}

#[cfg(feature = "threading")]
impl<B: BufRead + Send + 'static> Iterator for MultiThreadedParser<B> {
    type Item = Result<Entry, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        // return None if the parser has already determined that it is
        // finished processing the input
        if self.finished {
            return None;
        }

        // launch the background threads if this is the first call
        // to `next` since the struct was created
        if !self.started {
            self.producer.start();
            for consumer in &mut self.consumers {
                consumer.start();
            }
            self.started = true;
        }

        // poll for parsed entries to return
        loop {
            // poll every 1µs up to 100µs
            for _ in 0..100 {
                match self.receiver.recv_timeout(Duration::from_micros(1)) {
                    Ok(item) => return Some(item),
                    Err(RecvTimeoutError::Timeout) => (),
                    Err(RecvTimeoutError::Disconnected) => {
                        self.finished = true;
                        return Some(Err(Error::DisconnectedChannel));
                    }
                }
            }

            // if after 100µs the queue is still empty, check threads are
            // still running
            match self.receiver.try_recv()  {
                Ok(item) => return Some(item),
                Err(TryRecvError::Empty) => {
                    if !self.producer.is_alive() && self.consumers.iter().all(|w| !w.is_alive()) {
                        self.finished = true;
                        return None;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.finished = true;
                    return Some(Err(Error::DisconnectedChannel));
                }
            }
        }
    }
}

#[cfg(feature = "threading")]
/// The parser type for the crate, used by `uniprot::parse`.
pub type Parser<B> = MultiThreadedParser<B>;

#[cfg(feature = "threading")]
pub trait XmlRead: BufRead + Send + 'static {}

#[cfg(feature = "threading")]
impl<B: BufRead + Send + 'static> XmlRead for B {}

// ---------------------------------------------------------------------------

/// A parser for the Uniprot XML format that processes everything in the main thread.
pub struct SingleThreadedParser<B: BufRead> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
}

impl<B: BufRead> SingleThreadedParser<B> {
    pub fn new(reader: B) -> Self {
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

        Self {
            xml,
            buffer,
            cache,
            finished: false,
        }
    }
}

impl<B: BufRead> Iterator for SingleThreadedParser<B> {
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
                    return Some(Entry::from_xml(
                        &e.clone().into_owned(),
                        &mut self.xml,
                        &mut self.buffer,
                    ));
                },
                _ => (),
            }
        };
    }
}

#[cfg(not(feature = "threading"))]
/// The parser type for the crate, used by `uniprot::parse`.
pub type Parser<B> = SingleThreadedParser<B>;

#[cfg(not(feature = "threading"))]
pub trait XmlRead: BufRead {}

#[cfg(not(feature = "threading"))]
impl<B: BufRead> XmlRead for B {}

// ---------------------------------------------------------------------------
