//! The macros in this module can implement the serialisation traits for a
//! simple ASN.1 definition. IE: One ASN.1 type being defined as an alias of
//! another.
//!
//! ```text
//! TYPE1 ::= SEQUENCE {
//!   a PrintableString
//! }
//!
//! TYPE1 ::= TYPE2
//! ```
//!
//! This mdoule assumes this is being represented in Rust as a newtype.
//! 
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct Type1 {
//!     a: String
//!   }
//!
//!   asn1_sequence_info!(Type1, "TYPE1");
//!   asn1_sequence_serialize!(Type1, a);
//!   asn1_sequence_deserialize!(Type1, a);
//!
//!   struct Type2 (Type1);
//!   asn1_info!(Type2, 0x3, 0x1, true, "TYPE2");
//!   asn1_newtype_serialize!(Type2);
//!   asn1_newtype_deserialize!(Type2);
//! }
//! ```

#[macro_export]
/// This macro will generate an Asn1Serialize implementation for an
/// anonymous struct (a newtype).
macro_rules! asn1_newtype_serialize {
  ($rs_type:ident) => (
    impl $crate::serial::Asn1Serialize for $rs_type {
      fn serialize_bytes<E: $crate::enc::Asn1EncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        self.0.serialize_enc(e, writer)
      }
    }
  )
}

#[macro_export]
/// This macro will generate an Asn1Deserialize implementation for an
/// anonymous struct (a newtype).
macro_rules! asn1_newtype_deserialize {
  ($rs_type:ident) => (
    impl $crate::serial::Asn1Deserialize for $rs_type {
      fn deserialize_bytes<E: $crate::enc::Asn1EncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, len: Option<$crate::tag::LenNum>) -> Result<Self, $crate::err::DecodeError> {
        Ok($rs_type(try!($crate::serial::Asn1Deserialize::deserialize_enc(e, reader, len))))
      }
    }
  )
}
