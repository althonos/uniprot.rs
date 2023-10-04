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

#[cfg(feature = "threading")]
#[derive(Debug, PartialEq, Eq)]
/// The state of the `Producer`.
enum State {
    Started,
    Reading,
    AtEof,
    Finished,
}


#[cfg(feature = "threading")]
pub struct Producer<B> {
    reader: Option<B>,
    threads: usize,
    s_text: Sender<Option<Vec<u8>>>,
    alive: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl<B: BufRead + Send + 'static> Producer<B> {

    pub(super) fn new(
        reader: B,
        threads: usize,
        s_text: Sender<Option<Vec<u8>>>,
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
            let mut buffer = Vec::new();
            let mut state = State::Started;
            loop {
                match state {
                    State::Started => match reader.read_until(b'>', &mut buffer) {
                        // we reached EOF, but that's okay, we were not
                        // reading an entry;
                        Ok(0) => {
                            for _ in 0..threads {
                                s_text.send(None).ok();
                            }
                            state = State::AtEof;
                        }
                        // we found the beginning of an entry, now we
                        // must read the entire entry until the end.
                        Ok(_) => {
                            let i = memchr::memrchr(b'<', &buffer).unwrap();
                            if buffer[i..].starts_with(b"<entry") {
                                state = State::Reading;
                            }
                        }
                        // if an error is encountered, send it and bail out
                        Err(e) => {
                            panic!("XML error");
                            state = State::Finished;
                            // return Some(Err(Error::from(e)));
                        }
                    }
                    State::Reading => {
                        // read until the end of the entry.
                        match reader.read_until(b'>', &mut buffer) {
                            // if a full entry is found, send it
                            Ok(_) if buffer.ends_with(&b"</entry>"[..]) => {
                                s_text
                                    .send(Some(std::mem::take(&mut buffer)))
                                    .ok();
                                state = State::Started;
                            }
                            // if we reach EOF before finding the end of the
                            // entry, that's an issue, we report an error.
                            Ok(0) => {
                                for _ in 0..threads {
                                    s_text.send(None).ok();
                                }
                                state = State::AtEof;
                                panic!("unexpected EOF");
                                // return Some(Err(Error::from(XmlError::UnexpectedEof(String::from(
                                //     "entry",
                                // )))));
                            }
                            // if an error is encountered, send it and bail out
                            Err(e) => {
                                state = State::Finished;
                                panic!("XML error");
                                // return Some(Err(Error::from(e)));
                            }
                            // otherwise just keep iterating.
                            _ => (),
                        }
                    }                    
                    State::AtEof | State::Finished => break,
                    _ => unimplemented!(),
                }
            }
            alive.store(false, Ordering::SeqCst);
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