//! Implementation of the serialization traits for Rust strings.

use std::io;

use ::{BerSerialize, BerDeserialize, Asn1Info};
use tag;
use err;

impl Asn1Info for String {
  fn asn1_tag() -> Option<tag::Tag> {
    Some(tag::Tag {
      class: tag::Class::Universal,
      tagnum: 19u8.into(),
      constructed: false,
    })
  }

  fn asn1_type() -> tag::Type {
    tag::Type::from("PRINTABLESTRING")
  }
}

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
