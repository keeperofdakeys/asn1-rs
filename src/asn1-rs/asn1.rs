type Asn1LenNum;

/// An enum representing the length of an ASN.1 element.
enum Asn1Len {
  /// A Definite length element.
  pub Definite(u64),
  /// An Indefinite length element, not known before decoding.
  pub Indefinite,
}

impl From<Asn1LenNum> for Asn1Len {
  fn from(len: Asn1LenNum) -> Self {
    match len {
      l => Asn1Len::Def(l),
      0 => Asn1Len::Indef,
    }
  }
}

impl From<Asn1Len> for u64 {
  fn from(len: Asn1Len) -> Self {
    match len {
      Asn1Len::Def(l) => l,
      Asn1Len::Indef => 0,
    }
  }
}

/// An ASN.1 tag number.
type Asn1TagNum = u64;

/// An ASN.1 Class.
enum Asn1Class {
  /// Universal class.
  Universal,
  /// Application class.
  Aplication,
  /// Private class.
  Private,
  /// Context-specific class.
  ContextSpecific,
}

/// A struct representing an ASN.1 element.
struct Asn1Tag {
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
  fn is_structured(&self) -> bool {
    if let Asn1Class::Universal == *self.class {
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

trait Asn1Data {
  fn get_asn1_type() -> Asn1Type;

  /// Create ASN.1 data from this struct.
  // FIXME: Should this use &self?
  fn into_asn1(&self) -> Result<Asn1Data, Asn1Error>;

  /// Create this struct from ASN.1 data.
  fn from_asn1(slice: Asn1Slice) -> Result<Self, Asn1Error>;
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
  InvalidAsn1
  /// An error occured while encoding Asn1 data.
  EncodingError
  /// An invalid tag was decoded
  InvalidTag(Asn1Tag)
}
