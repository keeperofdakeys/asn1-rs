// #![feature(trace_macros)]
// trace_macros!(true);

// #![feature(plugin)]
// #![plugin(afl_plugin)]
// #![feature(proc_macro)]

extern crate asn1_cereal;
#[macro_use]
extern crate asn1_cereal_derive;
extern crate argparse;
#[macro_use]
extern crate log;
extern crate env_logger;
// extern crate afl;

use std::io;

use asn1_cereal::{BerSerialize, BerDeserialize, DER, BER, BERAlt};
use argparse::{ArgumentParser, StoreTrue};

fn main() {
  let opts = parse_args();
  env_logger::init().unwrap();

  let mut buffer: Vec<u8> = Vec::new();
  let mut output = io::BufWriter::new(io::stdout());
  let enc = BER;

  let seq = IntSequence {
    a: 3,
    b: vec![4],
    c: SomeString { a: "Hello".into() },
    d: Choice::Str("Hi".into()),
    e: C(56),
    f: None, // Some(false),
  };

  if opts.dump {
    let mut writer = io::BufWriter::new(&mut output);
    seq.serialize_enc(enc, &mut writer).unwrap();
    return;
  }

  if opts.input {
  }

  {
    let mut writer = io::BufWriter::new(&mut buffer);
    seq.serialize_enc(enc, &mut writer).unwrap();
  }
  println!("Before: {:?}", buffer);
  {
    let mut reader = buffer.iter().map(|x| Ok(*x) as Result<u8, std::io::Error>);
    let seq = IntSequence::deserialize_enc(enc, &mut reader).unwrap();
    println!("{:?}", seq);
  }
}

#[derive(Asn1Info, BerSerialize, BerDeserialize, Debug, PartialEq)]
#[asn1(tag="[PRIVATE 69]", asn1_type="SEQUENCE", log, form="alias")]
struct C(u64);

#[derive(Asn1Info, BerSerialize, BerDeserialize, Debug, PartialEq)]
#[asn1(asn1_type="Choice", log, form="choice")]
enum Choice {
  A(i32),
  Str(String),
}

#[derive(Asn1Info, BerSerialize, BerDeserialize, Debug, PartialEq)]
#[asn1(asn1_type="SOMESTRING", tag="[APPLICATION 15]", log, form="sequence")]
struct SomeString {
  a: String
}

#[derive(Asn1Info, BerSerialize, BerDeserialize, Debug, PartialEq)]
#[asn1(asn1_type="INTSEQ", tag="[APPLICATION 16]", log, form="sequence")]
struct IntSequence {
  a: u64,
  b: Vec<i32>,
  c: SomeString,
  d: Choice,
  #[asn1(optional)]
  f: Option<bool>,
  e: C,
}

struct ProgOpts {
  dump: bool,
  input: bool,
}

fn parse_args() -> ProgOpts {
  let mut opts = ProgOpts {
    dump: false,
    input: false,
  };
  {
    let mut ap = ArgumentParser::new();
    ap.refer(&mut opts.dump)
      .add_option(&["-d", "--dump"], StoreTrue, "Dump asn.1 packet");
    ap.refer(&mut opts.input)
      .add_option(&["-i", "--input"], StoreTrue, "Decode asn.1 data from stdin");
    ap.parse_args_or_exit();
  }
  opts
}
