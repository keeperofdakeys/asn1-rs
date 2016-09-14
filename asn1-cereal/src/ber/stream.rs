//! A `SAXParser` inspired stream parser and encoder for ber streams.
//!
//! Given a struct that implements `StreamDecodee`, a `StreamDecoder` can be used to
//! decode the stream, calling each function in the `StreamDecodee` trait as they
//! are encountered.
//!
//! The `StreamEncoder` struct is also provided, which itself implements the
//! `StreamDecodee` trait. This allows you to encode an ASN.1 stream using
//! the `StreamDecodee` interface as the caller.

use tag;
use err;
use byte;

use std::cmp::Ordering;
use std::io;

/// This trait provides a `SAXParser` inspired interface for parsing ASN.1 streams.
///
/// If it were implemented, the `ParseResult` return types would be used to
/// provide feedback on whether parsing of that element was successful.
pub trait StreamDecodee {
  /// This function is called when an ASN.1 tag is encountered. In other
  /// words, at the start of an ASN.1 element.
  fn start_element(&mut self, _: tag::Tag, _: tag::Len) -> ParseResult {
    ParseResult::Ok
  }

  /// This function is called when an ASN.1 element has finished decoding.
  /// Note that this is also called for all elements, even after a primitive()
  /// call. For this reason, you may need to check the constructed flag in some
  /// cases.
  fn end_element(&mut self, _: tag::Tag, _: tag::Len) -> ParseResult {
    ParseResult::Ok
  }

  // FIXME: Currently it's the function's responsibility to decode the element
  // with the correct amounts of bytes. Without heap allocation, this might be
  // the only way.
  /// This function is called when a primitive element is encountered. Note that both
  /// start_element and end_element are called before/after this function.
  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut byte::ByteReader<I>,
      len: tag::LenNum) -> ParseResult {
    for _ in 0..len {
      if let Err(e) = reader.read() {
        return e.into();
      }
    }
    ParseResult::Ok
  }

  /// This function would be called when a recoverable decode error occurs, however
  /// currently nothing calls this.
  fn warning(_: err::DecodeError) -> ParseResult {
    ParseResult::Stop
  }

  /// This function would be called when a fatal decoding error occurs, however 
  /// currently nothing calls this.
  fn error(_: err::DecodeError) {
  }
}

/// A decoder that calls into a struct implementing the `StreamDecodee` trait,
/// similar to a `SAXParser`.
pub struct StreamDecoder<'a, I: Iterator<Item=io::Result<u8>>, S: StreamDecodee + 'a> {
  /// Internal reader with an included byte counter.
  reader: byte::ByteReader<I>,
  /// Object implementing StreamDecodee trait, functions are called when
  /// specific things are found in the ASN.1 stream.
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
  fn _decode(&mut self) -> Result<(tag::Tag, tag::Len), err::DecodeError> {
    // Decode tag.
    let (tag, len) = try!(tag::read_taglen(&mut self.reader));
    let post_tag_count: tag::LenNum  = self.reader.count;

    // Call the decodee start element callback;
    self.decodee.start_element(tag, len);


    // If this type is constructed, decode child element..
    if tag.constructed {
      // Loop over child elements.
      loop {
        let decoded_len = self.reader.count - post_tag_count;
        // Compare decoded length with length in tag.
        // Put this first to handle zero-length elements.
        match len.partial_cmp(&decoded_len) {
          // Return an error when we've decoded too much.
          Some(Ordering::Less) => return Err(err::DecodeError::GreaterLen),
          // Finish loop when equal, we must be finished.
          Some(Ordering::Equal) => break,
          // Continue when we are still decoding, or using indefinite
          // length encoding.
          Some(Ordering::Greater) | None => {},
        };

        // Decode each child element.
        let (child_tag, child_len) = try!(self._decode());

        // If applicable, identify end of indefinite length encoding.
        // When decoding indefinite length encoding, stop on '00 00'
        // tag.
        if child_len == tag::Len::Def(0) &&
           child_tag.class == tag::Class::Universal &&
           child_tag.tagnum == 0 {
          break;
        }
      }
    // Otherwise decode primitive value.
    } else {
      let len_num = try!(match len {
        tag::Len::Def(l) => Ok(l),
        tag::Len::Indef =>
          Err(err::DecodeError::PrimIndef),
      });

      // Call decodee primitive decode callback.
      self.decodee.primitive(&mut self.reader, len_num);

      // Calculate decoded length.
      let decoded_len = self.reader.count - post_tag_count;
      // Ensure the exact amout of bytes was decoded.
      match len.partial_cmp(&decoded_len) {
        Some(Ordering::Less) => return Err(err::DecodeError::GreaterLen),
        Some(Ordering::Greater) => return Err(err::DecodeError::SmallerLen),
        _ => {},
      }
    }

    // Call decodee end element callback.
    self.decodee.end_element(tag, len);

    // Return decoded + tag_len, which is total decoded length.
    Ok((tag, len))
  }
}

/// A stream encoder that implements `StreamDecodee`. Using this,
/// a ASN.1 stream can be written using a `SAXParser` style interface.
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
  fn start_element(&mut self, tag: tag::Tag, len: tag::Len) -> ParseResult {
    if let Err(e) = tag::write_taglen(tag, len, &mut self.writer) {
      e.into()
    } else {
      ParseResult::Ok
    }
  }

  fn end_element(&mut self, _: tag::Tag, _: tag::Len) -> ParseResult {
    ParseResult::Ok
  }

  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut byte::ByteReader<I>,
      len: tag::LenNum) -> ParseResult {
    for _ in 0..len {
      // Read a byte and write a byte.
      match reader.read() {
        Ok(byte) => if let Err(e) = self.writer.write_byte(byte) {
          return e.into()
        },
        Err(e) => return e.into(),
      };
    }
    ParseResult::Ok
  }
}


// FIXME: This seems to have two mixed meanings, perhaps split it?
/// The result of parsing after a callback on a `StreamDecodee`.
///
/// If it were implemented, this would provide feedback to the Decoder/Parser.
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
