use std::io;

use tag;
use err;
use serial;

impl serial::traits::Asn1Info for Vec<u8> {
  fn asn1_tag() -> tag::Tag {
    tag::Tag {
      class: tag::Class::Universal,
      tagnum: 4u8.into(),
      constructed: false,
    }
  }

  fn asn1_type() -> tag::Type {
    tag::Type::from("OCTET STRING")
  }
}

impl serial::traits::Asn1Serialize for Vec<u8> {
  fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write_all(self));
    Ok(())
  }
}

impl serial::traits::Asn1Deserialize for Vec<u8> {
  fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.as_num().ok_or(err::DecodeError::PrimIndef));
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    Ok(try!(bytes))
  }
}

