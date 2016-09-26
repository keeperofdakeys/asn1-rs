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
//! The example then uses the `BerSerialize::serialize` function to encode
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
//!   ber_sequence!(
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

pub mod traits;
mod prim;

pub mod int;
pub mod str;
pub mod assign;
pub mod alias;
pub mod choice;
pub mod seq;
pub mod seq_of;
