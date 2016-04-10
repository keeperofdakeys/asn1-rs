use std::io;

/// Decode an ASN.1 tag from a stream.
fn decode_tag<R: io::Read>(reader: R) -> Result<asn1::Asn1Tag, Asn1ReadError> {
  let bytes = ByteReader::new(reader);

  let tag_byte = try!(bytes.read());
  let class_num = (tag_byte & 0xc0) >> 6;
  let constructed = tag_byte & 0x40 != 0x00;
  let mut tag_num: asn1::Asn1TagNum = tag_byte & 0x1f;
  if tag_num == 0x1F {
    while let tag_more = try!(bytes.read()) &&
          tag_more & 0x80 == 0x80 {
      tag_num += tag_more & 0x7f;
    }
  }
  let mut len_byte = try!(bytes.read());
  let mut len: asn1::Asn1LenNum = len_byte;
  if (len_byte & 0x80) == 0x00 {
    len = len_byte & 0x7
  }


  Ok asn1::Asn1Tag {
    class: asn1::Asn1Clas::from(class_num),
    tagnum: tag_num,
    len: asn1::Asn1Len::from(len),
    constructed: constructed,
  }
}

/// A reader to easily read a byte from a reader.
struct ByteReader<'a, R: io::Read> {
  reader: &'a mut R,
}

impl<'a, R: io::Read> ByteReader<'a> {
  fn new(reader: &'a mut R) -> ByteReader<'a> {
    ByteReader { reader: reader }
  }

  /// Read a byte from a reader.
  fn read<R: io::Read>(reader: R) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    // FIXME: Should retry on the Interrupted Error, and perhaps another error.
    match try!(reader.read(&mut buf)) {
      0 => Err(io::Error::new(io::ErrorKind::Other, "Read zero bytes");
      1 => {},
      _ => Err(io::Error::new(io::ErrorKind::Other, "Read more than one byte");
    }
    buf[0]
  }
}

/// Errors that can occur reading an ASN.1 element.
enum Asn1ReadError {
  /// Generic IO Error.
  IO(io::Error),
}

impl From<io::Error> for Asn1ReadError {
  fn from(err: io::Error) -> Self {
    Asn1ReadError::IO(err)
  }
}
