extern crate asn1_rs;
extern crate argparse;

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

fn decode_stream<R: io::Read>(reader: &mut R) -> Result<(asn1::Tag, asn1::LenNum), asn1::DecodeError> {
  _decode_stream(reader, 0)
}

fn _decode_stream<R: io::Read>(reader: &mut R, indent: usize) -> Result<(asn1::Tag, asn1::LenNum), asn1::DecodeError> {
  // Get tag and decoded tag length.
  let (tag, tag_len) = try!(asn1::Tag::decode_tag(reader));

  // Print tag info.
  println!("{:>width$}TagNum: {}, Class: {}, Len: {}, Constructed: {}", "",
           tag.tagnum, tag.class, tag.len, tag.constructed, width=indent);

  // Don't decode zero length elements.
  if tag.len == asn1::Len::Def(0) {
    return Ok((tag, tag_len));
  }

  // Decoded length of this element.
  let mut decoded_len: asn1::LenNum = 0;

  // If this type is constructed, decode child element..
  if tag.constructed {
    // Loop over child elements.
    loop {
      // Decode each child element, add to decoded length.
      let (child_tag, child_len) = try!(_decode_stream(reader, indent + 1));
      decoded_len += child_len;

      // If applicable, identify end of indefinite length encoding.
      if child_tag.len == asn1::Len::Def(0) &&
         child_tag.class == asn1::Class::Universal &&
         child_tag.tagnum == 0 {
        break;
      }

      // Compare decoded length with length in tag.
      match tag.len.partial_cmp(&decoded_len) {
        // Return an error when tag length is less.
        Some(Ordering::Less) => return Err(asn1::DecodeError::GreaterLen),
        // Finish loop when equal.
        Some(Ordering::Equal) => break,
        // Keep going when less than, or indefinite length.
        _ => {},
      };
    }
  // Otherwise decode primitive value.
  } else {
    let len_num: asn1::LenNum = match tag.len {
      asn1::Len::Def(l) => l,
      asn1::Len::Indef => return Err(asn1::DecodeError::PrimIndef),
    };
    // Since we're decoding an element, we use add tag length.
    decoded_len += len_num;

    // Indent line
    print!("{:>width$}", "", width=indent);

    // Extract contents
    let mut buf = [0u8; 1];
    for _ in 0..len_num {
      let count = try!(reader.read(&mut buf));
      if count == 0 {
        return try!(Err(io::Error::new(io::ErrorKind::Other, "Read zero bytes, while expecting one")));
      }
      print!("{:x}", buf[0]);
    }
    print!("\n");
  }
  // Print end.
  println!("{:>width$}{}", "", "End.", width=indent);
  
  // Return decoded + tag_len, which is total decoded length.
  Ok((tag, decoded_len + tag_len))
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
