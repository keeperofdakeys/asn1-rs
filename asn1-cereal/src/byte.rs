//! Constructs for reading and writing bytes used by this crate.
use std::io;

#[inline]
/// Read a byte from an iterator, and translate Eof into an `UnexpectedEof` error.
pub fn read_byte<I: Iterator<Item=io::Result<u8>>>(iter: &mut I) -> io::Result<u8> {
  match iter.next() {
    Some(res) => res,
    None => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Got unexpected EOF while reading stream")),
  }
}

/// A byte reader that will count how many bytes are read from it,
/// and allows a limit to be defined. Any reads over the limit will
/// return `None` upon `next()` being called, and set the exceeded
/// flag.
pub struct ByteReader<I: Iterator<Item=io::Result<u8>>> {
  reader: I,
  pub count: u64,
  pub limit: Option<u64>,
  pub exceeded: bool,
}

impl<I: Iterator<Item=io::Result<u8>>> ByteReader<I> {
  /// Create a new ByteReader from an Iterator.
  pub fn new(reader: I, limit: Option<u64>) -> ByteReader<I> {
    ByteReader {
      reader: reader,
      count: 0,
      limit: limit,
      exceeded: false
    }
  }

  /// Trun this ByteReader back into the original reader used
  /// to create it.
  pub fn into_reader(self) -> I {
    self.reader
  }

  /// Determine whether this ByteReader has reached its defined
  /// limit. If no limit is defined, `false` is returned.
  pub fn reached_limit(&self) -> bool {
    self.limit.map(|l| self.count >= l).unwrap_or(false)
  }

  /// Read a byte, and translate Eof into an UnxpectedEof error.
  pub fn read(&mut self) -> io::Result<u8> {
    read_byte(self)
  }
}

impl<I: Iterator<Item=io::Result<u8>>> Iterator for ByteReader<I> {
  type Item = io::Result<u8>;

  fn next(&mut self) -> Option<Self::Item> {
    // If we've read limit or more bytes, set exceeded flag
    // and return None.
    if self.limit.map(|l| self.count >= l).unwrap_or(false) {
      self.exceeded = true;
      return None;
    }
    self.count += 1;
    self.reader.next()
  }
}

impl<I: Iterator<Item=io::Result<u8>>> From<I> for ByteReader<I> {
  fn from(iter: I) -> Self {
    ByteReader::new(iter, None)
  }
}

#[inline]
/// Write a byte to a writer, and return an error when nothing was written.
pub fn write_byte<W: io::Write>(writer: &mut W, byte: u8) -> io::Result<()> {
  let buf = [byte];
  match try!(writer.write(&buf)) {
    0 => Err(io::Error::new(io::ErrorKind::Other, "Wrote zero bytes")),
    1 => {
      Ok(())
    },
    _ => Err(io::Error::new(io::ErrorKind::Other, "Wrote more than one byte")),
  }
}

/// A writer to easily write a byte to a writer, while keeping a write count.
pub struct ByteWriter<W: io::Write> {
  writer: W,
  pub count: u64,
}

impl<W: io::Write> ByteWriter<W> {
  pub fn new(writer: W) -> ByteWriter<W> {
    ByteWriter {
      writer: writer,
      count: 0
    }
  }

  /// Write a byte, failing if no data was written.
  pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
    write_byte(self, byte)
  }
}

impl<W: io::Write> io::Write for ByteWriter<W> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let res = self.writer.write(buf);
    if let Ok(c) = res {
      self.count += c as u64;
    }
    res
  }

  fn flush(&mut self) -> io::Result<()> {
    self.writer.flush()
  }
}
