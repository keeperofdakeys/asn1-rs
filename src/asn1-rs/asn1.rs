use std::fmt;
use std::cmp::Ordering;

pub type Asn1LenNum = u64;

#[derive(PartialEq, Debug)]
/// An enum representing the length of an ASN.1 element.
pub enum Asn1Len {
  /// A Definite length element.
  Def(u64),
  /// An Indefinite length element, not known before decoding.
  Indef,
}

impl From<Asn1LenNum> for Asn1Len {
  fn from(len: Asn1LenNum) -> Self {
    match len {
      0 => Asn1Len::Indef,
      l => Asn1Len::Def(l),
    }
  }
}

impl From<Asn1Len> for Asn1LenNum {
  fn from(len: Asn1Len) -> Self {
    match len {
      Asn1Len::Def(l) => l,
      Asn1Len::Indef => 0,
    }
  }
}

impl PartialOrd<Asn1Len> for Asn1Len {
  fn partial_cmp(&self, other: &Asn1Len) -> Option<Ordering> {
    match (self, other) {
      (&Asn1Len::Def(ref l),
        &Asn1Len::Def(ref r)) => Some(l.cmp(r)),
      _ => None,
    }
  }
}

impl PartialEq<Asn1LenNum> for Asn1Len {
  fn eq(&self, other: &Asn1LenNum) -> bool {
    match *self {
      Asn1Len::Def(ref l) => l.eq(other),
      Asn1Len::Indef => false,
    }
  }
}

impl PartialOrd<Asn1LenNum> for Asn1Len {
  fn partial_cmp(&self, other: &Asn1LenNum) -> Option<Ordering> {
    match *self {
      Asn1Len::Def(ref l) => Some(l.cmp(other)),
      Asn1Len::Indef => None,
    }
  }
}

/// An ASN.1 tag number.
pub type Asn1TagNum = u64;

#[derive(PartialEq, Debug)]
/// An ASN.1 Class.
pub enum Asn1Class {
  /// Universal class.
  Universal,
  /// Application class.
  Application,
  /// Context-specific class.
  ContextSpecific,
  /// Private class.
  Private,
}

impl From<u8> for Asn1Class {
  fn from(len: u8) -> Self {
    match len {
      0 => Asn1Class::Universal,
      1 => Asn1Class::Application,
      2 => Asn1Class::ContextSpecific,
      3 => Asn1Class::Private,
      _ => unreachable!()
    }
  }
}

impl fmt::Display for Asn1Class {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", match *self {
      Asn1Class::Universal => "Universal",
      Asn1Class::Application => "Application",
      Asn1Class::ContextSpecific => "Context-specific",
      Asn1Class::Private => "Private",
    })
  }
}

/// A struct representing an ASN.1 element.
pub struct Asn1Tag {
  /// The class of the ASN.1 element.
  pub class: Asn1Class,
  /// The tag number of the ASN.1 element.
  pub tagnum: Asn1TagNum,
  /// The length of the ASN.1 element.
  pub len: Asn1Len,
  /// A flag indicating whether an element is constructed.
  pub constructed: bool,
}

impl Asn1Tag {
  /// Returns true when this is a structured type.
  pub fn is_structured(&self) -> bool {
    if self.class == Asn1Class::Universal {
      match self.tagnum {
        // SEQUENCE (OF)
        16 => true,
        // SET (OF)
        17 => true,
        _ => false,
      }
    } else {
      false
    }
  }
}

type Asn1Type = String;

trait Asn1Data {
  fn get_asn1_type() -> Asn1Type;

  // /// Create ASN.1 data from this struct.
  // FIXME: Should this use &self?
  // fn into_asn1(&self) -> Result<Asn1Data, Asn1Error>;

  // /// Create this struct from ASN.1 data.
  // fn from_asn1(slice: Asn1Slice) -> Result<Self, Asn1Error>;
}

/// A macro to generate a generic Asn1Data trait implementation for a struct.
macro_rules! asn1_impl {
  ( $impl_type:ident, $asn1_type:expr,
    $( $name:ident, $rusttype:ident, $asn1type:expr ),*
  ) =>
(

impl Asn1Data for $impl_type {
  fn get_asn1_type() -> Asn1Type {
    $asn1_type
  }

  fn into_asn1(&self) -> Result<Asn1Data, Asn1Error> {
    Err(Asn1Error::EncodingError)
  }

  fn from_asn1(slice: Asn1Slice) -> Result<Self, Asn1Error> {
    Err(Asn1Error::InvalidAsn1)
  }
}

)
}

/// A list of errors that can occur decoding or encoding Asn1 data.
enum Asn1Error {
  /// Invalid Asn1 data.
  InvalidAsn1,
  /// An error occured while encoding Asn1 data.
  EncodingError,
  /// An invalid tag was decoded
  InvalidTag(Asn1Tag),
}
