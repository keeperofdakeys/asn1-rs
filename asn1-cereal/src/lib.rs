//! # asn1-cereal
//! A collection of encoders and decoders for BER, DER and ASN.1.
//!
//! The grains of this library are a collection of traits and macros, that
//! allow serialization and deserialization of rust types to and from ASN.1.
//!
//! The `Asn1Info`, `BerSerialize` and `BerDeserialize` traits are what
//! most users will want to use.
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
//!     z;
//!     y;
//!   );
//!
//!   use asn1_cereal::BerSerialize;
//!
//!   let data = ShortSequence{ z: 1, y: 2 };
//!   let mut bytes: Vec<u8> = Vec::new();
//!   BerSerialize::serialize(&data, &mut bytes).unwrap();
//! }
//! ```

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;

pub mod tag;
pub mod err;
pub mod byte;
#[macro_use]
pub mod info;
pub mod ber;

pub use info::Asn1Info;
pub use ber::serial::traits::{BerSerialize, BerDeserialize};
pub use ber::enc::{BER, DER, BERAlt, BerEncRules};
pub use ber::serial::prim::OctetString;
