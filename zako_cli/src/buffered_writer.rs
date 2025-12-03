use std::cell::RefCell;
use std::io::{Stderr, StderrLock, Stdout, StdoutLock, Write};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

/// 一个临时缓冲写入器，先将数据写入内存缓冲区，
///
/// 直到调用 `release` 方法后，才将缓冲区的数据写入底层写入器。
#[derive(Clone,Debug)]
pub struct TemporaryBufferedWriterMaker {
    inner: Arc<Mutex<TemporaryBufferedWriter>>
}

#[derive(Debug)]
pub struct TemporaryBufferedWriter {
    buffer: Vec<u8>,
    buffer_released: bool,
    silent:bool,
    release_to: Stdout,
}

impl TemporaryBufferedWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(512),
            buffer_released: false,
            silent:false,
            release_to: std::io::stdout(),
        }
    }

    /// 这下彻底兜不住了，把缓冲区的内容写入底层 writer。
    pub fn release(&mut self) -> std::io::Result<()> {
        if !self.buffer_released {
            self.buffer_released = true;
            self.release_to.lock().write_all(&self.buffer)?;
            self.buffer.clear();
            self.buffer.shrink_to_fit();
        }
        Ok(())
    }

    pub fn silent(&mut self){
        self.silent = true;
        self.buffer.clear();
        self.buffer.shrink_to_fit();
    }
}

impl TemporaryBufferedWriterMaker{
    pub fn new() -> (Self,Arc<Mutex<TemporaryBufferedWriter>>){
        let write = Arc::new(Mutex::new(TemporaryBufferedWriter::new()));
        (Self { inner: write.clone() },write)
    }
}

impl<'a> MakeWriter<'a> for TemporaryBufferedWriterMaker {
    type Writer = TemporaryBufferedWriterMaker;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl Write for TemporaryBufferedWriterMaker {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut inner = self.inner.lock().unwrap();

        if inner.silent{
            return Ok(buf.len())
        }

        if inner.buffer_released {
            inner.release_to.lock().write(buf)
        } else {
            inner.buffer.extend_from_slice(buf);
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let inner = self.inner.lock().unwrap();

        if inner.silent{
            return Ok(());
        }

        if inner.buffer_released {
            inner.release_to.lock().flush()
        } else {
            Ok(())
        }
    }
}
