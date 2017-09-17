extern crate asn1_cereal;
extern crate argparse;

use asn1_cereal::{tag, byte};
use asn1_cereal::ber::stream;

use std::io;
use std::io::Read;
use std::fs;
use std::path::Path;
use argparse::{ArgumentParser, StoreTrue, StoreOption};

fn main() {
  let opts = parse_args();

  let path = Path::new(opts.file.as_ref().unwrap());

  // Create a buffered reader from the file.
  let reader = io::BufReader::new(fs::File::open(path).unwrap()).bytes();
  let mut dumper = StreamDumper::new();
  let mut decoder = stream::StreamDecoder::new(reader, &mut dumper);
  decoder.decode().unwrap();
}

struct StreamDumper {
  indent: usize,
}

impl StreamDumper {
  fn new() -> Self {
    StreamDumper { indent: 0 }
  }
}

impl stream::StreamDecodee for StreamDumper {
  fn start_element(&mut self, tag: tag::Tag, len: tag::Len) -> stream::ParseResult {
    // Print tag info.
    println!("{:>width$}TagNum: {}, Class: {}, Len: {}, Constructed: {}", "",
             tag.tagnum, tag.class, len, tag.constructed, width=self.indent);
    self.indent += 1;
    stream::ParseResult::Ok
  }

  fn end_element(&mut self, _: tag::Tag, _: tag::Len) -> stream::ParseResult {
    self.indent -= 1;
    println!("{:>width$}{}", "", "End.", width=self.indent);
    stream::ParseResult::Ok
  }

  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut byte::ByteReader<I>, len: tag::LenNum) ->
    stream::ParseResult {
    // Indent line
    print!("{:>width$}", "", width=self.indent);

    // Extract contents
    for _ in 0..len {
      let byte = match reader.read() {
        Ok(b) => b,
        Err(e) => return e.into(),
      };
      print!("{:x}", byte);
    }
    print!("\n");
    stream::ParseResult::Ok
  }
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
