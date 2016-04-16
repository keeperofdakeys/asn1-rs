use asn1;

trait StreamDecodee {
  /// This function is called when an ASN.1 tag is encountered. In other
  /// words, at the start of an ASN.1 element.
  fn start_element(tag: asn::Tag) -> ParseResult {
    ParseResult::Ok
  }

  /// This function is called when an ASN.1 element has finished decoding.
  /// Specifically, this is called for both constructed, and un-constructed
  /// elements.
  fn end_element() -> ParseResult {
    ParseResult::Ok
  }

  // FIXME: Currently it's the function's responsibility to decode the element
  // with the correct amounts of bytes. Without heap allocation, this might be
  // the only way.
  /// This is called when a primitve element is encountered. The start_element
  /// function is always called before this.
  fn primitive(reader: asn1::ByteReader, len: asn1::LenNum) -> ParseResult {
    for x in 0..len {
      try!(reader.read());
    }
    ParseResult::Ok
  }
}

/// A decoder that calls into an object implementing the StreamDecodee
/// trait.
struct StreamDecoder<S: StreamDecodee> {
  reader: ByteReader,
  decodee: S,
}

impl<S: StreamDecodee> StreamDecoder<S> {
  fn new<R: Into<asn1::ByteReader>>(reader: R, decodee: S) -> Self {
    StreamDecoder {
      reader: reader::into(),
      decodee: decodee,
    }
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
}

impl From<asn1::DecodeError> for ParseResult {
  fn from(err: asn1::DecodeError) -> Self {
    ParseResult::DecodeError(err)
  }
}

impl From<asn1::IO> for ParseResult {
  fn from(err: asn1::IO) -> Self {
    ParseResult::IO(err)
  }
}
