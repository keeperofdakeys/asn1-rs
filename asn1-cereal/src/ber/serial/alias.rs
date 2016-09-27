//! Macros to generate the implementation of the serialization traits for Rust
//! newtypes, as ASN.1 type assignments. This doesn't create any explicit tag,
//! and any serialization functions call the inner functions directly.
//!
//! You should only need to call `ber_alias!`.
//!
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct A(u64);
//!
//!   ber_alias!(A, u64);
//! }
//! ```

// MERGE THIS WITH NEWTYPE
//
// We can do this now that asn1_tag returns an Option.

#[macro_export]
macro_rules! ber_alias {
  ($outer:ident, $inner:ident) => (
    ber_alias_info!($outer, $inner);
    ber_alias_serialize!($outer, $inner);
    ber_alias_deserialize!($outer, $inner);
  )
}

#[macro_export]
macro_rules! ber_alias_info {
  ($outer:ident, $inner:ident) => (
    impl $crate::Asn1Info for $outer {
      fn asn1_tag() -> Option<$crate::tag::Tag> {
        <$inner as $crate::Asn1Info>::asn1_tag()
      }

      fn asn1_type() -> $crate::tag::Type {
        <$inner as $crate::Asn1Info>::asn1_type()
      }
    }
  )
}

#[macro_export]
/// This macro defines the BerSerialize trait for a rust newtype.
macro_rules! ber_alias_serialize {
  ($outer:ident, $inner:ident) => (
    impl $crate::BerSerialize for $outer {
      fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        <$inner as $crate::BerSerialize>::serialize(&self.0, writer)
      }

      fn serialize_enc<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        <$inner as $crate::BerSerialize>::serialize_enc(&self.0, e, writer)
      }

      fn serialize_value<E: $crate::BerEncRules, W: std::io::Write>
        (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        <$inner as $crate::BerSerialize>::serialize_value(&self.0, e, writer)
      }
    }
  )
}

#[macro_export]
/// This macro defines the BerSerialize trait for a rust newtype.
macro_rules! ber_alias_deserialize {
  ($outer:ident, $inner:ident) => (
    impl $crate::BerDeserialize for $outer {
      fn deserialize<I: Iterator<Item=std::io::Result<u8>>>(reader: &mut I)
          -> Result<Self, $crate::err::DecodeError> {
        <$inner as $crate::BerDeserialize>::deserialize(reader).map(|v| $outer(v))
      }

      fn deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len)
          -> Result<Self, $crate::err::DecodeError> {
        <$inner as $crate::BerDeserialize>::deserialize_with_tag(e, reader, tag, len).map(|v| $outer(v))
      }

      fn _deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len)
          -> Option<Result<Self, $crate::err::DecodeError>> {
        <$inner as $crate::BerDeserialize>::_deserialize_with_tag(e, reader, tag, len)
          .map(|r| r.map(|v| $outer(v)))
      }

      fn deserialize_value<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, len: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        <$inner as $crate::BerDeserialize>::deserialize_value(e, reader, len).map(|v| $outer(v))
      }
    }
  )
}
