use std::io;

use tag;
use err;
use enc;
use serial;

impl serial::Asn1Info for Vec<u8> {
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

impl serial::Asn1Serialize for Vec<u8> {
  fn serialize_bytes<E: enc::Asn1EncRules, W: io::Write>
      (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write_all(self));
    Ok(())
  }
}

impl serial::Asn1Deserialize for Vec<u8> {
  fn deserialize_bytes<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
      (_: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.ok_or(err::DecodeError::PrimIndef));
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    Ok(try!(bytes))
  }
}

