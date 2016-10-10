//! The base traits that are used for serializing and deserializing rust types.
//!
//! BerSerialize implements serialization, and BerDeserialize implements deserialization.
//! Both traits depend upon the Asn1Info trait being implemented, to provide
//! metadata about the type (like ASN.1 tag and ASN.1 type).
//!
//! For both traits, only the (de)serialize_value functions should need overridden
//! to get custom behaviour. If you need to change how tags are handled, you may need
//! to override _serialize_enc or _deserialize_with_tag.
use std::io;

use ::Asn1Info;
use tag;
use err;
use ber::enc;

/// Provides the methods required to serialize this Rust type into an ASN.1 stream.
///
/// When implementing this for a simple primitive type, implementing `serialize_value`
/// should be all that's required. For more complex, structured types you may need to
/// implement `_serialize_enc` (this is called first by `serialize_enc`).
pub trait BerSerialize: Asn1Info {
  /// Serialize a value into ASN.1 data as DER.
  fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
    self.serialize_enc(enc::DER, writer)
  }

  /// Serialize a value into ASN.1 data using a specific set of encoding rules.
  fn serialize_enc<E: enc::BerEncRules, W: io::Write>
      (&self, e: E, writer: &mut W) -> Result<(), err::EncodeError> {
    debug!("Encoding the type {}", Self::asn1_type());
    if let Some(r) = self._serialize_enc(e, writer) {
      return r;
    }

    let tag = match Self::asn1_tag() {
      Some(tag) => tag,
      None => {
        debug!("Type does not have a tag");
        return self.serialize_value(e, writer);
      },
    };

    try!(tag.write_tag(writer));

    // If this is indefinite length and constructed, write the data directly.
    if E::len_rules() == enc::LenEnc::Indefinite &&
       tag.constructed {
      debug!("Using indefinite length");
      try!(tag::Len::Indef.write_len(writer));
      try!(self.serialize_value(e, writer));
      try!(tag::Len::write_indef_end(writer));
    // Otherwise write to a Vec first, so we can write the length.
    } else {
      debug!("Using definite length encoding");
      let mut bytes: Vec<u8> = Vec::new();
      try!(self.serialize_value(e, &mut bytes));
      try!(tag::Len::write_len(Some(bytes.len() as tag::LenNum).into(), writer));
      try!(writer.write_all(&bytes));
    }

    Ok(())
  }

  /// An empty method that is called first by `serialize_enc` to allow custom
  /// handling, without losing normal serialization behaviour.
  ///
  /// Return `Some(())` to indicate stop normal behaviour, or `None` to continue.
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
/// should be all that's required. For more complex, structued types you may need to
/// implement `_deserialize_with_tag` (this is called first by `deserialize_with_tag).
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
    debug!("Decoding the type {}", Self::asn1_type());
    if let Some(r) = Self::_deserialize_with_tag(e, reader, tag, len) {
      return r;
    }

    if Some(tag) != Self::asn1_tag() {
      if let Some(our_tag) = Self::asn1_tag() {
        warn!("Expected tag {}, but found tag {}", tag, our_tag);
      } else {
        warn!("Expected tag {}, but found no tag", tag);
      }
      return Err(err::DecodeError::TagTypeMismatch);
    }

    // Handle any indefinite length error conditions.
    if len == tag::Len::Indef {
      // Return an error if the encoding rules only allow definite length
      // encoding.
      if E::len_rules() == enc::LenEnc::Definite {
        warn!("Encountered indefinite length encoding, but encoding rules don't allow this");
        return Err(err::DecodeError::IndefiniteLen);
      // If this element is primitve, the length isn't allowed to be indefinite length.
      } else if !tag.constructed {
        warn!("Encountered indefinite length encoding, but this is a primitive element");
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

  /// An empty method that is called first by `deserialize_with_tag` to allow
  /// custom handling, without losing normal deserialization behaviour.
  ///
  /// Return `Some(..)` to return that value, or `None` to use normal behaviour.
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
    panic!("deserialize_value must be implemented for types that require it");
  }
}
