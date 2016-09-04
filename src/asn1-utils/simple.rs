#[macro_use]
extern crate asn1_cereal;

use std::io;

use asn1_cereal::serial::{Asn1Serialize, Asn1Deserialize};
use asn1_cereal::enc::{DER, BER};
use asn1_cereal::tag::Class;

fn main() {
  let mut buffer: Vec<u8> = Vec::new();
  // let mut buffer = io::BufWriter::new(io::stdout());
  let seq = IntSequence { a: 3, b: 4, c: SomeString("Hello".into()) };
  {
    let mut writer = io::BufWriter::new(&mut buffer);
    seq.serialize_enc(BER, &mut writer).unwrap();
  };
  println!("{:?}", buffer);
  {
    let mut reader = buffer.iter().map(|x| Ok(*x) as Result<u8, std::io::Error>);
    let seq = IntSequence::deserialize_enc(BER, &mut reader, None).unwrap();
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

asn1_newtype_serialize!(SomeString);
asn1_newtype_deserialize!(SomeString);

#[derive(Debug)]
struct IntSequence {
  a: u64,
  b: u64,
  c: SomeString,
}

asn1_sequence_info!(
  IntSequence,
  "INTSEQ"
);

asn1_sequence_serialize!(
  IntSequence,
  a,
  b,
  c
);

asn1_sequence_deserialize!(
  IntSequence,
  a,
  b,
  c
);
