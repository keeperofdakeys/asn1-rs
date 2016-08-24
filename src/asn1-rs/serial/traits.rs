use std::io;

use tag;
use err;

/// A trait that provides ASN.1 type information for a Rust type.
pub trait Asn1Info {
  /// Get the ASN.1 type for this Rust type.
  fn asn1_type(&self) -> tag::Type;

  /// Get the ASN.1 class for this Rust type.
  fn asn1_class(&self) -> tag::Class;

  /// Get the ASN.1 tag number for this Rust type.
  fn asn1_tagnum(&self) -> tag::TagNum;

  /// Get the ASN.1 constructed bit for this Rust type.
  fn asn1_constructed(&self) -> bool;

  /// Given a length, get the ASN.1 tag for this Rust type.
  fn asn1_tag(&self, len: tag::Len) -> tag::Tag {
    tag::Tag {
      class: self.asn1_class(),
      tagnum: self.asn1_tagnum(),
      len: len,
      constructed: self.asn1_constructed(),
    }
  }
}

/// A trait that provides the plumbing for serializing ASN.1
/// data from a Rust type.
pub trait Asn1Serialize: Asn1Info {
  /// Serialise an ASN.1 from a value.
  fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    let mut bytes: Vec<u8> = Vec::new();
    try!(self.serialize_bytes(&mut bytes));

    let len = bytes.len() as tag::LenNum;
    let tag = self.asn1_tag(tag::Len::from(Some(len)));
    try!(tag.encode_tag(writer));

    try!(writer.write(&bytes));

    Ok(())
  }

  /// Serialise ASN.1 bytes from a value.
  fn serialize_bytes<W: io::Write>(&self, writer: &mut W)
    -> Result<(), err::EncodeError>;
}

/// A trait that provides the plumbing for deserializing ASN.1
/// data into a Rust type.
pub trait Asn1Deserialize: Asn1Info + Sized {

  /// Deserialise ASN.1 data into a value.
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I)
    -> Result<Self, err::DecodeError>;
}
