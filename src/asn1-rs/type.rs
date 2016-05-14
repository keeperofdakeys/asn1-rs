use tag;
use err;

use std::io;

struct OctetString(Vec<u8>);

impl Asn1Serialize for OctetString {
  fn serialize<W: io::Write>(&self, writer: W) -> Result<(), err::EncodeError> {
    try!(writer.write(&self.0));
  }
}

impl Asn1Deserialize for OctetString {
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I) -> Result<Self, err::DecodeError> {
    unimplemented!();
  }
}

impl Asn1Info for OctetString {
  fn asn1_type() -> tag::Type {
    "OCCTET STRING".into()
  }
  
  fn asn1_class() -> tag::Class {
    tag::Class::Universal
  }

  fn asn1_tagnum() -> tag::TagNum {
    4.into()
  }

  fn asn1_constructed() -> bool {
    false
  }
}
