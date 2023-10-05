//! XML parser implementation.
//!
//! This module provides two parsers, one using multithreading to consume
//! the input, and another one performing everything in the main thread.
//! The multithreaded parser is about twice as fast, but does not guarantee
//! the `Entry` are yielded in the same order as they appear in the source
//! XML file.
//!
//! Some benchmarks results on an i7-8550U CPU running at 1.80GHz, where the
//! baseline only collect [`quick-xml`] events without deserializing them into
//! the appropriate owned types from [`::uniprot`]:
//!
//! ```text
//! test bench_baseline          ... bench:  33,280,101 ns/iter (+/- 1,154,274) = 119 MB/s
//! test bench_sequential_parser ... bench:  53,509,244 ns/iter (+/- 5,700,458) = 74 MB/s
//! test bench_threaded_parser   ... bench:  27,767,008 ns/iter (+/- 6,527,132) = 143 MB/s
//! ```
//!
//! [`::uniprot`]: ../uniprot/index.html
//! [`quick-xml`]: https://docs.rs/quick-xml

pub(crate) mod utils;

#[cfg(feature = "threading")]
mod consumer;
#[cfg(feature = "threading")]
mod producer;
#[macro_use]
mod macros;

use std::collections::HashSet;
use std::io::BufRead;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

#[cfg(feature = "threading")]
use crossbeam_channel::Receiver;
#[cfg(feature = "threading")]
use crossbeam_channel::RecvTimeoutError;
#[cfg(feature = "threading")]
use crossbeam_channel::Sender;
#[cfg(feature = "threading")]
use crossbeam_channel::TryRecvError;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;
use quick_xml::Reader;

use super::error::Error;

#[cfg(feature = "threading")]
use self::consumer::Consumer;
#[cfg(feature = "threading")]
use self::producer::Producer;

// ---------------------------------------------------------------------------

#[allow(unused)]
const SLEEP_DURATION: Duration = Duration::from_millis(10);

// ---------------------------------------------------------------------------

#[cfg(feature = "threading")]
#[derive(Debug, PartialEq, Eq)]
/// The state of the `ThreadedParser`.
enum State {
    Idle,
    Started,
    Finished,
}

#[cfg(feature = "threading")]
/// A parser for the Uniprot XML formats that parses entries in parallel.
pub struct ThreadedParser<B: BufRead, D: UniprotDatabase> {
    state: State,
    threads: usize,
    producer: Producer<B>,
    consumers: Vec<Consumer<D>>,
    r_item: Receiver<Result<D::Entry, Error>>,
}

#[cfg(feature = "threading")]
impl<B: BufRead + Send + 'static, D: UniprotDatabase> ThreadedParser<B, D> {
    /// Create a new `ThreadedParser` using all available CPUs.
    ///
    /// This number of threads is extracted at runtime using the
    /// [`num_cpus::get`] function, which returns the number of *virtual*
    /// CPUs, that can differ from the number of physical CPUs if the
    /// processor supports hyperthreading.
    ///
    /// [`num_cpus::get`]: https://docs.rs/num_cpus/1.12.0/num_cpus/fn.get.html
    pub fn new(reader: B) -> Self {
        lazy_static! {
            static ref THREADS: usize = num_cpus::get();
        }
        let threads = unsafe { NonZeroUsize::new_unchecked(*THREADS) };
        Self::with_threads(reader, threads)
    }

    /// Create a new `ThreadedParser` with the requested number of threads.
    ///
    /// This function can be useful to fine tune the number of threads to use,
    /// or in the case of a program to allow the number of threads to be given
    /// as an argument (like `make` with the `-j` flag).
    ///
    /// Note that at least one thread is always going to be spawned; use the
    /// [`SequentialParser`](./struct.SequentialParser.html) instead to keep
    /// everything in the main thread.
    pub fn with_threads(reader: B, threads: NonZeroUsize) -> Self {
        let threads = threads.get();
        let mut buffer = Vec::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // create the communication channels
        let (s_text, r_text) = crossbeam_channel::bounded(threads);
        let (s_item, r_item) = crossbeam_channel::bounded(threads);

        // read until we enter the root element
        loop {
            buffer.clear();
            match xml.read_event_into(&mut buffer) {
                Ok(Event::Start(e)) if D::ROOTS.contains(&e.local_name().as_ref()) => {
                    break;
                }
                Ok(Event::Start(e)) => {
                    let x = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                    s_item
                        .send(Err(Error::UnexpectedRoot(x)))
                        .expect("channel should still be connected");
                    break;
                }
                Err(e) => {
                    s_item
                        .send(Err(Error::from(e)))
                        .expect("channel should still be connected");
                    break;
                }
                Ok(Event::Eof) => {
                    let e = String::from("xml");
                    s_item
                        .send(Err(Error::from(XmlError::UnexpectedEof(e))))
                        .expect("channel should still be connected");
                    break;
                }
                _ => (),
            }
        }

        // create the worker threads
        let producer = Producer::new(xml.into_inner(), threads, s_text);
        let mut consumers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let consumer = Consumer::new(r_text.clone(), s_item.clone());
            consumers.push(consumer);
        }

        // return the parser
        Self {
            r_item,
            producer,
            threads,
            consumers,
            state: State::Idle,
        }
    }
}

#[cfg(feature = "threading")]
impl<B: BufRead + Send + 'static, D: UniprotDatabase> Iterator for ThreadedParser<B, D> {
    type Item = Result<D::Entry, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::Idle => {
                    self.state = State::Started;
                    self.producer.start();
                    for consumer in &mut self.consumers {
                        consumer.start();
                    }
                }
                State::Finished => {
                    self.producer.join().unwrap();
                    for consumer in &mut self.consumers {
                        consumer.join().unwrap();
                    }
                    return None;
                }
                State::Started => {
                    // poll for parsed entries to return
                    match self.r_item.recv_timeout(SLEEP_DURATION) {
                        // item is found: simply return it
                        Ok(item) => return Some(item),
                        // empty queue: check if the producer is finished
                        Err(RecvTimeoutError::Timeout) => {
                            if !self.producer.is_alive() {
                                self.state = State::Finished
                            }
                        },
                        // queue was disconnected: stop and return an error
                        Err(RecvTimeoutError::Disconnected) => {
                            if self.state != State::Finished {
                                self.state = State::Finished;
                                return Some(Err(Error::DisconnectedChannel));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "threading")]
/// The parser type for the crate, used by `uniprot::parse`.
pub type Parser<B, D> = ThreadedParser<B, D>;

// --------------------------------------------------------------------------

/// A parser for the Uniprot XML formats that parses entries sequentially.
pub struct SequentialParser<B: BufRead, D: UniprotDatabase> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
    root: Vec<u8>,
}

impl<B: BufRead, D: UniprotDatabase> SequentialParser<B, D> {
    /// Create a new `SequentialParser` wrapping the given reader.
    pub fn new(reader: B) -> Self {
        let mut root = Vec::new();
        let mut buffer = Vec::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // read until we enter the `uniprot` element
        let cache = loop {
            buffer.clear();
            match xml.read_event_into(&mut buffer) {
                Err(e) => break Some(Err(Error::from(e))),
                Ok(Event::Start(e)) if D::ROOTS.contains(&e.local_name().as_ref()) => {
                    root.extend(e.local_name().as_ref());
                    break None;
                }
                Ok(Event::Start(e)) => {
                    let x = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                    break Some(Err(Error::UnexpectedRoot(x)));
                }
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
            root,
        }
    }

    /// Parse a single entry from the given reader.
    pub fn parse_entry(reader: B) -> <Self as Iterator>::Item {
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);
        let mut parser = Self {
            xml,
            buffer: Vec::new(),
            cache: None,
            finished: false,
            root: Vec::new(),
        };

        parser.next().unwrap_or_else(|| {
            let e = String::from("xml");
            Err(Error::from(XmlError::UnexpectedEof(e)))
        })
    }
}

impl<B: BufRead, D: UniprotDatabase> Iterator for SequentialParser<B, D> {
    type Item = Result<D::Entry, Error>;
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
            match self.xml.read_event_into(&mut self.buffer) {
                // if an error is raised, return it
                Err(e) => return Some(Err(Error::from(e))),
                // error if reaching EOF
                Ok(Event::Eof) => {
                    let e = String::from("entry");
                    self.finished = true;
                    return Some(Err(Error::from(XmlError::UnexpectedEof(e))));
                }
                // if end of `uniprot` is reached, return no further item
                Ok(Event::End(ref e)) if e.local_name().as_ref() == &self.root => {
                    self.finished = true;
                    return None;
                }
                // create a new Entry
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"entry" => {
                    return Some(D::Entry::from_xml(
                        &e.clone().into_owned(),
                        &mut self.xml,
                        &mut self.buffer,
                    ));
                }
                _ => (),
            }
        }
    }
}

#[cfg(not(feature = "threading"))]
/// The parser type for the crate, used by `uniprot::parse`.
pub type Parser<B, D> = SequentialParser<B, D>;

// ---------------------------------------------------------------------------

/// A trait for types that can be parsed from an XML element.
pub trait FromXml: Sized {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error>;
}

/// A trait for UniProt databases.
pub trait UniprotDatabase {
    type Entry: FromXml + Send + 'static;
    const ROOTS: &'static [&'static [u8]];
}
