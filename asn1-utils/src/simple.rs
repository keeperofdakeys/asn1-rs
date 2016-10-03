// #![feature(trace_macros)]
// 
// trace_macros!(true);

#[macro_use]
extern crate asn1_cereal;
extern crate argparse;

use std::io;

use asn1_cereal::{BerSerialize, BerDeserialize, DER, BER, BERAlt};
use argparse::{ArgumentParser, StoreTrue};

fn main() {
  let opts = parse_args();

  let mut buffer: Vec<u8> = Vec::new();
  let mut output = io::BufWriter::new(io::stdout());
  let enc = BER;

  let seq = IntSequence {
    a: 4,
    b: vec![4],
    c: Some(SomeString("Hello".into()))
  };

  if opts.dump {
    let mut writer = io::BufWriter::new(&mut output);
    seq.serialize_enc(enc, &mut writer).unwrap();
    return;
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
}

ber_sequence!(
  IntSequence,
  "INTSEQ",
  a (DEFAULT 4);
  b;
  c (OPTIONAL);
);

struct ProgOpts {
  dump: bool,
}

fn parse_args() -> ProgOpts {
  let mut opts = ProgOpts {
    dump: false,
  };
  {
    let mut ap = ArgumentParser::new();
    ap.refer(&mut opts.dump)
      .add_option(&["-d", "--dump"], StoreTrue, "Dump asn.1 packet");
    ap.parse_args_or_exit();
  }
  opts
}
