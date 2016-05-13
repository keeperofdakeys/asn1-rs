use byte::{read_byte, write_byte};
use err;

use std::fmt;
use std::cmp::Ordering;
use std::io;

pub type LenNum = u64;

#[derive(PartialEq, Debug, Clone, Copy)]
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

#[derive(PartialEq, Debug, Clone, Copy)]
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

pub type Type = String;

#[derive(PartialEq, Debug, Clone, Copy)]
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
  pub fn decode_tag<I: Iterator<Item=io::Result<u8>>>(bytes: &mut I) ->
    Result<Self, err::DecodeError> {
    // Decode tag byte, which includes class, constructed flag, and tag number.
    let tag_byte = try!(read_byte(bytes));
    let class_num = (tag_byte & 0xc0) >> 6;
    let constructed = tag_byte & 0x20 == 0x20;
    // If tag is 0x1F, use extended decode format.
    let tag = if (tag_byte & 0x1f) == 0x1f {
      let mut tag: TagNum = 0;
      loop {
        // Incrementatlly read bytes, adding base-128 to tag.
        let tag_more = try!(read_byte(bytes));
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
    let len_byte = try!(read_byte(bytes));
    let len = match len_byte {
      // When byte is 0x80, this is the start of indefinite length encoding.
      0x80 => Len::Indef,
      // If 0x80 is set, then other bits indicate the number of len bytes.
      l => if (l & 0x80) == 0x80 {
          let mut len: LenNum = 0;
          let byte_count = l & 0x7f;
          // Loop through number of len bytes.
          for _ in 0..byte_count {
            let len_more = try!(read_byte(bytes));
            // Add up each byte base-256.
            len = (len << 8) + len_more as TagNum;
          }
          Len::Def(len)
        // If 0x80 bit is not set, just decode the value.
        } else {
          Len::Def(l as LenNum)
        },
    };

    Ok(Tag {
      class: Class::from(class_num),
      tagnum: tag,
      len: len,
      constructed: constructed,
    })
  }

  /// Encode an ASN.1 stream from a tag.
  pub fn encode_tag<W: io::Write>(self, writer: &mut W) -> Result<(), err::EncodeError> {
    let (class, tagnum, len, constructed) =
      (self.class, self.tagnum, self.len, self.constructed);

    // Create first tag_byte from class, constructed and tag number.
    let mut tag_byte: u8 = (u8::from(class) << 6) & 0xc0;
    if constructed {
      tag_byte |= 0x20;
    }
    // If tag number is <31, add to single byte.
    if tagnum < 31 {
      tag_byte |= tagnum as u8 & 0x1f;
      try!(write_byte(writer, tag_byte));
    // Otherwise build additional tag bytes.
    } else {
      if tagnum & 0x8000000000000000 != 0 {
        panic!("Bit 63 set on asn1 tag. Not handling, since this is \
                impractically huge, and it messes up my nice little algorithm.");
      }
      tag_byte |= 0x1f;
      try!(write_byte(writer, tag_byte));
      let mut started = false;
      // Take 7 bit slices eg. 62-55, ..., 6-0.
      // The first non-zero slice marks the start of the int.
      for offset in (0..9).rev() {
        // Get 7 bit slice.
        let mut tag_part = ((tagnum >> (offset * 7)) & 0x7f) as u8;

        if !started {
          // Skip if tag_part is zero and we haven't started.
          if tag_part == 0 {
            continue;
          }
          // TODO: Does tagnum have sign issues like length?
          // Emit an initial zero byte if slice starts with a 1 bit.
          // if tag_part & 0x40 != 0 {
          //   try!(write_byte(writer, 0));
          // }
          started = true;
        }

        // For all slices except the last, set 7th bit.
        if offset != 0 {
          tag_part |= 0x80;
        }
        try!(write_byte(writer, tag_part));
      }
    }

    match len {
      Len::Indef => try!(write_byte(writer, 0x80)),
      Len::Def(l) if l < 128 =>
        try!(write_byte(writer, l as u8)),
      Len::Def(l) => {
        let mut started = false;

        // Loop through each eight byte slice of l.
        for offset in (0..8).rev() {
          let len_part: u8 = ((l >> (offset * 8)) & 0xff) as u8;

          if !started {
            // Skip if len_part is zero and we haven't strated.
            if len_part == 0 {
              continue;
            }

            // TODO: Do we need this?
            // Work around some decoders using signed ints.
            // if len_num & 0x80 != 0 {
            //   try!(write_byte(writer, 0));
            // }
            started = true;

            // Write number of len bytes.
            try!(write_byte(writer, 0x80 | (offset + 1)));
          }

          try!(write_byte(writer, len_part));
        }
      },
    }

    Ok(())
  }
}

#[cfg(test)]
use std::io::Read;

#[test]
fn tag_simple() {
  let bytes = b"\x02\x00";
  let tag = Tag {
    class: 0u8.into(),
    tagnum: 2u64.into(),
    len: Some(0u64).into(),
    constructed: false,
  };
  assert_eq!(
    Tag::decode_tag(bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    bytes
  );
}

#[test]
fn high_tag_class_1() {
  let short_bytes = b"\x41\x10";
  let long_bytes = b"\x5f\x01\x10";
  let tag = Tag {
    class: 1u8.into(),
    tagnum: 1u64.into(),
    len: Some(16u64).into(),
    constructed: false,
  };
  assert_eq!(
    Tag::decode_tag(short_bytes.bytes().by_ref()).unwrap(),
    tag
  );
  assert_eq!(
    Tag::decode_tag(long_bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    short_bytes
  );
}

#[test]
fn high_tag_class_2() {
  let bytes = b"\x5f\x21\x10";
  let tag = Tag {
    class: 1u8.into(),
    tagnum: 33u64.into(),
    len: Some(16u64).into(),
    constructed: false,
  };
  assert_eq!(
    Tag::decode_tag(bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    bytes
  );
}

#[test]
fn tag_constructed() {
  let bytes = b"\x30\x12";
  let tag = Tag {
    class: 0u8.into(),
    tagnum: 16u64.into(),
    len: Some(18u64).into(),
    constructed: true,
  };
  assert_eq!(
    Tag::decode_tag(bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    bytes
  );
}

#[test]
fn tag_indefinite() {
  let bytes = b"\x30\x80";
  let tag = Tag {
    class: 0u8.into(),
    tagnum: 16u64.into(),
    len: None.into(),
    constructed: true,
  };
  assert_eq!(
    Tag::decode_tag(bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    bytes
  );
}

#[test]
fn tag_long_len_1() {
  let long_bytes = b"\x30\x81\x11";
  let short_bytes = b"\x30\x11";
  let tag = Tag {
    class: 0u8.into(),
    tagnum: 16u64.into(),
    len: Some(17u64).into(),
    constructed: true,
  };
  assert_eq!(
    Tag::decode_tag(short_bytes.bytes().by_ref()).unwrap(),
    tag
  );
  assert_eq!(
    Tag::decode_tag(long_bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    short_bytes
  );
}

#[test]
fn tag_long_len_2() {
  let bytes = b"\x30\x81\x81";
  let tag = Tag {
    class: 0u8.into(),
    tagnum: 16u64.into(),
    len: Some(129u64).into(),
    constructed: true,
  };
  assert_eq!(
    Tag::decode_tag(bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    bytes
  );
}

#[test]
fn tag_ridiculous() {
  let bytes = b"\x7f\x81\x80\x01\x85\x80\x00\x00\x00\x01";
  let tag = Tag {
    class: 1u8.into(),
    tagnum: 0x4001u64.into(),
    len: Some(549755813889u64).into(),
    constructed: true,
  };
  assert_eq!(
    Tag::decode_tag(bytes.bytes().by_ref()).unwrap(),
    tag
  );
  let mut buf: Vec<u8> = Vec::new();
  tag.encode_tag(&mut buf).unwrap();
  assert_eq!(
    &buf,
    bytes
  );
}

#[test]
fn tag_missing_bytes() {
  let res = Tag::decode_tag(b"".bytes().by_ref());
  match res {
    Err(err::DecodeError::IO(ref err)) if err.kind() == io::ErrorKind::UnexpectedEof => {},
    _ => panic!("Expected UnexpectedEOf, got {:?}", res.unwrap_err()),
  }
}

#[test]
fn tag_missing_tag_bytes() {
  let res = Tag::decode_tag(b"\x1f".bytes().by_ref())
    .or(Tag::decode_tag(b"\x1f\x80".bytes().by_ref()))
    .or(Tag::decode_tag(b"\x1f\x80\x82".bytes().by_ref()));
  match res {
    Err(err::DecodeError::IO(ref err)) if err.kind() == io::ErrorKind::UnexpectedEof => {},
    _ => panic!("Expected UnexpectedEOf, got {:?}", res.unwrap_err()),
  }
}

#[test]
fn tag_missing_len_bytes() {
  let res = Tag::decode_tag(b"\x30".bytes().by_ref())
    .or(Tag::decode_tag(b"\x30\x81".bytes().by_ref()))
    .or(Tag::decode_tag(b"\x30\x83\x01\x03".bytes().by_ref()));
  match res {
    Err(err::DecodeError::IO(ref err)) if err.kind() == io::ErrorKind::UnexpectedEof => {},
    _ => panic!("Expected UnexpectedEOf, got {:?}", res.unwrap_err()),
  }
}