//! A collection of primitive wrappers for ASN.1, where a direct Rust equivalent may not exist.
//!
//! Note that a Rust type can represent only a single ASN.1 type. So since Vec
//! represents a generic SEQUENCE OF container, OctetString must use a separate
//! type.

use std::io;

use ::{BerSerialize, BerDeserialize};
use tag;
use err;

/// A Rust equivalent to an ASN.1 OCTET STRING.
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
