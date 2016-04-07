use super::asn1::{Asn1Tag, Asn1Data, Asn1Slice, Asn1Error};
// FIXME: Do I actually return an Asn1Type?
// FIXME: How do we handle indefinite length encoding, no subslice.
/// Given a slice, decode tag and return tag, next slice, and inner slice of data.
fn decode_tag<'a>(data: Asn1Slice<'a>) -> Result<(Asn1Tag, Asn1Slice<'a>, Asn1Slice<'a>), Asn1Error> {
  Err(Asn1Error::InvalidTag(0))
}

/// An iterator over asn1 types.
struct<'a> Asn1Iter {
  data: &'a [u8],
  tag: Asn1Tag
}

impl<'a> Asn1Iter<'a> {
  // FIXME: If this is indef length, we'll might need to update upper slice?
  /// Create a new iterator over an Asn1Slice.
  fn new(data: Asn1Slice<'a>) -> Asn1Iter<'a> {
    let (tag, len, slice) = decode_tag(data).unwrap();
    Asn1Iter {
      data: data,
      tag: tag,
      indef: len == 0
    }
  }
}

impl<'a> Iterator for Asn1Iter<'a> {
  type Item = (Asn1Tag, AsnSlice<'a>);

  fn next(&mut self) -> Option<Self::Item> {
    // FIXME: Once again, fix this for indefinite length encoding
    if (self.data.len == 0) {
      return None;
    }
    let (tag, next, inner) = decode_tag(self.data).unwrap();
    self.data = next;
    Some(tag, inner)
  }
}
