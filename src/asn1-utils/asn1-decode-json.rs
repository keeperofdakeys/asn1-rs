extern crate asn1_rs;
extern crate argparse;

use asn1_rs::asn1;
use asn1_rs::decode;

use std::io;
use std::io::Read;
use std::fs;
use std::path::Path;
use argparse::{ArgumentParser, StoreTrue, StoreOption};

fn main() {
  let opts = parse_args();

  let path = Path::new(opts.file.as_ref().unwrap());
  if !path.is_file() {
    panic!("Supplied file does not exist");
  }

  // Create a buffered reader from the file.
  let reader = io::BufReader::new(fs::File::open(path).unwrap()).bytes();
  let mut decoder = decode::StreamDecoder::new(reader, StreamDumper::new());
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

impl decode::StreamDecodee for StreamDumper {
  fn start_element(&mut self, tag: asn1::Tag) -> decode::ParseResult {
    // Print: {{ tag: 
    print!("{:>width$}{{\n{:>width2$}\"tag\": ", "", "", width=self.indent*2, width2=(self.indent+1)*2);
    // Print: { .. tag details .. }
    print!("{{ \"num\": {}, \"class\": \"{}\", \"constructed\": {}",
      tag.tagnum, tag.class, tag.constructed
    ); 
    match tag.len {
      asn1::Len::Def(ref l) => print!(", \"len\": {}", l),
      _ => {},
    }
    print!(" }},\n");

    self.indent += 1;

    if tag.constructed {
      println!("{:>width$}\"elements\": [", "", width=self.indent*2);
      self.indent += 1;
    }

    decode::ParseResult::Ok
  }

  fn end_element(&mut self, tag: asn1::Tag) -> decode::ParseResult {
    self.indent -= 1;

    if tag.constructed {
      println!("{:>width$}]", "", width=self.indent*2);
      self.indent -= 1;
    }
    println!("{:>width$}}},", "", width=self.indent*2);
    decode::ParseResult::Ok
  }

  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut asn1::ByteReader<I>, len: asn1::LenNum) ->
    decode::ParseResult {
    // Indent line
    print!("{:>width$}\"bytes\": \"", "", width=self.indent*2);

    // Extract contents
    for _ in 0..len {
      let byte = match reader.read() {
        Ok(b) => b,
        Err(e) => return e.into(),
      };
      print!("{:x}", byte);
    }
    println!("\"");
    decode::ParseResult::Ok
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
