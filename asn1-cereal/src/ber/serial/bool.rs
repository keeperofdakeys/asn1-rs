//! Implementation of the serialization traits for Rust booleans.

use std::io;

use ::{BerSerialize, BerDeserialize};
use tag;
use err;
use byte::{read_byte, write_byte};

asn1_info!(bool, [UNIVERSAL 1], "BOOLEAN");

impl BerSerialize for bool {
  fn serialize_value<E: ::BerEncRules, W: io::Write>
      (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(write_byte(writer, if *self { 0x01 } else { 0x00 }));
    Ok(())
  }
}

impl BerDeserialize for bool {
  fn deserialize_value<E: ::BerEncRules, I: Iterator<Item=io::Result<u8>>>
      (_: E, reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.as_num().ok_or(err::DecodeError::PrimIndef));

    if len_num > 0 {
      let byte = try!(read_byte(reader));
      Ok(byte != 0x00)
    } else {
      Ok(false)
    }
  }
}
