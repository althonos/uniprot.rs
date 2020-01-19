use std::collections::HashSet;
use std::io::BufRead;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::thread::Result as ThreadResult;
use std::time::Duration;
use std::io::Error as IoError;

use bytes::Bytes;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::TryRecvError;
use crossbeam_channel::RecvTimeoutError;
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;

use crate::error::Error;
use crate::model::Entry;
use crate::model::Dataset;
use crate::parser::FromXml;

pub struct Consumer {
    r_text: Receiver<Option<Vec<u8>>>,
    s_buff: Sender<Vec<u8>>,
    s_item: Sender<Result<Entry, Error>>,
    alive: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Consumer {
    pub(super) fn new(
        r_text: Receiver<Option<Vec<u8>>>,
        s_item: Sender<Result<Entry, Error>>,
        s_buff: Sender<Vec<u8>>,
    ) -> Self {
        Self {
            r_text,
            s_buff,
            s_item,
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        self.alive.store(true, Ordering::SeqCst);

        let s_item = self.s_item.clone();
        let s_buff = self.s_buff.clone();
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
                        let e = Entry::from_xml(&s.into_owned(), &mut xml, &mut buffer);
                        s_item.send(e).ok();
                    }
                    _ => unreachable!("unexpected XML event"),
                }

                // send the buffer back to the consumer so it can be reused
                if s_buff.send(text).is_err() {
                    alive.store(false, Ordering::SeqCst);
                    return;
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
