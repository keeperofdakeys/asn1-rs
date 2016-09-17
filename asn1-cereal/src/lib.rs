//! # asn1-cereal
//! A collection of encoders and decoders for BER, DER and ASN.1.
//!
//! The grains of this library are a collection of traits and macros, that
//! allow serialization and deserialization of rust types to and from ASN.1.
//!
//! The `Asn1Info`, `BerSerialize` and `BerDeserialize` traits are what
//! most users will want to use.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

pub mod tag;
pub mod err;
pub mod byte;
#[macro_use]
pub mod info;
pub mod ber;

pub use info::Asn1Info;
pub use ber::serial::traits::{BerSerialize, BerDeserialize};
pub use ber::enc::{BER, DER, BERAlt, BerEncRules};
