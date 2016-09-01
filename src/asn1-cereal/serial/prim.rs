use std::io;

use tag;
use err;
use serial;

pub struct OctetString(Vec<u8>);

impl serial::traits::Asn1Info for OctetString {
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

impl serial::traits::Asn1Serialize for OctetString {
  fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    try!(writer.write(&self.0));
    unimplemented!();
  }
}

impl serial::traits::Asn1Deserialize for OctetString {
  fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    unimplemented!();
  }
}

