type Asn1Len = u64;
type Asn1TagNum = u64;

type Asn1Data = Vec<u8>;
type Asn1Slice<'a> = &'a [u8];

enum Asn1Tag {
  // Universal types here ...
  UniversalTag(Asn1TagNum)
  AplicationTag(Asn1TagNum),
  PrivateTag(Asn1TagNum),
  ContextSpecificTag(Asn1TagNum),
}

impl Asn1Tag {
  /// Returns true when this is a structured type.
  fn is_structured(&self) -> bool {
    if let Asn1Tag::UniversalTag(tag) == *self {
      match tag {
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
