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
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use bytes::Bytes;
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

// ---------------------------------------------------------------------------

#[cfg(feature = "threading")]
#[derive(Debug, PartialEq, Eq)]
/// The state of the `ThreadedParser`.
enum State {
    Idle,
    Started,
    Reading,
    AtEof,
    Waiting,
    Finished,
}

#[cfg(feature = "threading")]
/// A parser for the Uniprot XML format that parses entries in parallel.
pub struct ThreadedParser<B: BufRead, E: FromXml + Send + 'static> {
    reader: B,
    state: State,
    threads: usize,
    consumers: Vec<Consumer<E>>,
    r_item: Receiver<Result<E, Error>>,
    s_text: Sender<Option<Vec<u8>>>,
    buffer: Vec<u8>,
}

#[cfg(feature = "threading")]
impl<B: BufRead, E: FromXml + Send + 'static> ThreadedParser<B, E> {
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
            match xml.read_event(&mut buffer) {
                Ok(Event::Start(_)) => {
                    break;
                },
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

        // create the consumer and the workers
        let mut consumers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let consumer = Consumer::new(r_text.clone(), s_item.clone());
            consumers.push(consumer);
        }

        // return the parser
        Self {
            r_item,
            s_text,
            threads,
            consumers,
            reader: xml.into_underlying_reader(),
            state: State::Idle,
            buffer: Vec::new(),
        }
    }
}

#[cfg(feature = "threading")]
impl<B: BufRead, E: FromXml + Send + 'static> Iterator for ThreadedParser<B, E> {
    type Item = Result<E, Error>;
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
                    // read until a new entry is found, or exit
                    match self.reader.read_until(b'>', &mut self.buffer) {
                        // we reached EOF, but that's okay, we were not
                        // reading an entry;
                        Ok(0) => {
                            for _ in 0..self.threads {
                                self.s_text.send(None).ok();
                            }
                            self.state = State::AtEof;
                        }
                        // we found the beginning of an entry, now we
                        // must read the entire entry until the end.
                        Ok(_) => {
                            let i = memchr::memrchr(b'<', &self.buffer).unwrap();
                            if self.buffer[i..].starts_with(b"<entry") {
                                self.state = State::Reading;
                            }
                        }
                        // if an error is encountered, send it and bail out
                        Err(e) => {
                            self.state = State::Finished;
                            return Some(Err(Error::from(e)));
                        }
                    }
                }
                State::Reading => {
                    // read until the end of the entry.
                    match self.reader.read_until(b'>', &mut self.buffer) {
                        // if a full entry is found, send it
                        Ok(_) if self.buffer.ends_with(&b"</entry>"[..]) => {
                            self.s_text.send(Some(std::mem::take(&mut self.buffer))).ok();
                            self.state = State::Started;
                        }
                        // if we reach EOF before finding the end of the
                        // entry, that's an issue, we report an error.
                        Ok(0) => {
                            for _ in 0..self.threads {
                                self.s_text.send(None).ok();
                            }
                            self.state = State::AtEof;
                            return Some(Err(Error::from(XmlError::UnexpectedEof(String::from("entry")))));
                        }
                        // if an error is encountered, send it and bail out
                        Err(e) => {
                            self.state = State::Finished;
                            return Some(Err(Error::from(e)));
                        }
                        // otherwise just keep iterating.
                        _ => (),
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
pub type Parser<B, E> = ThreadedParser<B, E>;

// --------------------------------------------------------------------------

/// A parser for the Uniprot XML format that  parses entries sequentially.
pub struct SequentialParser<B: BufRead, E: FromXml> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
    root: Vec<u8>,
}

impl<B: BufRead, E: FromXml> SequentialParser<B, E> {
    /// Create a new `SequentialParser` wrapping the given reader.
    pub fn new(reader: B) -> Self {
        let mut root = Vec::new();
        let mut buffer = Vec::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // read until we enter the `uniprot` element
        let cache = loop {
            buffer.clear();
            match xml.read_event(&mut buffer) {
                Err(e) => break Some(Err(Error::from(e))),
                Ok(Event::Start(e)) => {
                    root.extend(e.local_name());
                    break None;
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
}

impl<B: BufRead, E: FromXml> Iterator for SequentialParser<B, E> {
    type Item = Result<E, Error>;
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
                Ok(Event::End(ref e)) if e.local_name() == &self.root => {
                    self.finished = true;
                    return None;
                }
                // create a new Entry
                Ok(Event::Start(ref e)) if e.local_name() == b"entry" => {
                    return Some(E::from_xml(
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
pub type Parser<B, E> = SequentialParser<B, E>;

// ---------------------------------------------------------------------------

pub trait FromXml: Sized {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error>;
}
