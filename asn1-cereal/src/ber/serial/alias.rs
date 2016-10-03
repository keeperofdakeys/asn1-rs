//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct A(u64);
//!
//!   // Alias with no tag
//!   ber_alias!(A ::= u64, "A");
//!
//!   // OR
//!   struct B(u64);
//!
//!   // Alias with a custom tag.
//!   ber_alias!(B ::= [APPLICATION 3] u64, "B");
//! }
//! ```

#[macro_export]
macro_rules! ber_alias {
  ($outer:ident ::= [$($args:tt)*] $inner:ident, $asn1_ty:expr) => (
    ber_alias_info!($outer ::= [$($args)*] $inner, $asn1_ty);
    ber_alias_serialize!($outer ::= $inner);
    ber_alias_deserialize!($outer ::= $inner);
  );
  ($outer:ident ::= $inner:ident, $asn1_ty:expr) => (
    ber_alias_info!($outer ::= $inner, $asn1_ty);
    ber_alias_serialize!($outer ::= $inner);
    ber_alias_deserialize!($outer ::= $inner);
  );
}

#[macro_export]
macro_rules! ber_alias_info {
  ($outer:ident ::= [$($args:tt)*] $inner:ident, $asn1_ty:expr) => (
    asn1_info!(
      $outer,
      [$($args)*],
      $asn1_ty
    );
  );
  ($outer:ident ::= $inner:ident, $asn1_ty:expr) => (
    asn1_info!(
      $outer,
      $asn1_ty
    );
  );
}

#[macro_export]
/// This macro defines the BerSerialize trait for an ASN.1 type alias.
macro_rules! ber_alias_serialize {
  ($outer:ident ::= $inner:ident) => (
    impl $crate::BerSerialize for $outer {
      fn _serialize_enc<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Option<Result<(), $crate::err::EncodeError>> {
        let tag = <Self as $crate::Asn1Info>::asn1_tag();
        // If we have a tag, and encoding uses implicit tags, skip our tag.
        if tag.is_some() && E::tag_rules() == $crate::ber::enc::TagEnc::Implicit {
          Some(self.serialize_value(e, writer))
        } else {
          None
        }
      }

      fn serialize_value<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        self.0.serialize_enc(e, writer)
      }
    }
  )
}

#[macro_export]
/// This macro defines the BerSerialize trait for an ASN.1 type alias.
macro_rules! ber_alias_deserialize {
  ($outer:ident ::= $inner:ident) => (
    impl $crate::BerDeserialize for $outer {
      fn _deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len)
          -> Option<Result<Self, $crate::err::DecodeError>> {
        // If we don't have a tag, decode the internal item.
        let my_tag = match <Self as $crate::Asn1Info>::asn1_tag() {
          Some(tag) => tag,
          None => return Some(
            $crate::BerDeserialize::deserialize_with_tag(e, reader, tag, len)
            .and_then(|s| Ok($outer(s)))
          ),
        };

        // If we're decoding using Implicit tagging rules, throw an error if this isn't an implicit tag.
        if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit && tag == my_tag {
          return Some(Err($crate::err::DecodeError::ExplicitTag));
        }

        if tag != my_tag {
          let res =
            $crate::BerDeserialize::deserialize_with_tag(e, reader, tag, len)
            .and_then(|s| Ok($outer(s)));
          Some(res)
        } else {
          None
        }
      }

      fn deserialize_value<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, len: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        Ok($outer(try!($crate::BerDeserialize::deserialize_enc(e, reader))))
      }
    }
  )
}
