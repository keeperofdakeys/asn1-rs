use asn1;

use std::cmp::Ordering;
use std::io;

trait StreamDecodee {
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
  fn primitive<R: io::Read>(&mut self, reader: asn1::ByteReader<R>, len: asn1::LenNum) -> ParseResult {
    for x in 0..len {
      try!(reader.read());
    }
    ParseResult::Ok
  }
}

/// A decoder that calls into an object implementing the StreamDecodee
/// trait.
struct StreamDecoder<'a, R: io::Read + 'a, S: StreamDecodee> {
  /// Internal reader with an included byte counter.
  reader: asn1::ByteReader<'a, R>,
  /// Object implementing StreamDecodee trait, called into during decoding.
  decodee: S,
}

impl<'a, R: io::Read, S: StreamDecodee> StreamDecoder<'a, R, S> {
  fn new<T: Into<asn1::ByteReader<'a, R>>>(reader: T, decodee: S) -> Self {
    StreamDecoder {
      reader: reader.into(),
      decodee: decodee,
    }
  }

  /// Decode an asn1 element.
  pub fn decode(&self) {
    let _ = self._decode();
  }

  // FIXME: Convert explicit decoded_len to use diff of internal reader count.
  /// Internal decode function.
  fn _decode(&self) -> Result<asn1::Tag, asn1::LenNum> {
    // Get tag and decoded tag length.
    let (tag, tag_len) = try!(asn1::Tag::decode_tag(self.reader));

    // Call the decodee start element callback;
    self.decodee.start_element(tag);

    // Don't decode zero length elements.
    if tag.len == asn1::Len::Def(0) {
      return Ok((tag, tag_len));
    }

    // Decoded length of this element.
    let mut decoded_len: asn1::LenNum = 0;

    // If this type is constructed, decode child element..
    if tag.constructed {
      // Loop over child elements.
      loop {
        // Decode each child element, add to decoded length.
        let (child_tag, child_len) = try!(self._decode());
        decoded_len += child_len;

        // If applicable, identify end of indefinite length encoding.
        // When decoding indefinite length encoding, stop on '00 00'
        // tag.
        if child_tag.len == asn1::Len::Def(0) &&
           child_tag.class == asn1::Class::Universal &&
           child_tag.tagnum == 0 {
          break;
        }

        // Compare decoded length with length in tag.
        match tag.len.partial_cmp(&decoded_len) {
          // Return an error when we've decoded too much.
          Some(Ordering::Less) => return Err(asn1::DecodeError::GreaterLen),
          // Finish loop when equal, we must be finished.
          Some(Ordering::Equal) => break,
          // Continue when less, we're still decoding.
          Some(Ordering::Less) => {},
          // Continue when using indefinite length encoding.
          None => {},
        };
      }
    // Otherwise decode primitive value.
    } else {
      let len_num = tag.len.into().or_ok(asn1::DecodeError::PrimIndef);

      // Call decodee primitive decode callback.
      self.decodee.primitive(self.reader, len_num);

      // Since we're decoding an element, we use add tag length.
      decoded_len += len_num;
    }

    // Call decodee end element callback.
    self.decodee.end_element();

    // Return decoded + tag_len, which is total decoded length.
    Ok((tag, decoded_len + tag_len))
  }
}

/// The result of a parsing function.
enum ParseResult {
  /// Everything went okay.
  Ok,
  /// An error occured decoding an element.
  DecodeError(asn1::DecodeError),
  /// An IO error occured.
  IO(io::Error),
  /// Early EOF reached.
  EOF,
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
