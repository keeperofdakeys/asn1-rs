//! A collection of primitive wrappers for ASN.1, where a direct Rust equivalent may not exist.

use std::io;

use ::{BerSerialize, BerDeserialize};
use tag;
use err;

/// A Rust wrapper for OCTET STRING.
///
/// To access the internal element, call `a.0`.
///
/// (Use this instead of Vec<u8>, since Vec is used for SEQUENCE OF).
pub struct OctetString(Vec<u8>);

asn1_info!(OctetString, [UNIVERSAL 4], "OCTET STRING");

impl BerSerialize for OctetString {
  fn serialize_value<E: ::BerEncRules, W: io::Write>
      (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write_all(&self.0));
    Ok(())
  }
}

impl BerDeserialize for OctetString {
  fn deserialize_value<E: ::BerEncRules, I: Iterator<Item=io::Result<u8>>>
      (_: E, reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.as_num().ok_or(err::DecodeError::PrimIndef));
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    Ok(OctetString(try!(bytes)))
  }
}
