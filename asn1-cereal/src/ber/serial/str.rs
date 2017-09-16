//! Implementation of the serialization traits for String.
//!
//! This currently uses OCTET STRING, as ASN.1 was invented before UTF8,
//! and OCTET STRING is the only thing that can actually hold this.
//!
//! If you require specific types of strings, please use the assoicated
//! rust type in the prim module.

use std::io;

use ::{BerSerialize, BerDeserialize};
use tag;
use err;

asn1_info!(String, [PRIM UNIVERSAL 19], "OCTET STRING");

impl BerSerialize for String {
  fn serialize_value<E: ::BerEncRules, W: io::Write>
      (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write_all(self.as_bytes()));
    Ok(())
  }
}

impl BerDeserialize for String {
  fn deserialize_value<E: ::BerEncRules, I: Iterator<Item=io::Result<u8>>>
      (_: E, reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.as_num().ok_or(err::DecodeError::PrimIndef));
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    match String::from_utf8(try!(bytes)) {
      Ok(str) => Ok(str),
      Err(_) => Err(err::DecodeError::Custom("Error decoding PrintableString as UTF8")),
    }
  }
}
