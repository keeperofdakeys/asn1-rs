//! Encoding and Decoding errors that this crate can produce.

use std::io;

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
}

impl From<io::Error> for EncodeError {
  fn from(err: io::Error) -> Self {
    EncodeError::IO(err)
  }
}
