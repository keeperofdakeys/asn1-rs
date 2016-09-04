use std::io;

use tag;
use err;
use enc;

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
/// Usually you'll only need to implement serialize_enc yourself.
pub trait Asn1Serialize: Asn1Info {
  /// Serialize a value into ASN.1 data as DER.
  fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    self.serialize_enc(enc::DER, writer)
  }

  /// Serialize a value into ASN.1 data using a specific set of encoding rules.
  fn serialize_enc<E: enc::Asn1EncRules, W: io::Write>
      (&self, e: E, writer: &mut W) -> Result<(), err::EncodeError> {
    if E::tag_rules() == enc::TagEnc::Implicit {
      try!(self.serialize_bytes(e, writer));
      return Ok(())
    }

    let mut bytes: Vec<u8> = Vec::new();
    try!(self.serialize_bytes(e, &mut bytes));

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
  fn serialize_bytes<E: enc::Asn1EncRules, W: io::Write>
    (&self, e: E, writer: &mut W) -> Result<(), err::EncodeError>;
}

/// A trait that provides the plumbing for deserializing ASN.1
/// data into a Rust type.
///
/// Usually you'll only need to implement deserialize_enc yourself.
pub trait Asn1Deserialize: Asn1Info + Sized {
  /// Deserialize ASN.1 data into a Rust value, accepting any valid BER.
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: &mut I) -> Result<Self, err::DecodeError> {
    Self::deserialize_enc(enc::BER, reader, None)
  }

  /// Deserialize ASN.1 data into a Rust value, using a specific set of encoding rules.
  fn deserialize_enc<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
      (e: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError> {
    if E::tag_rules() == enc::TagEnc::Implicit {
      return Self::deserialize_bytes(e, reader, len);
    }
    let tag = try!(tag::TagLen::read_taglen(reader));

    // If element is primitive, and length is indefinite, we can't decode it.
    if tag.tag.constructed && tag.len == tag::Len::Indef {
      Err(err::DecodeError::PrimIndef)
    } else {
      Self::deserialize_bytes(e, reader, tag.len.as_num())
    }
  }

  /// Deserialise ASN.1 data without a tag into a value.
  ///
  /// Since the data has no tag, the byte length must be passed to this function.
  fn deserialize_bytes<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
    (e: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError>;
}
