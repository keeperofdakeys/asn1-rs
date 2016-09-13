//! Traits for serializing and deserializing rust types from/to ASN.1.
//!
//! The traits in this module can be used to implement serialization and
//! deserialization for a Rust type from/to ASN.1. For built-in types,
//! this module provides an implementation already. For more complex
//! types (like structs and enumms), macros are provided to generate
//! an implementation for them.
//!
//! # Usage
//! The below example defines a Rust struct, and uses a macro to generate
//! the code required to serialize it to an ASN.1 sequence, or to deserialize
//! an ASN.1 sequence into an instance of the struct.
//!
//! The example then uses the BerSerialize::serialize function to encode
//! the struct to a DER-encoded byte stream.
//!
//!
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct ShortSequence {
//!     z: u64,
//!     y: u32,
//!   }
//!
//!   asn1_sequence!(
//!     ShortSequence,
//!     "SHORT_SEQUENCE",
//!     z,
//!     y
//!   );
//!
//!   use asn1_cereal::BerSerialize;
//!
//!   let data = ShortSequence{ z: 1, y: 2 };
//!   let mut bytes: Vec<u8> = Vec::new();
//!   BerSerialize::serialize(&data, &mut bytes).unwrap();
//! }
//! ```

#[macro_export]
/// This macro defines the Asn1Info trait for a rust type.
///
/// This information is used to match tag information during deserialization,
/// so it should match the expected values in the ASN.1 stream.
macro_rules! asn1_info {
  ($rs_type:ty, $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
    impl $crate::Asn1Info for $rs_type {
      fn asn1_tag() -> $crate::tag::Tag {
        $crate::tag::Tag {
          class: ($class as u8).into(),
          tagnum: $tagnum as $crate::tag::TagNum,
          constructed: $constructed,

        }
      }

      fn asn1_type() -> $crate::tag::Type {
        $crate::tag::Type::from($asn1_ty)
      }
    }
  )
}

pub mod traits;
mod prim;

pub mod int;
pub mod str;
pub mod assign;
pub mod choice;
pub mod seq;
pub mod seq_of;
