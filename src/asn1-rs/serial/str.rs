use std::io;

use tag;
use err;
use serial;

impl serial::traits::Asn1Info for String {
  fn asn1_type() -> tag::Type {
    tag::Type::from("INTEGER")
  }

  fn asn1_class() -> tag::Class {
    tag::Class::Universal
  }

  fn asn1_tagnum() -> tag::TagNum {
    tag::TagNum::from(19u8)
  }

  fn asn1_constructed() -> bool {
    false
  }
}

impl serial::traits::Asn1Serialize for String {
  fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write(self.as_bytes()));
    Ok(())
  }
}

impl serial::traits::Asn1Deserialize for String {
  fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let len_num = try!(match len {
      tag::Len::Def(l) => Ok(l),
      _ => Err(err::DecodeError::PrimIndef),
    });
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    match String::from_utf8(try!(bytes)) {
      Ok(str) => Ok(str),
      Err(_) => Err(err::DecodeError::Custom("Error decoding PrintableString as UTF8")),
    }
  }
}
