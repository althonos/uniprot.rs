use std::collections::HashSet;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Error as IoError;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;
use std::thread::Result as ThreadResult;

use bytes::Bytes;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::TryRecvError;
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;

use crate::error::Error;
use crate::model::Entry;
use crate::model::Dataset;
use crate::parser::FromXml;

use super::THREADS;
use super::Status;

pub struct Producer<B: BufRead + Send + 'static> {
    reader: Option<B>,
    handle: Option<JoinHandle<()>>,
    alive: Arc<AtomicBool>,

    // the queue to send text fully read
    text_sender: Sender<Status<Vec<u8>, IoError>>,
    // the queue to receive recycled buffers
    buffer_receiver: Receiver<Vec<u8>>,
}

impl<B: BufRead + Send + 'static> Producer<B> {
    pub(super) fn new(
        reader: B,
        text_sender: Sender<Status<Vec<u8>, IoError>>,
        buffer_receiver: Receiver<Vec<u8>>,
    ) -> Self {
        Self {
            reader: Some(reader),
            handle: None,
            alive: Arc::new(AtomicBool::new(false)),
            text_sender: text_sender,
            buffer_receiver: buffer_receiver,
        }
    }

    pub fn start(&mut self) {
        self.alive.store(true, Ordering::SeqCst);

        let alive = self.alive.clone();
        let text_sender = self.text_sender.clone();
        let buffer_receiver = self.buffer_receiver.clone();
        let mut reader = self.reader.take().unwrap();

        self.handle = Some(std::thread::spawn(move || {
            loop {
                let mut buffer = buffer_receiver.recv().unwrap();
                buffer.clear();
                loop {
                    match reader.read_until(b'>', &mut buffer) {
                        // if reached EOF, bail out
                        Ok(0) => {
                            alive.store(false, Ordering::SeqCst);
                            for _ in 0..THREADS {
                                text_sender.send(Status::Finished).ok();
                            }
                            return;
                        }
                        // if a full entry is found, send it
                        Ok(_) if buffer.ends_with(&b"</entry>"[..]) => {
                            text_sender.send(Status::Ok(buffer)).ok();
                            break;
                        }
                        // if an error is encountered, send it and bail out
                        Err(e) => {
                            text_sender.send(Status::Err(e)).ok();
                            alive.store(false, Ordering::SeqCst);
                            return;
                        }
                        _ => (),
                    }
                }
            }
        }));
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}
