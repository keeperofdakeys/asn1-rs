use std::io;

use tag;
use err;
use serial;
use byte::{read_byte, write_byte};

impl serial::traits::Asn1Info for u64 {
  fn asn1_type() -> tag::Type {
    tag::Type::from("INTEGER")
  }

  fn asn1_class() -> tag::Class {
    tag::Class::Universal
  }

  fn asn1_tagnum() -> tag::TagNum {
    tag::TagNum::from(2u8)
  }

  fn asn1_constructed() -> bool {
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
    let len_num = try!(match len {
      tag::Len::Def(l) => Ok(l),
      _ => Err(err::DecodeError::PrimIndef),
    });

    let mut int: u64 = 0;

    for _ in 0..len_num {
      let byte = try!(read_byte(reader));
      int = (int << 8) + (byte as u64);
    }

    Ok(int)
  }
}
