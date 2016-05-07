use asn1;

use std::io;

/// A trait that provides the plumbing for serializing ASN.1
/// data from a Rust type.
trait Asn1Serialize: Asn1Info {
  /// Serialise ASN.1 data from a value.
  fn serialize<W: io::Write>(&self, writer: W)
    -> Result<(), asn1::EncodeError>;
}

/// A trait that provides the plumbing for deserializing ASN.1
/// data into a Rust type.
trait Asn1Deserialize: Asn1Info + Sized {

  /// Deserialise ASN.1 data into a value.
  fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I)
    -> Result<Self, asn1::DecodeError>;
}

/// A trait that provides ASN.1 type information for a Rust type.
trait Asn1Info {
  /// Get the ASN.1 type for this Rust type.
  fn asn1_type() -> asn1::Type;

  /// Get the ASN.1 class for this Rust type.
  fn asn1_class() -> asn1::Class;

  /// Get the ASN.1 tag number for this Rust type.
  fn asn1_tagnum() -> asn1::TagNum;

  /// Get the ASN.1 constructed bit for this Rust type.
  fn asn1_constructed() -> bool;
}
