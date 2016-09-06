use std::io;

use tag;
use err;
use enc;
use serial;

impl serial::Asn1Info for String {
  fn asn1_tag() -> tag::Tag {
    tag::Tag {
      class: tag::Class::Universal,
      tagnum: 19u8.into(),
      constructed: false,
    }
  }

  fn asn1_type() -> tag::Type {
    tag::Type::from("PRINTABLESTRING")
  }
}

impl serial::Asn1Serialize for String {
  fn serialize_bytes<E: enc::Asn1EncRules, W: io::Write>
      (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write_all(self.as_bytes()));
    Ok(())
  }
}

impl serial::Asn1Deserialize for String {
  fn deserialize_bytes<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
      (_: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.ok_or(err::DecodeError::PrimIndef));
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    match String::from_utf8(try!(bytes)) {
      Ok(str) => Ok(str),
      Err(_) => Err(err::DecodeError::Custom("Error decoding PrintableString as UTF8")),
    }
  }
}
