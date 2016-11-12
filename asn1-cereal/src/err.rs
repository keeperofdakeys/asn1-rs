//! Encoding and Decoding errors that this crate can produce.

use std::io;
use tag::{Tag, Type};
use std::error::Error;

#[allow(dead_code)]
/// An error that occurs while decoding an ASN.1 element.
pub struct DecodeError2 {
  /// If relevant, the tag for the element being decoded when the error occured.
  tag: Option<Tag>,
  /// If relevant, the ASN.1 type for the element being decoced when the error occured.
  asn1_type: Option<Type>,
  /// Optionally, an error that caused this error.
  inner: Option<Box<Error>>,
  /// Optionally, a string containing more information about this error.
  more: Option<&'static str>,
}

#[allow(dead_code)]
struct TypeInfo {
  tag: Option<Tag>,
  asn1_type: Type,
}

#[derive(Debug)]
/// Errors that can occur while decoding an ASN.1 element.
pub enum DecodeError {
  /// Generic IO Error.
  IO(io::Error),
  /// Child element(s) decoded to greater length than the parent's tag.
  GreaterLen,
  /// Child element(s) decoded to smaller length than the parent's tag.
  SmallerLen,
  /// Primitive value encoded with an indefinite length.
  PrimIndef,
  /// Decoded tag does not match the expected tag for this type.
  TagTypeMismatch,
  /// An explicit tag appeared where an Implicit tag was expected.
  ExplicitTag,
  /// Indefinite length encoding appeared when definite length encoding was
  /// expected.
  IndefiniteLen,
  /// Indefinite length encoding was started, but no terminator was found
  /// at the end.
  IndefiniteLenEnd,
  /// Custom decoding error.
  Custom(&'static str),
}

impl From<io::Error> for DecodeError {
  fn from(err: io::Error) -> Self {
    DecodeError::IO(err)
  }
}

#[derive(Debug)]
/// Errors that can occur while encoding an ASN.1 element.
pub enum EncodeError {
  /// Generic IO Error.
  IO(io::Error),
  /// Custom encoding error.
  Custom(&'static str),
}

impl From<io::Error> for EncodeError {
  fn from(err: io::Error) -> Self {
    EncodeError::IO(err)
  }
}
