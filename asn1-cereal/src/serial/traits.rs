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
    let tag = Self::asn1_tag();
    try!(tag.write_tag(writer));

    // If this is indefinite length and constructed, write the data directly.
    if E::len_rules() == enc::LenEnc::Indefinite &&
       tag.constructed {
      try!(tag::Len::Indef.write_len(writer));
      try!(self.serialize_bytes(e, writer));
      try!(tag::Len::write_indef_end(writer));
    // Otherwise write to a Vec first, so we can write the length.
    } else {
      let mut bytes: Vec<u8> = Vec::new();
      try!(self.serialize_bytes(e, &mut bytes));
      try!(tag::Len::write_len(Some(bytes.len() as tag::LenNum).into(), writer));
      try!(writer.write_all(&bytes));
    }

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
    Self::deserialize_enc(enc::BER, reader)
  }

  /// Deserialize ASN.1 data into a Rust value, using a specific set of encoding rules.
  fn deserialize_enc<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
      (e: E, reader: &mut I) -> Result<Self, err::DecodeError> {
    let tag = try!(tag::Tag::read_tag(reader));
    Self::deserialize_enc_tag(e, reader, tag)
  }

  /// Deserialize ASN.1 data into a Rust value, using a specific set of encoding rules. Also
  /// use a specific tag, rather than reading from stream.
  fn deserialize_enc_tag<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
      (e: E, reader: &mut I, tag: tag::Tag) -> Result<Self, err::DecodeError> {
    if tag != Self::asn1_tag() {
      return Err(err::DecodeError::TagTypeMismatch);
    }

    // Read length fromm stream.
    let len = try!(tag::Len::read_len(reader));

    // Handle any indefinite length error conditions.
    if len == tag::Len::Indef {
      // Return an error if the encoding rules only allow definite length
      // encoding.
      if E::len_rules() == enc::LenEnc::Definite {
        return Err(err::DecodeError::IndefiniteLen);
      // If this element is primitve, the length isn't allowed to be indefinite length.
      } else if !tag.constructed {
       return Err(err::DecodeError::PrimIndef)
      }
    }
    // Read the main data.
    let item: Self = try!(Self::deserialize_bytes(e, reader, len.as_num()));

    // If this is encoded with an indefinte length, try to read the end octets.
    if len == tag::Len::Indef {
      try!(tag::Len::read_indef_end(reader));
    }

    Ok(item)
  }

  /// Deserialise ASN.1 data without a tag into a value.
  ///
  /// Since the data has no tag, the byte length must be passed to this function.
  fn deserialize_bytes<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
    (e: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError>;
}
