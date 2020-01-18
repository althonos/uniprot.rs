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

use super::utils::Status;

pub struct Consumer {
    text_receiver: Receiver<Status<Vec<u8>, IoError>>,
    buffer_sender: Sender<Vec<u8>>,
    item_sender: Sender<Result<Entry, Error>>,
    alive: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Consumer {
    pub(super) fn new(
        text_receiver: Receiver<Status<Vec<u8>, IoError>>,
        item_sender: Sender<Result<Entry, Error>>,
        buffer_sender: Sender<Vec<u8>>,
    ) -> Self {
        Self {
            text_receiver,
            buffer_sender,
            item_sender,
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        self.alive.store(true, Ordering::SeqCst);

        let item_sender = self.item_sender.clone();
        let buffer_sender = self.buffer_sender.clone();
        let text_receiver = self.text_receiver.clone();
        let alive = self.alive.clone();

        self.handle = Some(std::thread::spawn(move || {
            let mut buffer = Vec::new();
            loop {
                buffer.clear();

                // get the buffer containing the XML entry
                let text = loop {
                    match text_receiver.recv_timeout(Duration::from_micros(1)) {
                        Ok(Status::Ok(text)) => break text,
                        Ok(Status::Finished) => {
                            alive.store(false, Ordering::SeqCst);
                            return;
                        }
                        Ok(Status::Err(e)) => {
                            item_sender.send(Err(Error::from(e))).ok();
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
                        item_sender.send(Err(Error::from(e))).ok();
                        return;
                    }
                    Ok(Event::Eof) => {
                        let name = String::from("entry");
                        let err = Error::from(XmlError::UnexpectedEof(name));
                        item_sender.send(Err(err)).ok();
                        return;
                    }
                    Ok(Event::Start(s)) if s.local_name() == b"entry" => {
                        let e = Entry::from_xml(&s.into_owned(), &mut xml, &mut buffer);
                        item_sender.send(e).ok();
                    }
                    _ => unreachable!("unexpected XML event"),
                }

                // send the buffer back to the consumer so it can be reused
                if buffer_sender.send(text).is_err() {
                    alive.store(false, Ordering::SeqCst);
                    return;
                }
            }
        }));
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}
