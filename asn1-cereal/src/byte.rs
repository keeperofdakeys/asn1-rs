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

/// A reader to easily read a byte from a reader, while keeping a read count.
pub struct ByteReader<I: Iterator<Item=io::Result<u8>>> {
  reader: I,
  pub count: u64,
  limit: Option<u64>,
}

impl<I: Iterator<Item=io::Result<u8>>> ByteReader<I> {
  /// Create a new ByteReader from an Iterator.
  pub fn new(reader: I) -> ByteReader<I> {
    ByteReader {
      reader: reader,
      count: 0,
      limit: None
    }
  }

  /*
  /// Create a new ByteReader from an Iterator, and add
  /// a maximum length that can be read from it.
  pub fn new_limit(reader: I, limit: u64) -> ByteReader<I> {
    ByteReader {
      reader: reader,
      count: 0,
      limit: Some(limit),
    }
  }
  */

  /// Read a byte, and translate Eof into an UnxpectedEof error.
  pub fn read(&mut self) -> io::Result<u8> {
    read_byte(self)
  }
}

impl<I: Iterator<Item=io::Result<u8>>> Iterator for ByteReader<I> {
  type Item = io::Result<u8>;

  fn next(&mut self) -> Option<Self::Item> {
    let val = self.reader.next();
    if val.is_some() {
      self.count += 1;
      // Return None if we've exceeded our limit.
      if let Some(l) = self.limit {
        if l > self.count {
          return None;
        }
      }
    }
    val
  }
}

impl<I: Iterator<Item=io::Result<u8>>> From<I> for ByteReader<I> {
  fn from(iter: I) -> Self {
    ByteReader::new(iter)
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
