//! Traits for serializing and deserializing rust types from/to ASN.1.
//!
//! The traits in this module can be used to implement serialization and
//! deserialization for a Rust type from/to ASN.1. For built-in types,
//! this module provides an implementation already. For more complex
//! types (like structs and enumms), macros are provided to generate
//! an implementation for them.

pub mod traits;
pub mod prim;

pub mod int;
pub mod str;
pub mod bool;
pub mod seq_of;
