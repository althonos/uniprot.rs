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
pub struct Producer<B> {
    reader: Option<B>,
    threads: usize,
    s_text: Sender<Option<Result<Buffer, Error>>>,
    alive: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
    buffer_size: usize,
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
            buffer_size: 65536, //8192,
        }
    }

    pub fn start(&mut self) {
        self.alive.store(true, Ordering::SeqCst);

        let buffer_size = self.buffer_size;
        let alive = self.alive.clone();
        let threads = self.threads;
        let s_text = self.s_text.clone();
        let mut reader = self.reader.take().unwrap();

        self.handle = Some(std::thread::spawn(move || {
            let mut buffer = vec![0; buffer_size];
            let mut buffer_start = 0;
            let mut buffer_end = 0;

            loop {
                match reader.read(&mut buffer[buffer_end..]) {
                    Err(e) => {
                        s_text.send(Some(Err(Error::from(e)))).ok();
                        break;
                    }
                    Ok(n) => {
                        buffer_end += n;
                        if let Some(i) = memchr::memchr_iter(b'<', &buffer[..buffer_end])
                            .find(|&x| buffer[x..buffer_end].starts_with(b"<entry"))
                        {
                            if let Some(j) = memchr::memrchr_iter(b'>', &buffer[i..buffer_end])
                                .map(|y| y + i)
                                .find(|&y| buffer[..=y].ends_with(b"</entry>"))
                            {
                                s_text
                                    .send(Some(Ok(Buffer {
                                        data: buffer[i..j].to_vec(),
                                    })))
                                    .ok();
                                buffer_start = j + 1;
                            } else if n == 0 {
                                let name = String::from("entry");
                                let err = Error::from(XmlError::UnexpectedEof(name));
                                s_text.send(Some(Err(err))).ok();
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
