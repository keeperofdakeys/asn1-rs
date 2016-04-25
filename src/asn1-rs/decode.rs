use asn1;

use std::cmp::Ordering;
use std::io;

pub trait StreamDecodee {
  /// This function is called when an ASN.1 tag is encountered. In other
  /// words, at the start of an ASN.1 element.
  fn start_element(&mut self, tag: asn1::Tag) -> ParseResult {
    ParseResult::Ok
  }

  /// This function is called when an ASN.1 element has finished decoding.
  /// Specifically, this is called for both constructed, and un-constructed
  /// elements.
  fn end_element(&mut self) -> ParseResult {
    ParseResult::Ok
  }

  // FIXME: Currently it's the function's responsibility to decode the element
  // with the correct amounts of bytes. Without heap allocation, this might be
  // the only way.
  /// This is called when a primitve element is encountered. The start_element
  /// function is always called before this.
  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut asn1::ByteReader<I>,
      len: asn1::LenNum) -> ParseResult {
    for _ in 0..len {
      match reader.read() {
        Err(e) => return e.into(),
        _ => {},
      }
    }
    ParseResult::Ok
  }

  /// This is called when a decoding error occurs.
  fn error(err: asn1::DecodeError) {
  }
}

/// A decoder that calls into an object implementing the StreamDecodee
/// trait.
pub struct StreamDecoder<I: Iterator<Item=io::Result<u8>>, S: StreamDecodee> {
  /// Internal reader with an included byte counter.
  reader: asn1::ByteReader<I>,
  /// Object implementing StreamDecodee trait, called into during decoding.
  decodee: S,
}

impl<I: Iterator<Item=io::Result<u8>>, S: StreamDecodee> StreamDecoder<I, S> {
  pub fn new<T: Into<asn1::ByteReader<I>>>(reader: T, decodee: S) -> Self {
    StreamDecoder {
      reader: reader.into(),
      decodee: decodee,
    }
  }

  /// Decode an asn1 element.
  pub fn decode(&mut self) -> Result<(), asn1::DecodeError> {
    self._decode().and(Ok(()))
  }

  // FIXME: Convert explicit decoded_len to use diff of internal reader count.
  /// Internal decode function.
  fn _decode(&mut self) -> Result<asn1::Tag, asn1::DecodeError> {
    let pre_tag_count: asn1::LenNum = self.reader.count;

    // Decode tag.
    let tag = try!(asn1::Tag::decode_tag(&mut self.reader));
    let post_tag_count: asn1::LenNum  = self.reader.count;

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
          Some(Ordering::Less) => return Err(asn1::DecodeError::GreaterLen),
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
        if child_tag.len == asn1::Len::Def(0) &&
           child_tag.class == asn1::Class::Universal &&
           child_tag.tagnum == 0 {
          break;
        }
      }
    // Otherwise decode primitive value.
    } else {
      let len_num = try!(match tag.len {
        asn1::Len::Def(l) => Ok(l),
        asn1::Len::Indef =>
          Err(asn1::DecodeError::PrimIndef),
      });

      // Call decodee primitive decode callback.
      self.decodee.primitive(&mut self.reader, len_num);
    }

    let post_decode_count = self.reader.count;

    // FIXME: If decoded length is larger than tag length, error here.

    // Call decodee end element callback.
    self.decodee.end_element();

    // Return decoded + tag_len, which is total decoded length.
    Ok(tag)
  }
}

/// The result of a parsing function.
pub enum ParseResult {
  /// Everything went okay.
  Ok,
  /// An error occured decoding an element.
  DecodeError(asn1::DecodeError),
  /// An IO error occured.
  IO(io::Error),
}

impl From<asn1::DecodeError> for ParseResult {
  fn from(err: asn1::DecodeError) -> Self {
    ParseResult::DecodeError(err)
  }
}

impl From<io::Error> for ParseResult {
  fn from(err: io::Error) -> Self {
    ParseResult::IO(err)
  }
}
