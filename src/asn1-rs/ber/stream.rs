use tag;
use err;
use byte;

use std::cmp::Ordering;
use std::io;

pub trait StreamDecodee {
  /// This function is called when an ASN.1 tag is encountered. In other
  /// words, at the start of an ASN.1 element.
  fn start_element(&mut self, tag: tag::Tag) -> ParseResult {
    ParseResult::Ok
  }

  /// This function is called when an ASN.1 element has finished decoding.
  /// Specifically, this is called for both constructed, and un-constructed
  /// elements.
  fn end_element(&mut self, tag: tag::Tag) -> ParseResult {
    ParseResult::Ok
  }

  // FIXME: Currently it's the function's responsibility to decode the element
  // with the correct amounts of bytes. Without heap allocation, this might be
  // the only way.
  /// This is called when a primitive element is encountered. The start_element
  /// function is always called before this.
  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut byte::ByteReader<I>,
      len: tag::LenNum) -> ParseResult {
    for _ in 0..len {
      match reader.read() {
        Err(e) => return e.into(),
        _ => {},
      }
    }
    ParseResult::Ok
  }

  fn warning(err: err::DecodeError) -> ParseResult {
    ParseResult::Stop
  }

  /// This is called when a fatal decoding error occurs.
  fn error(err: err::DecodeError) {
  }
}

/// A decoder that calls into an object implementing the StreamDecodee
/// trait.
pub struct StreamDecoder<'a, I: Iterator<Item=io::Result<u8>>, S: StreamDecodee + 'a> {
  /// Internal reader with an included byte counter.
  reader: byte::ByteReader<I>,
  /// Object implementing StreamDecodee trait, called into during decoding.
  decodee: &'a mut S,
}

impl<'a, I: Iterator<Item=io::Result<u8>>, S: StreamDecodee> StreamDecoder<'a, I, S> {
  pub fn new<R: Into<byte::ByteReader<I>>>(reader: R, decodee: &'a mut S) -> Self {
    StreamDecoder {
      reader: reader.into(),
      decodee: decodee,
    }
  }

  /// Decode an asn1 element.
  pub fn decode(&mut self) -> Result<(), err::DecodeError> {
    self._decode().and(Ok(()))
  }

  // FIXME: Convert explicit decoded_len to use diff of internal reader count.
  /// Internal decode function.
  fn _decode(&mut self) -> Result<tag::Tag, err::DecodeError> {
    // Decode tag.
    let tag = try!(tag::Tag::decode_tag(&mut self.reader));
    let post_tag_count: tag::LenNum  = self.reader.count;

    // Call the decodee start element callback;
    self.decodee.start_element(tag);


    // If this type is constructed, decode child element..
    if tag.constructed {
      // Loop over child elements.
      loop {
        let decoded_len = self.reader.count - post_tag_count;
        // Compare decoded length with length in tag.
        // Put this first to handle zero-length elements.
        match tag.len.partial_cmp(&decoded_len) {
          // Return an error when we've decoded too much.
          Some(Ordering::Less) => return Err(err::DecodeError::GreaterLen),
          // Finish loop when equal, we must be finished.
          Some(Ordering::Equal) => break,
          // Continue when we are still decoding.
          Some(Ordering::Greater) => {},
          // Continue when using indefinite length encoding.
          None => {},
        };

        // Decode each child element.
        let child_tag = try!(self._decode());

        // If applicable, identify end of indefinite length encoding.
        // When decoding indefinite length encoding, stop on '00 00'
        // tag.
        if child_tag.len == tag::Len::Def(0) &&
           child_tag.class == tag::Class::Universal &&
           child_tag.tagnum == 0 {
          break;
        }
      }
    // Otherwise decode primitive value.
    } else {
      let len_num = try!(match tag.len {
        tag::Len::Def(l) => Ok(l),
        tag::Len::Indef =>
          Err(err::DecodeError::PrimIndef),
      });

      // Call decodee primitive decode callback.
      self.decodee.primitive(&mut self.reader, len_num);

      // Calculate decoded length.
      let decoded_len = self.reader.count - post_tag_count;
      // Ensure the exact amout of bytes was decoded.
      match tag.len.partial_cmp(&decoded_len) {
        Some(Ordering::Less) => return Err(err::DecodeError::GreaterLen),
        Some(Ordering::Greater) => return Err(err::DecodeError::SmallerLen),
        _ => {},
      }
    }

    // Call decodee end element callback.
    self.decodee.end_element(tag);

    // Return decoded + tag_len, which is total decoded length.
    Ok(tag)
  }
}

pub struct StreamEncoder<W: io::Write> {
  writer: byte::ByteWriter<W>
}

impl<W: io::Write> StreamEncoder<W> {
  pub fn new<T: Into<byte::ByteWriter<W>>>(writer: T) -> Self {
    StreamEncoder {
      writer: writer.into(),
    }
  }
}

impl<W: io::Write> StreamDecodee for StreamEncoder<W> {
  fn start_element(&mut self, tag: tag::Tag) -> ParseResult {
    match tag.encode_tag(&mut self.writer) {
      Err(e) =>  return e.into(),
      _ => (),
    };
    ParseResult::Ok
  }

  fn end_element(&mut self, tag: tag::Tag) -> ParseResult {
    ParseResult::Ok
  }

  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut byte::ByteReader<I>,
      len: tag::LenNum) -> ParseResult {
    for _ in 0..len {
      // Read a byte and write a byte.
      match reader.read() {
        Ok(byte) => match self.writer.write_byte(byte) {
          Err(e) => return e.into(),
          _ => (),
        },
        Err(e) => return e.into(),
      };
    }
    ParseResult::Ok
  }
}


// FIXME: This seems to have two mixed meanings, perhaps split it?
/// The result of a parsing function.
pub enum ParseResult {
  /// Everything went okay.
  Ok,
  /// Decoding should stop.
  Stop,
  /// Decoding should skip next element.
  Skip,
  /// An error occured decoding an element.
  DecodeError(err::DecodeError),
  /// An error occured encoding an element.
  EncodeError(err::EncodeError),
  /// An IO error occured.
  IO(io::Error),
}

impl From<err::DecodeError> for ParseResult {
  fn from(err: err::DecodeError) -> Self {
    ParseResult::DecodeError(err)
  }
}

impl From<err::EncodeError> for ParseResult {
  fn from(err: err::EncodeError) -> Self {
    ParseResult::EncodeError(err)
  }
}

impl From<io::Error> for ParseResult {
  fn from(err: io::Error) -> Self {
    ParseResult::IO(err)
  }
}
