use std::io;

use ::Asn1Info;
use tag;
use err;
use ber::enc;

/// Provides the methods required to serialize this Rust type into an ASN.1 stream.
///
/// When implementing this for a simple primitive type, implementing `serialize_value`
/// should be all that's required.
pub trait BerSerialize: Asn1Info {
  /// Serialize a value into ASN.1 data as DER.
  fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    self.serialize_enc(enc::DER, writer)
  }

  /// Serialize a value into ASN.1 data using a specific set of encoding rules.
  fn serialize_enc<E: enc::BerEncRules, W: io::Write>
      (&self, e: E, writer: &mut W) -> Result<(), err::EncodeError> {
    if let Some(r) = self._serialize_enc(e, writer) {
      return r;
    }

    let tag = match Self::asn1_tag() {
      Some(tag) => tag,
      None => return self.serialize_value(e, writer),
    };

    try!(tag.write_tag(writer));

    // If this is indefinite length and constructed, write the data directly.
    if E::len_rules() == enc::LenEnc::Indefinite &&
       tag.constructed {
      try!(tag::Len::Indef.write_len(writer));
      try!(self.serialize_value(e, writer));
      try!(tag::Len::write_indef_end(writer));
    // Otherwise write to a Vec first, so we can write the length.
    } else {
      let mut bytes: Vec<u8> = Vec::new();
      try!(self.serialize_value(e, &mut bytes));
      try!(tag::Len::write_len(Some(bytes.len() as tag::LenNum).into(), writer));
      try!(writer.write_all(&bytes));
    }

    Ok(())
  }

  fn _serialize_enc<E: enc::BerEncRules, W: io::Write>
      (&self, e: E, writer: &mut W) -> Option<Result<(), err::EncodeError>> {
    let _ = (e, writer);
    None
  }

  /// Serialise a value into ASN.1 data, without a tag (implicit tagging).
  fn serialize_value<E: enc::BerEncRules, W: io::Write>
    (&self, e: E, writer: &mut W) -> Result<(), err::EncodeError>;
}

/// Provides the methods required to deserialize this Rust type from an ASN.1 stream.
///
/// When implementing this for a simple primitive type, implementing `deserialize_value`
/// should be all that's required.
pub trait BerDeserialize: Asn1Info + Sized {
  /// Deserialize ASN.1 data into a Rust value, accepting any valid BER.
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: &mut I) -> Result<Self, err::DecodeError> {
    Self::deserialize_enc(enc::BER, reader)
  }

  /// Deserialize ASN.1 data into a Rust value, using a specific set of encoding rules.
  fn deserialize_enc<E: enc::BerEncRules, I: Iterator<Item=io::Result<u8>>>
      (e: E, reader: &mut I) -> Result<Self, err::DecodeError> {
    let (tag, len) = try!(tag::read_taglen(reader));
    Self::deserialize_with_tag(e, reader, tag, len)
  }


  /// Deserialize ASN.1 data into a Rust value, using a specific set of encoding rules, and
  /// also providing the decoded tag and length.
  ///
  /// This function assumes the next bytes to decode are
  /// the BER length of this element.
  fn deserialize_with_tag<E: enc::BerEncRules, I: Iterator<Item=io::Result<u8>>>
      (e: E, reader: &mut I, tag: tag::Tag, len: tag::Len) -> Result<Self, err::DecodeError> {
    if let Some(r) = Self::_deserialize_with_tag(e, reader, tag, len) {
      return r;
    }

    if let None = Self::asn1_tag() {
      panic!("Trying to decode item with no defined tag.");
    };

    if Some(tag) != Self::asn1_tag() {
      return Err(err::DecodeError::TagTypeMismatch);
    }

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
    let item: Self = try!(Self::deserialize_value(e, reader, len));

    // If this is encoded with an indefinte length, try to read the end octets.
    if len == tag::Len::Indef {
      try!(tag::Len::read_indef_end(reader));
    }

    Ok(item)
  }

  fn _deserialize_with_tag<E: enc::BerEncRules, I: Iterator<Item=io::Result<u8>>>
      (e: E, reader: &mut I, tag: tag::Tag, len: tag::Len) -> Option<Result<Self, err::DecodeError>> {
    let _ = (e, reader, tag, len);
    None
  }

  /// Deserialize an ASN.1 value from a BER stream, after having the tag and length
  /// decoded.
  ///
  /// The data length must be explicitly passed to this function. For primitive types,
  /// an error will be returned if this length is Indefinite.
  fn deserialize_value<E: enc::BerEncRules, I: Iterator<Item=io::Result<u8>>>
    (e: E, reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
    let (_, _, _) = (e, reader, len);
    unimplemented!();
  }
}
