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
  pub fn new<T: Into<byte::ByteReader<I>>>(reader: T, decodee: &'a mut S) -> Self {
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

/*
/// Ber decoder that allows a client to control the decoding process.  
/// Clients are given 
struct TreeParser;

/// An ASN.1 element located in a tree of ASN.1 elements.
pub struct Asn1Node<I: Iterator<Item=io::Result<u8>>> {
  /// ASN.1 tag for this element.
  pub tag: tag::Tag,
  reader: byte::ByteReader<I>,
  // State that reader's byte counter should be at
  // for the next operation.
  offset: u64,
}

impl<I: Iterator<Item=io::Result<u8>>> Asn1Node<I> {
  /// Try retreiving the next child of a constructed ASN.1 element.
  ///
  /// Given a constructed ASN.1 element node, this function
  /// will return an ASN.1 node for the next child of this
  /// element. Once the last child is reached, None is returned.
  // FIXME: This has real problems with not decoding children,
  // and how to handle indefinite length encoding.
  fn next_child(&mut self) -> Option<Self> {
    if !self.tag.constructed {
      panic!("Can't call next_child on a primitive element.");
    }
    // panic if reader is beyond offset.
    unimplemented!()
  }

  /// For a primitive element, return a byte iterator that
  /// can be used to decode the ASN.1 value. THe returned
  /// iterator will reach Eof (return None), if more than
  /// the length of the element is read (as declared in
  /// the tag).
  ///
  /// The returned byte::ByteReader also provides a .read() function
  /// that will return an appropriate DecodeError when Eof is
  /// prematurely reached.
  fn decode<I2: Iterator<Item=io::Result<u8>>>(&mut self) -> byte::ByteReader<I2> {
    if self.tag.constructed {
      panic!("Can't call decode on a non-primitive element.");
    }
    let len = Option::<tag::LenNum>::from(self.tag.len).unwrap();

    byte::ByteReader::new_limit(self.reader, len)
  }
}
*/

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
  /// An IO error occured.
  IO(io::Error),
}

impl From<err::DecodeError> for ParseResult {
  fn from(err: err::DecodeError) -> Self {
    ParseResult::DecodeError(err)
  }
}

impl From<io::Error> for ParseResult {
  fn from(err: io::Error) -> Self {
    ParseResult::IO(err)
  }
}
