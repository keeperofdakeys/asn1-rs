use std::fmt;
use std::cmp::Ordering;
use std::io;

pub type Asn1LenNum = u64;

#[derive(PartialEq, Debug)]
/// An enum representing the length of an ASN.1 element.
pub enum Asn1Len {
  /// A Definite length element.
  Def(u64),
  /// An Indefinite length element, not known before decoding.
  Indef,
}

impl From<Option<Asn1LenNum>> for Asn1Len {
  fn from(len: Option<Asn1LenNum>) -> Self {
    match len {
      None => Asn1Len::Indef,
      Some(l) => Asn1Len::Def(l),
    }
  }
}

impl From<Asn1Len> for Option<Asn1LenNum> {
  fn from(len: Asn1Len) -> Self {
    match len {
      Asn1Len::Def(l) => Some(l),
      Asn1Len::Indef => None,
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

impl fmt::Display for Asn1Len {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Asn1Len::Def(ref l) => write!(f, "{}", l),
      Asn1Len::Indef => write!(f, "Indefinite Length"),
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
  fn from(class: u8) -> Self {
    match class {
      0 => Asn1Class::Universal,
      1 => Asn1Class::Application,
      2 => Asn1Class::ContextSpecific,
      3 => Asn1Class::Private,
      _ => unreachable!()
    }
  }
}

impl From<Asn1Class> for u8 {
  fn from(class: Asn1Class) -> Self {
    match class {
      Asn1Class::Universal => 0,
      Asn1Class::Application => 1,
      Asn1Class::ContextSpecific => 2,
      Asn1Class::Private => 3,
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

#[derive(Debug)]
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

  /// Decode an ASN.1 tag from a stream.
  pub fn decode_tag<R: io::Read>(reader: &mut R) -> Result<(Self, Asn1LenNum), Asn1DecodeError> {
    let mut bytes = ByteReader::new(reader);

    // Decode tag byte, which includes class, constructed flag, and tag number.
    let tag_byte = try!(bytes.read());
    let class_num = (tag_byte & 0xc0) >> 6;
    let constructed = tag_byte & 0x20 == 0x20;
    // If tag is 0x1F, use extended decode format.
    let tag = if (tag_byte & 0x1f) == 0x1f {
      let mut tag: Asn1TagNum = 0;
      loop {
        // Incrementatlly read bytes, adding base-128 to tag.
        let tag_more = try!(bytes.read());
        tag = (tag << 7) + (tag_more & 0x7f) as Asn1TagNum;
        // Stop looping when 0x80 bit is set.
        if tag_more & 0x80 == 0x00 {
          break;
        }
      }
      tag
    // Otherwise it's just bits 5-1.
    } else {
      (tag_byte & 0x1f) as Asn1TagNum
    };

    // Decode len byte.
    let len_byte = try!(bytes.read());
    let len = match len_byte {
      // When byte is 0x80, this is the start of indefinite length encoding.
      0x80 => Asn1Len::Indef,
      // If 0x80 is set, then other bits indicate the number of len bytes.
      l => if (l & 0x80) == 0x80 {
          let mut len: Asn1LenNum = 0;
          let byte_count = l & 0x7f;
          // Loop through number of len bytes.
          for _ in 0..byte_count {
            let len_more = try!(bytes.read());
            // Add up each byte base-256.
            len = (len << 8) + len_more as Asn1TagNum;
          }
          Asn1Len::Def(len)
        // If 0x80 bit is not set, just decode the value.
        } else {
          Asn1Len::Def(l as Asn1LenNum)
        },
    };

    Ok((Asn1Tag {
      class: Asn1Class::from(class_num),
      tagnum: tag,
      len: len,
      constructed: constructed,
    }, bytes.count))
  }

  /// Encode an ASN.1 stream from a tag.
  pub fn encode_tag<W: io::Write>(self, writer: &mut W) -> Result<Asn1LenNum, Asn1EncodeError> {
    let mut bytes = ByteWriter::new(writer);
    let (class, tagnum, len, constructed) =
      (self.class, self.tagnum, self.len, self.constructed);

    // Create first tag_byte from class, constructed and tag number.
    let mut tag_byte = u8::from(class) << 6;
    if constructed {
      tag_byte += 0x20;
    }
    // If tag number is <31, add to single byte.
    if tagnum < 31 {
      tag_byte += tagnum as u8 & 0x1f;
      try!(bytes.write(tag_byte));
    // Otherwise build additional tag bytes.
    } else {
      tag_byte += 0x1f;
      try!(bytes.write(tag_byte));
    }

    Ok(bytes.count)
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

/// A reader to easily read a byte from a reader.
struct ByteReader<'a, R: io::Read + 'a> {
  reader: &'a mut R,
  pub count: u64,
}

impl<'a, R: io::Read + 'a> ByteReader<'a, R> {
  fn new(reader: &'a mut R) -> ByteReader<'a, R> {
    ByteReader {
      reader: reader,
      count: 0
    }
  }

  /// Read a byte from a reader.
  fn read(&mut self) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    // FIXME: Should retry on the Interrupted Error, and perhaps another error.
    match try!(self.reader.read(&mut buf)) {
      0 => Err(io::Error::new(io::ErrorKind::Other, "Read zero bytes")),
      1 => {
        self.count += 1;
        Ok(buf[0])
      },
      _ => Err(io::Error::new(io::ErrorKind::Other, "Read more than one byte")),
    }
  }
}

/// A writer to easily write a byte to a writer.
struct ByteWriter<'a, W: io::Write + 'a> {
  writer: &'a mut W,
  pub count: u64,
}

impl<'a, W: io::Write + 'a> ByteWriter<'a, W> {
  fn new(writer: &'a mut W) -> ByteWriter<'a, W> {
    ByteWriter {
      writer: writer,
      count: 0
    }
  }

  /// Write a byte to a writer.
  fn write(&mut self, byte: u8) -> io::Result<()> {
    let buf = [byte];
    match try!(self.writer.write(&buf)) {
      0 => Err(io::Error::new(io::ErrorKind::Other, "Wrote zero bytes")),
      1 => {
        self.count += 1;
        Ok(())
      },
      _ => Err(io::Error::new(io::ErrorKind::Other, "Wrote more than one byte")),
    }
  }
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

#[derive(Debug)]
/// Errors that can occur while decoding an ASN.1 element.
pub enum Asn1DecodeError {
  /// Generic IO Error.
  IO(io::Error),
  /// Child element(s) decoded to greater length than the parent's tag.
  GreaterLen,
  /// Primitive value encoded with an indefinite length.
  PrimIndef,
}

impl From<io::Error> for Asn1DecodeError {
  fn from(err: io::Error) -> Self {
    Asn1DecodeError::IO(err)
  }
}

#[derive(Debug)]
/// Errors that can occur while encoding an ASN.1 element.
pub enum Asn1EncodeError {
  /// Generic IO Error.
  IO(io::Error),
}

impl From<io::Error> for Asn1EncodeError {
  fn from(err: io::Error) -> Self {
    Asn1EncodeError::IO(err)
  }
}
