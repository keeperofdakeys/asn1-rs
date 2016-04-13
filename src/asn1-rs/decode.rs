use std::io;
use asn1;

/// Decode an ASN.1 tag from a stream.
pub fn decode_tag<R: io::Read>(reader: &mut R) -> Result<asn1::Asn1Tag, Asn1ReadError> {
  let mut bytes = ByteReader::new(reader);

  // Decode tag byte, which includes class, constructed flag, and tag number.
  let tag_byte = try!(bytes.read());
  let class_num = (tag_byte & 0xc0) >> 6;
  let constructed = tag_byte & 0x40 == 0x40;
  let mut tag_num = (tag_byte & 0x1f) as asn1::Asn1TagNum;
  // If tag is 0x1F, use extended decode format.
  if tag_num == 0x1F {
    loop {
      // Incrementatlly read bytes, adding base-128 to tag.
      let tag_more = try!(bytes.read());
      tag_num = (tag_num << 7) + (tag_more & 0x7f) as asn1::Asn1TagNum;
      // Stop looping when 0x80 bit is set.
      if tag_more & 0x80 == 0x80 {
        break;
      }
    }
  }

  // Decode len byte.
  let len_byte = try!(bytes.read());
  let len = match len_byte {
    // When byte is 0x80, this is the start of indefinite length encoding.
    0x80 => asn1::Asn1Len::Indef,
    // If 0x80 is set, then other bits indicate the number of len bytes.
    l => if (l & 0x80) == 0x80 {
        let mut len: asn1::Asn1LenNum = 0;
        let byte_count = l & 0x7f;
        // Loop through number of len bytes.
        for _ in 0..byte_count {
          let len_more = try!(bytes.read());
          // Add up each byte base-256.
          len = (len << 8) + len_more as asn1::Asn1TagNum;
        }
        asn1::Asn1Len::Def(len)
      // If 0x80 bit is not set, just decode the value.
      } else {
        asn1::Asn1Len::Def(l as asn1::Asn1LenNum)
      },
  };

  Ok(asn1::Asn1Tag {
    class: asn1::Asn1Class::from(class_num),
    tagnum: tag_num,
    len: len,
    constructed: constructed,
  })
}

/// A reader to easily read a byte from a reader.
struct ByteReader<'a, R: io::Read + 'a> {
  reader: &'a mut R,
}

impl<'a, R: io::Read + 'a> ByteReader<'a, R> {
  fn new(reader: &'a mut R) -> ByteReader<'a, R> {
    ByteReader { reader: reader }
  }

  /// Read a byte from a reader.
  fn read(&mut self) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    // FIXME: Should retry on the Interrupted Error, and perhaps another error.
    match try!(self.reader.read(&mut buf)) {
      0 => Err(io::Error::new(io::ErrorKind::Other, "Read zero bytes")),
      1 => Ok(buf[0]),
      _ => Err(io::Error::new(io::ErrorKind::Other, "Read more than one byte")),
    }
  }
}

#[derive(Debug)]
/// Errors that can occur reading an ASN.1 element.
pub enum Asn1ReadError {
  /// Generic IO Error.
  IO(io::Error),
  /// Decoded child element(s) had greater length than parent's len.
  GreaterLen,
}

impl From<io::Error> for Asn1ReadError {
  fn from(err: io::Error) -> Self {
    Asn1ReadError::IO(err)
  }
}
