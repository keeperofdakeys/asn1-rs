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
