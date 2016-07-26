use std::io;

use tag;
use err;
use serial;

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
  fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    unimplemented!();
  }
}

impl serial::traits::Asn1Deserialize for u64 {
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I) -> Result<Self, err::DecodeError> {
    unimplemented!();
  }
}
