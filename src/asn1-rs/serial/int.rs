use tag;
use err;

use std::io;

impl Asn1Info for u64 {
  fn asn1_type() -> tag::Type {
    "INTEGER".into()
  }

  fn asn1_class() -> tag::Class {
    tag::Class::Universal
  }

  fn asn1_tagnum() -> tag::TagNum {
    0x02.into()
  }

  fn asn1_constructed() -> bool {
    false
  }
}

impl Asn1Serialize for u64 {
  fn serialize<W: io::Write>(&self, writer: W)
    -> Result<(), err::EncodeError> {
    
  }
}

impl Asn1Deserialize for OctetString {
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I) -> Result<Self, err::DecodeError> {
    unimplemented!();
  }
}
