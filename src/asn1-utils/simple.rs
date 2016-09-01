#[macro_use]
extern crate asn1_cereal;

use std::io;

use asn1_cereal::serial;

fn main() {
  let mut buffer: Vec<u8> = Vec::new();
  // let mut buffer = io::BufWriter::new(io::stdout());
  let seq = IntSequence { a: 3, b: 4, c: "Hello".into() };
  {
    let mut writer = io::BufWriter::new(&mut buffer);
    serial::traits::Asn1Serialize::serialize_exp(&seq, &mut writer).unwrap();
  };
  println!("{:?}", buffer);
  {
    let mut reader = buffer.iter().map(|x| Ok(*x) as Result<u8, std::io::Error>);
    let seq: IntSequence = serial::traits::Asn1Deserialize::deserialize_exp(&mut reader).unwrap();
    println!("{:?}", seq);
  }
}

#[derive(Debug)]
struct IntSequence {
  a: u64,
  b: u64,
  c: String,
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
