use std::io;

use tag;
use err;

/// A trait that provides data about the ASN.1 tag and type for a Rust type.
pub trait Asn1Info {
  /// Get the ASN.1 tag for this Rust type.
  fn asn1_tag() -> tag::Tag;

  /// Get the ASN.1 type for this Rust type.
  fn asn1_type() -> tag::Type;
}

/// A trait that provides the plumbing for serializing ASN.1
/// data from a Rust type.
///
/// Usually you'll only need to implement serialize_imp yourself.
pub trait Asn1Serialize: Asn1Info {
  /// Serialise a value into ASN.1 data, with a tag (explicit tagging).
  fn serialize_exp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    let mut bytes: Vec<u8> = Vec::new();
    try!(self.serialize_imp(&mut bytes));

    let len = bytes.len() as tag::LenNum;
    let tag = tag::TagLen {
      tag: Self::asn1_tag(),
      len: Some(len).into(),
    };
    try!(tag.write_taglen(writer));

    try!(writer.write_all(&bytes));

    Ok(())
  }

  /// Serialise a value into ASN.1 data, without a tag (implicit tagging).
  ///
  /// (In order to write a tag yourself, you may need to know the byte-count written.
  /// This is most easily achieved by supplying a &mut Vec<u8> as the writer.)
  fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError>;
}

/// A trait that provides the plumbing for deserializing ASN.1
/// data into a Rust type.
///
/// Usually you'll only need to implement deserialize_imp yourself.
pub trait Asn1Deserialize: Asn1Info + Sized {

  /// Deserialise ASN.1 data with a tag into a value.
  ///
  /// This function will decode the tag to verify the tag for this type,
  /// and only read the amount of bytes declared in the tag.
  fn deserialize_exp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I) -> Result<Self, err::DecodeError> {
    let tag = try!(tag::TagLen::read_taglen(reader));

    // If element is primitive, and length is indefinite, we can't decode it.
    if tag.tag.constructed && tag.len == tag::Len::Indef {
      Err(err::DecodeError::PrimIndef)
    } else {
      Self::deserialize_imp(reader, tag.len)
    }
  }

  /// Deserialise ASN.1 data without a tag into a value.
  ///
  /// Since the data has no tag, the byte length must be passed to this function.
  fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError>;
}
