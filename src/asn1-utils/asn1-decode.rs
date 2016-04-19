extern crate asn1_rs;
extern crate argparse;

use asn1_rs::asn1;
use asn1_rs::decode;

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
  let decoder = decode::StreamDecoder(reader, StreamDumper::new());
  decoder.decode();
}

struct StreamDumper {
  indent: u32,
}

impl StreamDumper {
  fn new() -> Self {
    StreamDumper { indent: 0 }
  }
}

impl StreamDecodee for StreamDumper {
  fn start_element(&mut self, tag: asn1::Tag) -> decode::ParseResult {
    // Print tag info.
    println!("{:>width$}TagNum: {}, Class: {}, Len: {}, Constructed: {}", "",
             tag.tagnum, tag.class, tag.len, tag.constructed, width=self.indent);
    self.indent += 1;
    decode::ParseResult::Ok
  }

  fn end_element(&mut self) -> decode::ParseResult {
    self.indent -= 1;
    println!("{:>width$}{}", "", "End.", width=self.indent);
    decode::ParseResult::Ok
  }

  fn primitive(&mut self, reader: asn1::ByteReader, len: asn1::LenNum) -> decode::ParseResult {
    // Indent line
    print!("{:>width$}", "", width=self.indent);

    // Extract contents
    for _ in 0..len_num {
      let byte = match reader.next() {
        Some(b) => try!(b),
        None => return decode::ParseResult::EOF,
      };
      print!("{:x}", byte);
    }
    print!("\n");
    decode::ParseResult::Ok
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
