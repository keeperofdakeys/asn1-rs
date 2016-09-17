//! Encoding rules to use for BER.
//!
//! This module provides encoding rules that can be used with
//! the `BerSerialize` and `BerDerserialize` traits.
//!
//! ```
//! use asn1_cereal::{BerSerialize, DER};
//!
//! let mut bytes: Vec<u8> = Vec::new();
//! let _ = BerSerialize::serialize_enc(&4u64, DER, &mut bytes);
//! ```

#[derive(PartialEq)]
/// An enum which determines how tags are encoded.
pub enum TagEnc {
  /// When encoding, all tags will be encoded as explicit tags.
  /// When decoding, implict or explict tags will be accepted.
  /// Implicit: Tags that can be encoded implicitly must
  Explicit,
  /// When encoding, constructed tags will be encoded as implicit
  /// tags where possible.
  /// When decoding, it's an error to use an explicit tag for
  /// a constructed element, where an implicit tag could be
  /// used instead.
  Implicit,
}

#[derive(PartialEq)]
/// An enum which determines how lengths are encoded.
pub enum LenEnc {
  /// When encoding, all lengths are encoded using the definite form.
  /// When decoding, it's an error for lengths to be encoded in
  /// anything but definite form.
  Definite,
  /// When encoding, all lengths for constructed elements are
  /// encoded using the indefinite form.
  /// When decoding, definite and indefinite lengths are accepted.
  Indefinite,
}

/// A trait to define encoding rules to use while encoding ASN.1.
pub trait BerEncRules: Copy {
  /// Get the encoding rules for tags.
  fn tag_rules() -> TagEnc;

  /// Get the encoding rules for lengths.
  fn len_rules() -> LenEnc;

  /// Returns true when primitive elements should be encoded using
  /// the shortest form. If this is true when decoding, it should
  /// be an error to not use the shortest form.
  fn shortest_form() -> bool;
}

#[derive(Copy, Clone)]
/// Distinguished Encoding Rules are a subset of BER, and provide a
/// deterministic, shortest form of encoding. These are the default
/// encoding rules used when encoding ASN.1.
///
/// Note: When decoding elements, invalid DER will cause an error.
/// If this is an issue, the BER encoding rules should be used
pub struct DER;

impl BerEncRules for DER {
  fn tag_rules() -> TagEnc {
    TagEnc::Implicit
  }

  fn len_rules() -> LenEnc {
    LenEnc::Definite
  }

  fn shortest_form() -> bool {
    true
  }
}

#[derive(Copy, Clone)]
/// Basic Encoding Rules define the most basic rules that can be
/// used to encode an ASN.1 tag. These are the default encoding rules
/// used when decoding ASN1, as all variants are valid BER.
///
/// When encoding, explicit tags and definite length encoding will
/// be used for all elements.
pub struct BER;

impl BerEncRules for BER {
  fn tag_rules() -> TagEnc {
    TagEnc::Explicit
  }

  fn len_rules() -> LenEnc {
    LenEnc::Definite
  }

  fn shortest_form() -> bool {
    false
  }
}

#[derive(Copy, Clone)]
/// This set of rules creates a valid BER stream, but will use
/// implicit tags where possible, and indefinite length encoding
/// for all constructed elements.
/// 
/// Since definite length encoding must generate the element
/// data before the tag, and indefinite length encoding does not, this
/// is the ideal set of rules for large ASN.1 streams.
///
/// Note. Due to indefinite length encoding, this will not produce a valid
/// DER encoding.
pub struct BERAlt;

impl BerEncRules for BERAlt {
  fn tag_rules() -> TagEnc {
    TagEnc::Implicit
  }

  fn len_rules() -> LenEnc {
    LenEnc::Indefinite
  }

  fn shortest_form() -> bool {
    true
  }
}
