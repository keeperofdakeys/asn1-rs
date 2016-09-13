//! # asn1-cereal
//! A collection of encoders and decoders for BER, DER and ASN.1.
//!
//! The grains of this library are a collection of traits and macros, that
//! allow serialization and deserialization of rust types to and from ASN.1.
//!
//! New users of this library probably want to start with the [`serial`] library.
//!
//! [`serial`]: serial/index.html

pub mod tag;
pub mod err;
pub mod byte;
pub mod info;
pub mod ber;

pub use info::Asn1Info;
pub use ber::serial::traits::{BerSerialize, BerDeserialize};
pub use tag::{Type, Class, LenNum, Len, Tag};
