use std::io;

use tag;
use err;
use serial;

impl serial::traits::Asn1Info for String {
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

impl serial::traits::Asn1Serialize for String {
  fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write_all(self.as_bytes()));
    Ok(())
  }
}

impl serial::traits::Asn1Deserialize for String {
  fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let len_num = try!(len.as_num().ok_or(err::DecodeError::PrimIndef));
    let bytes: Result<Vec<u8>, _> = reader.take(len_num as usize).collect();
    match String::from_utf8(try!(bytes)) {
      Ok(str) => Ok(str),
      Err(_) => Err(err::DecodeError::Custom("Error decoding PrintableString as UTF8")),
    }
  }
}
