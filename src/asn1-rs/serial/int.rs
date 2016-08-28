use std::io;

use tag;
use err;
use serial;
use byte::write_byte;

impl serial::traits::Asn1Info for u64 {
  fn asn1_type(&self) -> tag::Type {
    tag::Type::from("INTEGER")
  }

  fn asn1_class(&self) -> tag::Class {
    tag::Class::Universal
  }

  fn asn1_tagnum(&self) -> tag::TagNum {
    tag::TagNum::from(2u8)
  }

  fn asn1_constructed(&self) -> bool {
    false
  }
}

impl serial::traits::Asn1Serialize for u64 {
  fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    let mut started = false;
    // Loop through bytes in int backwards, start writing when first non-zero byte is encounted.
    for offset in (0..8).rev() {
      let shifted: u64 = self >> (offset * 8);
      let byte: u8 = (shifted & 0xff) as u8;
      if !started {
        if byte == 0 {
          continue;
        }
        started = true;
      }
      try!(write_byte(writer, byte));
    }

    // If we haven't written anything, write a zero byte.
    if !started {
      try!(write_byte(writer, 0 as u8));
    }
    Ok(())
  }
}

impl serial::traits::Asn1Deserialize for u64 {
  fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    unimplemented!();
  }
}
