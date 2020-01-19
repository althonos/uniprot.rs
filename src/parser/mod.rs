//! UniprotKB XML parser implementation.
//!
//! This module provides two parsers, one using multithreading to consume
//! the input, and another one performing everything in the main thread.
//! The multithreaded parser is about twice as fast, but does not guarantee
//! the `Entry` are yielded in the same order as they appear in the source
//! XML file.
//!
//! Some benchmarks results on an i7-8550U CPU running at 1.80GHz, where the
//! baseline only collect [`quick-xml`] events without deserializing them into
//! the appropriate owned types from [`::model`]:
//!
//! ```text
//! test bench_baseline          ... bench:  33,280,101 ns/iter (+/- 1,154,274) = 119 MB/s
//! test bench_sequential_parser ... bench:  53,509,244 ns/iter (+/- 5,700,458) = 74 MB/s
//! test bench_threaded_parser   ... bench:  27,767,008 ns/iter (+/- 6,527,132) = 143 MB/s
//! ```
//!
//! [`::model`]: ../model/index.html
//! [`quick-xml`]: https://docs.rs/quick-xml

pub(crate) mod utils;

#[cfg(feature = "threading")]
mod consumer;
#[macro_use]
mod macros;

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
use crossbeam_channel::Sender;
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
lazy_static !{
    /// The number of threads used for parsing.
    ///
    /// Note that one extra thread is spawned simply to consume the buffered
    /// reader; the other threads will parse the resulting bytes into proper
    /// entries.
    pub static ref THREADS: usize = num_cpus::get();
}

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
#[derive(PartialEq, Eq)]
/// The state of the `ThreadedParser`.
enum State {
    Idle,
    Started,
    AtEof,
    Waiting,
    Finished,
}

#[cfg(feature = "threading")]
/// A parser for the Uniprot XML format that parses entries iteratively.
pub struct ThreadedParser<B: BufRead> {
    // producer: Producer<B>,
    reader: B,
    consumers: Vec<Consumer>,
    r_item:  Receiver<Result<Entry, Error>>,
    r_buff: Receiver<Vec<u8>>,
    s_text: Sender<Option<Vec<u8>>>,
    state: State,
}

#[cfg(feature = "threading")]
impl<B: BufRead> ThreadedParser<B> {
    pub fn new(reader: B) -> Self {
        let mut buffer = Vec::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // create the communication channels
        let (s_buff, r_buff) = crossbeam_channel::bounded(*THREADS);
        let (s_text, r_text) = crossbeam_channel::bounded(*THREADS);
        let (s_item, r_item) = crossbeam_channel::bounded(*THREADS);

        // read until we enter the `uniprot` element
        loop {
            buffer.clear();
            match xml.read_event(&mut buffer) {
                Ok(Event::Start(ref e)) if e.local_name() == b"uniprot" => break,
                Err(e) => {
                    s_item.send(Err(Error::from(e)))
                        .expect("channel should still be connected");
                    break;
                }
                Ok(Event::Eof) => {
                    let e = String::from("xml");
                    s_item.send(Err(Error::from(XmlError::UnexpectedEof(e))))
                        .expect("channel should still be connected");
                    break;
                }
                _ => (),
            }
        };

        // create the consumer and the workers
        // let producer = Producer::new(xml.into_underlying_reader(), s1, r0);
        let mut consumers = Vec::with_capacity(*THREADS);
        for _ in 0..*THREADS {
            let consumer = Consumer::new(r_text.clone(), s_item.clone(), s_buff.clone());
            consumers.push(consumer);
            s_buff.send(Vec::new()).unwrap();
        }

        // return the parser
        Self {
            // producer,
            consumers,
            reader: xml.into_underlying_reader(),
            state: State::Idle,
            r_item: r_item,
            r_buff: r_buff,
            s_text: s_text,
        }
    }
}

#[cfg(feature = "threading")]
impl<B: BufRead> Iterator for ThreadedParser<B> {
    type Item = Result<Entry, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // poll for parsed entries to return
            match self.r_item.try_recv() {
                // item is found: simply return it
                Ok(item) => return Some(item),
                // empty queue after all the threads were joined: we are done
                Err(TryRecvError::Empty) if self.state == State::Waiting => {
                    self.state = State::Finished;
                    return None;
                }
                // empty queue in any other state: just do something else
                Err(TryRecvError::Empty) => (),
                // queue was disconnected: stop and return an error
                Err(TryRecvError::Disconnected) => {
                    if self.state != State::Finished {
                        self.state = State::Finished;
                        return Some(Err(Error::DisconnectedChannel));
                    }
                }
            }

            // depending on the state, do something before polling
            match self.state {
                State::Started => {
                    let mut buffer = self.r_buff.recv().unwrap();
                    buffer.clear();
                    loop {
                        match self.reader.read_until(b'>', &mut buffer) {
                            // if reached EOF, bail out
                            Ok(0) => {
                                for _ in 0..*THREADS {
                                    self.s_text.send(None).ok();
                                }
                                self.state = State::AtEof;
                                break;
                            }
                            // if a full entry is found, send it
                            Ok(_) if buffer.ends_with(&b"</entry>"[..]) => {
                                self.s_text.send(Some(buffer)).ok();
                                break;
                            }
                            // if an error is encountered, send it and bail out
                            Err(e) => {
                                self.state = State::Finished;
                                return Some(Err(Error::from(e)));
                            }
                            _ => (),
                        }
                    }
                }
                State::AtEof => {
                    self.state = State::Waiting;
                    for consumer in self.consumers.iter_mut() {
                        consumer.join().unwrap();
                    }
                }
                State::Idle => {
                    self.state = State::Started;
                    for consumer in &mut self.consumers {
                        consumer.start();
                    }
                }
                State::Finished => {
                    return None;
                }
                State::Waiting => {
                    std::thread::sleep(Duration::from_micros(1));
                }
            }
        }
    }
}

#[cfg(feature = "threading")]
/// The parser type for the crate, used by `uniprot::parse`.
pub type Parser<B> = ThreadedParser<B>;

// --------------------------------------------------------------------------

/// A parser for the Uniprot XML format that processes everything in the main thread.
pub struct SequentialParser<B: BufRead> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
}

impl<B: BufRead> SequentialParser<B> {
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

impl<B: BufRead> Iterator for SequentialParser<B> {
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
pub type Parser<B> = SequentialParser<B>;

// ---------------------------------------------------------------------------
