extern crate asn1_rs;
extern crate argparse;
extern crate serde;
extern crate serde_json;

use asn1_rs::asn1;
use asn1_rs::decode;

use std::io;
use std::io::Read;
use std::fs;
use std::path::Path;
use std::collections::BTreeMap;
use argparse::{ArgumentParser, StoreTrue, StoreOption};
use serde_json::value::Value;
use serde_json::ser::to_string_pretty;

fn main() {
  let opts = parse_args();

  let path = Path::new(opts.file.as_ref().unwrap());
  if !path.is_file() {
    panic!("Supplied file does not exist");
  }

  // Create a buffered reader from the file.
  let reader = io::BufReader::new(fs::File::open(path).unwrap()).bytes();
  let mut dumper = StreamDumper::new();
  {
    let mut decoder = decode::StreamDecoder::new(reader, &mut dumper);
    decoder.decode().unwrap();
  }
  println!("{}", to_string_pretty(&dumper.stack.last().unwrap().last().unwrap()).unwrap());
}

struct StreamDumper {
  pub stack: Vec<Vec<Value>>,
  elem: Option<Value>,
}

impl StreamDumper {
  fn new() -> Self {
    StreamDumper {
      stack: vec![Vec::new()],
      elem: None,
    }
  }
}

impl decode::StreamDecodee for StreamDumper {
  fn start_element(&mut self, tag: asn1::Tag) -> decode::ParseResult {
    if tag.constructed {
      self.stack.push(Vec::new());
    }

    decode::ParseResult::Ok
  }

  fn end_element(&mut self, tag: asn1::Tag) -> decode::ParseResult {
    let mut tag_map = BTreeMap::new();
    let mut map = BTreeMap::new();
    tag_map.insert(
      "class",
      match tag.class {
        asn1::Class::Application => "application",
        asn1::Class::Universal => "universal",
        asn1::Class::Private => "private",
        asn1::Class::ContextSpecific => "context",
      }.to_owned(),
    );
    if let asn1::Len::Def(ref l) = tag.len {
      tag_map.insert("length", l.to_string());
    }
    tag_map.insert("num", tag.tagnum.to_string());
    tag_map.insert("constructed", tag.constructed.to_string());
    map.insert("tag", serde_json::to_value(&tag_map));
    if tag.constructed {
      map.insert("elements", serde_json::to_value(&self.stack.pop().unwrap()));
    } else {
      if self.elem.is_some() {
        map.insert("bytes", self.elem.as_ref().unwrap().clone());
        self.elem = None;
      } else {
        panic!("No primitive element found");
      }
    }
    self.stack.last_mut().unwrap().push(serde_json::to_value(&map));

    decode::ParseResult::Ok
  }

  fn primitive<I: Iterator<Item=io::Result<u8>>>(&mut self, reader: &mut asn1::ByteReader<I>, len: asn1::LenNum) ->
    decode::ParseResult {
    if self.elem.is_some() {
      panic!("elem should not be defined already!");
    }
    let mut bytes = String::new();

    // Extract contents
    for _ in 0..len {
      let byte = match reader.read() {
        Ok(b) => b,
        Err(e) => return e.into(),
      };
      bytes.push_str(&format!("{:x}", byte));
    }
    self.elem = Some(serde_json::to_value(&bytes));

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
