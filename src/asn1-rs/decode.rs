use super::asn1::{Asn1Tag, Asn1Data, Asn1Slice, Asn1Error};
// FIXME: Do I actually return an Asn1Type?
// FIXME: How do we handle indefinite length encoding, no subslice.
/// Given a slice, decode tag and return tag and subslice of data.
fn decode_tag(data: Asn1Slice) -> Result<(Asn1Tag, Asn1Slice), Asn1Error> {
  Err(Asn1Error::InvalidTag(0))
}

/// An iterator over asn1 types.
struct Asn1Iter {
  data: &[u8],
  tag: Asn1Tag
}

impl Asn1Iter {
  /// Create a new iterator over an Asn1Slice.
  fn new(data: Asn1Slice) -> Asn1Iter {
    let (tag, slice) = decode_tag(data);
    Asn1Iter {
      data: data,
      tag: tag
    }
  }
}

impl Iterator for Asn1Iter {
  fn next
}
