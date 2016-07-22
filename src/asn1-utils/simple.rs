extern crate asn1_cereal;

use std::Io;

use asn1_cereal::serial::{Asn1Serialize, Asn1Deserialize};

fn main() {
  let buffer = String::new();
  {
    let writer = io::BufWriter::new(buffer);
    Asn1Serialize::deserialize(3u64, writer).unwrap();
  };
}

struct IntSequence {
  a: u64,
  b: u64,
}

asn1_cerealize!(
  IntSequence, 'SEQUENCE',
  a,
  b,
);
