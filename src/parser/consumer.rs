use std::collections::HashSet;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Error as IoError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::thread::Result as ThreadResult;
use std::time::Duration;

use crossbeam_channel::Receiver;
use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::Sender;
use crossbeam_channel::TryRecvError;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::FromXml;

pub struct Consumer<E: FromXml + Send + 'static> {
    r_text: Receiver<Option<Vec<u8>>>,
    s_item: Sender<Result<E, Error>>,
    alive: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl<E: FromXml + Send + 'static> Consumer<E> {
    pub(super) fn new(
        r_text: Receiver<Option<Vec<u8>>>,
        s_item: Sender<Result<E, Error>>,
    ) -> Self {
        Self {
            r_text,
            s_item,
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        self.alive.store(true, Ordering::SeqCst);

        let s_item = self.s_item.clone();
        let r_text = self.r_text.clone();
        let alive = self.alive.clone();

        self.handle = Some(std::thread::spawn(move || {
            let mut buffer = Vec::new();
            loop {
                // get the buffer containing the XML entry
                let text = loop {
                    match r_text.recv_timeout(Duration::from_micros(1)) {
                        Ok(Some(text)) => break text,
                        Ok(None) => {
                            alive.store(false, Ordering::SeqCst);
                            return;
                        }
                        Err(RecvTimeoutError::Timeout) => (),
                        Err(RecvTimeoutError::Disconnected) => {
                            alive.store(false, Ordering::SeqCst);
                            return;
                        }
                    }
                };

                // parse the XML file and send the result to the main thread
                let mut xml = Reader::from_reader(Cursor::new(&text));
                xml.expand_empty_elements(true).trim_text(true);
                match xml.read_event(&mut buffer) {
                    Err(e) => {
                        s_item.send(Err(Error::from(e))).ok();
                        return;
                    }
                    Ok(Event::Eof) => {
                        let name = String::from("entry");
                        let err = Error::from(XmlError::UnexpectedEof(name));
                        s_item.send(Err(err)).ok();
                        return;
                    }
                    Ok(Event::Start(s)) if s.local_name() == b"entry" => {
                        let e = E::from_xml(&s.into_owned(), &mut xml, &mut buffer);
                        s_item.send(e).ok();
                    }
                    e => unreachable!("unexpected XML event: {:?}", e),
                }

                // clear the event buffer
                buffer.clear();
            }
        }));
    }

    pub fn join(&mut self) -> std::thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
        } else {
            Ok(())
        }
    }
}
