// #![feature(trace_macros)]

#![recursion_limit = "10240"]
// trace_macros!(true);

// #![feature(plugin)]
// #![plugin(afl_plugin)]
#![feature(proc_macro)]

#[macro_use]
extern crate asn1_cereal;
#[macro_use]
extern crate asn1_cereal_derive;
extern crate argparse;
#[macro_use] extern crate log;
extern crate env_logger;
// extern crate afl;

use std::io;
use std::io::Read;

use asn1_cereal::{BerSerialize, BerDeserialize, DER, BER, BERAlt};
use argparse::{ArgumentParser, StoreTrue};

// fn main() {
//   afl::handle_bytes(|b| {
//     let a = IntSequence::deserialize_enc(enc, b.iter().map(|b| Ok(b) as std::io::Result<_>));
//     println!("{:?}", a);
//   })
// }

fn main() {
  let opts = parse_args();
  env_logger::init().unwrap();

  let mut buffer: Vec<u8> = Vec::new();
  let mut output = io::BufWriter::new(io::stdout());
  let enc = DER;

  let seq = IntSequence {
    a: 3,
    b: vec![4],
    c: Some(SomeString("Hello".into())),
    d: Choice::Str("Hi".into()),
    e: C(54),
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
// #[derive(Debug, PartialEq)]
#[asn1(tag="[PRIVATE 69]", asn1_type="SEQUENCE", log)]
struct C(u64);

// ber_alias_serialize!(C ::= u64);
// ber_alias_deserialize!(C ::= u64);

#[derive(Asn1Info, BerSerialize, BerDeserialize, Debug, PartialEq)]
#[asn1(asn1_type="Choice", log)]
enum Choice {
  A(i32),
  Str(String),
}

#[derive(Debug, PartialEq)]
struct SomeString(String);

ber_alias!(
  SomeString ::= [PRIVATE 1] String,
  "SOMESTRING"
);

#[derive(Debug, PartialEq)]
struct IntSequence {
  a: u64,
  b: Vec<i32>,
  c: Option<SomeString>,
  d: Choice,
  e: C,
}

ber_sequence!(
  IntSequence,
  "INTSEQ",
  a (DEFAULT 4);
  b;
  c (OPTIONAL);
  d;
  e;
);

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
