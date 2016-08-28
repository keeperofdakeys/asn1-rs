#[macro_use]
extern crate asn1_cereal;

use std::io;

use asn1_cereal::err;
use asn1_cereal::tag;
use asn1_cereal::serial;

fn main() {
  // let mut buffer: Vec<u8> = Vec::new();
  let mut buffer = io::BufWriter::new(io::stdout());
  let seq = IntSequence { a: 3, b: 4 };
  {
    let mut writer = io::BufWriter::new(&mut buffer);
    serial::traits::Asn1Serialize::serialize_exp(&seq, &mut writer).unwrap();
  };
}

struct IntSequence {
  a: u64,
  b: u64,
}

asn1_sequence_info!(
  IntSequence,
  "INTSEQ"
);

asn1_sequence_serialize!(
  IntSequence,
  a,
  b
);
