use std::fmt;
use std::cmp::Ordering;
use std::io;

pub type LenNum = u64;

#[derive(PartialEq, Debug)]
/// An enum representing the length of an ASN.1 element.
pub enum Len {
  /// A Definite length element.
  Def(u64),
  /// An Indefinite length element, not known before decoding.
  Indef,
}

impl From<Option<LenNum>> for Len {
  fn from(len: Option<LenNum>) -> Self {
    match len {
      None => Len::Indef,
      Some(l) => Len::Def(l),
    }
  }
}

impl From<Len> for Option<LenNum> {
  fn from(len: Len) -> Self {
    match len {
      Len::Def(l) => Some(l),
      Len::Indef => None,
    }
  }
}

impl PartialOrd<Len> for Len {
  fn partial_cmp(&self, other: &Len) -> Option<Ordering> {
    match (self, other) {
      (&Len::Def(ref l),
        &Len::Def(ref r)) => Some(l.cmp(r)),
      _ => None,
    }
  }
}

impl PartialEq<LenNum> for Len {
  fn eq(&self, other: &LenNum) -> bool {
    match *self {
      Len::Def(ref l) => l.eq(other),
      Len::Indef => false,
    }
  }
}

impl PartialOrd<LenNum> for Len {
  fn partial_cmp(&self, other: &LenNum) -> Option<Ordering> {
    match *self {
      Len::Def(ref l) => Some(l.cmp(other)),
      Len::Indef => None,
    }
  }
}

impl fmt::Display for Len {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Len::Def(ref l) => write!(f, "{}", l),
      Len::Indef => write!(f, "Indefinite Length"),
    }
  }
}

/// An ASN.1 tag number.
pub type TagNum = u64;

#[derive(PartialEq, Debug)]
/// An ASN.1 Class.
pub enum Class {
  /// Universal class.
  Universal,
  /// Application class.
  Application,
  /// Context-specific class.
  ContextSpecific,
  /// Private class.
  Private,
}

impl From<u8> for Class {
  fn from(class: u8) -> Self {
    match class {
      0 => Class::Universal,
      1 => Class::Application,
      2 => Class::ContextSpecific,
      3 => Class::Private,
      _ => unreachable!()
    }
  }
}

impl From<Class> for u8 {
  fn from(class: Class) -> Self {
    match class {
      Class::Universal => 0,
      Class::Application => 1,
      Class::ContextSpecific => 2,
      Class::Private => 3,
    }
  }
}

impl fmt::Display for Class {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", match *self {
      Class::Universal => "Universal",
      Class::Application => "Application",
      Class::ContextSpecific => "Context-specific",
      Class::Private => "Private",
    })
  }
}

#[derive(Debug)]
/// A struct representing an ASN.1 element.
pub struct Tag {
  /// The class of the ASN.1 element.
  pub class: Class,
  /// The tag number of the ASN.1 element.
  pub tagnum: TagNum,
  /// The length of the ASN.1 element.
  pub len: Len,
  /// A flag indicating whether an element is constructed.
  pub constructed: bool,
}

impl Tag {
  /// Returns true when this is a structured type.
  pub fn is_structured(&self) -> bool {
    if self.class == Class::Universal {
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
  pub fn decode_tag<R: io::Read>(reader: &mut R) -> Result<(Self, LenNum), DecodeError> {
    let mut bytes = ByteReader::new(reader);

    // Decode tag byte, which includes class, constructed flag, and tag number.
    let tag_byte = try!(bytes.read());
    let class_num = (tag_byte & 0xc0) >> 6;
    let constructed = tag_byte & 0x20 == 0x20;
    // If tag is 0x1F, use extended decode format.
    let tag = if (tag_byte & 0x1f) == 0x1f {
      let mut tag: TagNum = 0;
      loop {
        // Incrementatlly read bytes, adding base-128 to tag.
        let tag_more = try!(bytes.read());
        tag = (tag << 7) + (tag_more & 0x7f) as TagNum;
        // Stop looping when 0x80 bit is set.
        if tag_more & 0x80 == 0x00 {
          break;
        }
      }
      tag
    // Otherwise it's just bits 5-1.
    } else {
      (tag_byte & 0x1f) as TagNum
    };

    // Decode len byte.
    let len_byte = try!(bytes.read());
    let len = match len_byte {
      // When byte is 0x80, this is the start of indefinite length encoding.
      0x80 => Len::Indef,
      // If 0x80 is set, then other bits indicate the number of len bytes.
      l => if (l & 0x80) == 0x80 {
          let mut len: LenNum = 0;
          let byte_count = l & 0x7f;
          // Loop through number of len bytes.
          for _ in 0..byte_count {
            let len_more = try!(bytes.read());
            // Add up each byte base-256.
            len = (len << 8) + len_more as TagNum;
          }
          Len::Def(len)
        // If 0x80 bit is not set, just decode the value.
        } else {
          Len::Def(l as LenNum)
        },
    };

    Ok((Tag {
      class: Class::from(class_num),
      tagnum: tag,
      len: len,
      constructed: constructed,
    }, bytes.count))
  }

  /// Encode an ASN.1 stream from a tag.
  pub fn encode_tag<W: io::Write>(self, writer: &mut W) -> Result<LenNum, EncodeError> {
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

type Type = String;

trait Data {
  fn get_asn1_type() -> Type;

  // /// Create ASN.1 data from this struct.
  // FIXME: Should this use &self?
  // fn into_asn1(&self) -> Result<Data, Error>;

  // /// Create this struct from ASN.1 data.
  // fn from_asn1(slice: Slice) -> Result<Self, Error>;
}

/// A macro to generate a generic Data trait implementation for a struct.
macro_rules! asn1_impl {
  ( $impl_type:ident, $asn1_type:expr,
    $( $name:ident, $rusttype:ident, $asn1type:expr ),*
  ) =>
(

impl Data for $impl_type {
  fn get_asn1_type() -> Type {
    $asn1_type
  }

  fn into_asn1(&self) -> Result<Data, Error> {
    Err(Error::EncodingError)
  }

  fn from_asn1(slice: Slice) -> Result<Self, Error> {
    Err(Error::Invalid)
  }
}

)
}

/// A reader to easily read a byte from a reader.
pub struct ByteReader<'a, R: io::Read + 'a> {
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
}

impl<'a, R: io::Read + 'a> Iterator for ByteReader<'a, R> {
  type Item = io::Result<u8>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut buf = [0];
    match self.reader.read(&mut buf) {
      Ok(1) => Some(Ok(1)),
      Err(e) => Some(Err(e)),
      _ => None,
    }
  }
}

impl<'a, R: io::Read + 'a> From<&'a mut R> for ByteReader<'a, R> {
  fn from(reader: &'a mut R) -> Self {
    ByteReader {
      count: 0,
      reader: reader,
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

/// A list of errors that can occur decoding or encoding  data.
enum Error {
  /// Invalid  data.
  Invalid,
  /// An error occured while encoding  data.
  EncodingError,
  /// An invalid tag was decoded
  InvalidTag(Tag),
}

#[derive(Debug)]
/// Errors that can occur while decoding an ASN.1 element.
pub enum DecodeError {
  /// Generic IO Error.
  IO(io::Error),
  /// Child element(s) decoded to greater length than the parent's tag.
  GreaterLen,
  /// Primitive value encoded with an indefinite length.
  PrimIndef,
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
