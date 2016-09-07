#[macro_use]
extern crate asn1_cereal;
extern crate argparse;

use std::io;

use asn1_cereal::serial::{Asn1Serialize, Asn1Deserialize};
use asn1_cereal::enc::{DER, BER, BERAlt};
use asn1_cereal::tag::Class;
use argparse::{ArgumentParser, StoreTrue};

fn main() {
  let opts = parse_args();

  let mut buffer: Vec<u8> = Vec::new();
  let mut output = io::BufWriter::new(io::stdout());

  let seq = IntSequence { a: 3, b: 4, c: SomeString("Hello".into()) };

  if opts.dump {
    let mut writer = io::BufWriter::new(&mut output);
    seq.serialize_enc(BERAlt, &mut writer).unwrap();
    return;
  }
  {
    let mut writer = io::BufWriter::new(&mut buffer);
    seq.serialize_enc(BERAlt, &mut writer).unwrap();
  }
  println!("{:?}", buffer);
  {
    let mut reader = buffer.iter().map(|x| Ok(*x) as Result<u8, std::io::Error>);
    let seq = IntSequence::deserialize_enc(BERAlt, &mut reader).unwrap();
    println!("{:?}", seq);
  }
}

#[derive(Debug)]
struct SomeString(String);

asn1_info!(
  SomeString,
  Class::Private,
  1,
  true,
  "SOMESTRING"
);

asn1_newtype!(SomeString);

#[derive(Debug)]
struct IntSequence {
  a: u64,
  b: u64,
  c: SomeString,
}

asn1_sequence!(
  IntSequence,
  "INTSEQ",
  a,
  b,
  c
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
