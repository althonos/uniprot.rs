use std::collections::HashSet;
use std::io::BufRead;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crossbeam_channel::Sender;
use quick_xml::Error as XmlError;

use super::Buffer;
use crate::error::Error;

#[cfg(feature = "threading")]
#[derive(Debug, PartialEq, Eq)]
/// The state of the `Producer`.
enum State {
    Started,
    Reading,
    Finished,
}

#[cfg(feature = "threading")]
pub struct Producer<B> {
    reader: Option<B>,
    threads: usize,
    s_text: Sender<Option<Result<Buffer, Error>>>,
    alive: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl<B: BufRead + Send + 'static> Producer<B> {
    pub(super) fn new(
        reader: B,
        threads: usize,
        s_text: Sender<Option<Result<Buffer, Error>>>,
    ) -> Self {
        Self {
            reader: Some(reader),
            s_text,
            threads,
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        self.alive.store(true, Ordering::SeqCst);

        let alive = self.alive.clone();
        let threads = self.threads;
        let mut reader = self.reader.take().unwrap();
        let mut s_text = self.s_text.clone();

        self.handle = Some(std::thread::spawn(move || {
            let mut buffer = vec![0; 8192];
            let mut buffer_start = 0;
            let mut buffer_end = 0;
            let mut buffer_entries = 0;

            loop {
                match reader.read(&mut buffer[buffer_end..]) {
                    Err(e) => {
                        s_text.send(Some(Err(Error::from(e)))).ok();
                        break;
                    }
                    Ok(n) => {
                        buffer_end += n;
                        let mut i = None;
                        for x in memchr::memchr_iter(b'<', &buffer[..buffer_end]) {
                            if buffer[x..buffer_end].starts_with(b"<entry") {
                                i = Some(x);
                                break;
                            }
                        }
                        let mut j = None;
                        for y in memchr::memrchr_iter(b'>', &buffer[..buffer_end]) {
                            if buffer[..=y].ends_with(b"</entry>") {
                                j = Some(y);
                                break;
                            }
                        }

                        if let Some(i) = i {
                            if let Some(j) = j {
                                s_text
                                    .send(Some(Ok(Buffer {
                                        data: Arc::new(buffer[i..j].to_vec()),
                                        range: 0..j - i,
                                    })))
                                    .ok();
                                buffer_start = j + 1;
                            } else {
                                if n == 0 {
                                    let name = String::from("entry");
                                    let err = Error::from(XmlError::UnexpectedEof(name));
                                    s_text.send(Some(Err(err))).ok();
                                }
                            }
                        }

                        if buffer_start > 0 {
                            buffer.copy_within(buffer_start..buffer_end, 0);
                            buffer_end -= buffer_start;
                            buffer_start = 0;
                        }
                        if buffer_end == buffer.len() {
                            buffer.resize(buffer.len() * 2, 0);
                        }
                        if n == 0 {
                            break;
                        }
                    }
                }
            }

            for _ in 0..threads {
                s_text.send(None).ok();
            }
            alive.store(false, Ordering::SeqCst);

            // let mut buffer = Vec::new();
            // let mut state = State::Started;
            // loop {
            //     match state {
            //         State::Started => match reader.read_until(b'>', &mut buffer) {
            //             // we reached EOF, but that's okay, we were not
            //             // reading an entry;
            //             Ok(0) => {
            //                 state = State::Finished;
            //             }
            //             // we found the beginning of an entry, now we
            //             // must read the entire entry until the end.
            //             Ok(_) => {
            //                 let i = memchr::memrchr(b'<', &buffer).unwrap();
            //                 if buffer[i..].starts_with(b"<entry") {
            //                     state = State::Reading;
            //                 }
            //             }
            //             // if an error is encountered, send it and bail out
            //             Err(e) => {
            //                 s_text.send(Some(Err(Error::from(e)))).ok();
            //                 state = State::Finished;
            //             }
            //         },
            //         State::Reading => {
            //             // read until the end of the entry.
            //             match reader.read_until(b'>', &mut buffer) {
            //                 // if a full entry is found, send it
            //                 Ok(_) if buffer.ends_with(&b"</entry>"[..]) => {
            //                     s_text.send(Some(Ok(buffer.as_slice().to_vec()))).ok();
            //                     state = State::Started;
            //                     buffer.clear();
            //                 }
            //                 // if we reach EOF before finding the end of the
            //                 // entry, that's an issue, we report an error.
            //                 Ok(0) => {
            //                     s_text
            //                         .send(Some(Err(Error::from(XmlError::UnexpectedEof(
            //                             String::from("entry"),
            //                         )))))
            //                         .ok();
            //                     state = State::Finished;
            //                 }
            //                 // if an error is encountered, send it and bail out
            //                 Err(e) => {
            //                     s_text.send(Some(Err(Error::from(e)))).ok();
            //                     state = State::Finished;
            //                 }
            //                 // otherwise just keep iterating.
            //                 _ => (),
            //             }
            //         }
            //         State::Finished => {
            //             for _ in 0..threads {
            //                 s_text.send(None).ok();
            //             }
            //             alive.store(false, Ordering::SeqCst);
            //             break;
            //         }
            //     }
            // }
        }));
    }

    pub fn join(&mut self) -> std::thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
        } else {
            Ok(())
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}
