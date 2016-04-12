extern crate asn1_rs;
extern crate argparse;

use asn1_rs::decode::{decode_tag, Asn1ReadError};
use asn1_rs::asn1;

use std::io;
use std::fs;
use std::path::Path;
use std::cmp::Ordering;
use argparse::{ArgumentParser, StoreTrue, StoreOption};

fn main() {
  let opts = parse_args();

  let path = Path::new(opts.file.as_ref().unwrap());
  if !path.is_file() {
    panic!("Supplied file does not exist");
  }

  // Create a buffered reader from the file.
  let mut reader = io::BufReader::new(fs::File::open(path).unwrap());
  decode_stream(&mut reader).unwrap();
}

fn decode_stream<R: io::Read>(reader: &mut R) -> Result<(asn1::Asn1Tag, asn1::Asn1LenNum), Asn1ReadError> {
  _decode_stream(reader, 0)
}

fn _decode_stream<R: io::Read>(reader: &mut R, indent: usize) -> Result<(asn1::Asn1Tag, asn1::Asn1LenNum), Asn1ReadError> {
  let tag = try!(decode_tag(reader));

  let mut decoded_len: asn1::Asn1LenNum = 0;
  println!("{:>width$}TagNum: {}, Class: {}, Len: {}", "", tag.tagnum, tag.class, tag.len, width=indent);
  // If this type is structured (SEQUENCE or SET), decode child elements.
  if tag.is_structured() {
    loop {
      let (child_tag, child_len) = try!(_decode_stream(reader, indent + 1));
      decoded_len += child_len;

      // Identify end of indefinite length.
      if child_tag.len == asn1::Asn1Len::Def(0) &&
         child_tag.class == asn1::Asn1Class::Universal &&
         child_tag.tagnum == 0 {
        break;
      }

      // Compare deoded length with tag length.
      match tag.len.partial_cmp(&child_len) {
        // Return an error when decoded length is greater.
        Some(Ordering::Less) => return Err(Asn1ReadError::GreaterLen),
        // Finish loop when equal.
        Some(Ordering::Equal) => break,
        // Keep going when less than, or indefinite length.
        _ => continue,
      };
    }
  } else {
    let len_num: asn1::Asn1LenNum = match tag.len {
      asn1::Asn1Len::Def(l) => l,
      asn1::Asn1Len::Indef => 
        panic!("I don't know how to handle unstructured, indefinite length encoding"),
    };

    print!("{:>width$}", "", width=indent);
    let mut buf = [0u8; 1];
    // FIXME: This assumes definite length
    decoded_len += len_num;
    for _ in 0..len_num {
      let count = try!(reader.read(&mut buf));
      if count == 0 {
        return try!(Err(io::Error::new(io::ErrorKind::Other, "Read zero bytes, while expecting one")));
      }
      print!("{:x}", buf[0]);
    }
    print!("\n");
  }
  Ok((tag, decoded_len))
}

struct ProgOpts {
  file: Option<String>,
  verbose: bool,
}

fn parse_args() -> ProgOpts {
  let mut opts = ProgOpts {
    file: None,
    verbose: false,
  };

  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Decode ASN.1 files");
    ap.refer(&mut opts.verbose)
      .add_option(&["-v", "--verbose"], StoreTrue, "Verbose output");
    ap.refer(&mut opts.file)
      .add_argument("file", StoreOption, "ASN.1 file to decode");
    ap.parse_args_or_exit();
  }
  opts
}
