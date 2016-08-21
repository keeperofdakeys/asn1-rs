#[macro_use]
extern crate asn1_cereal;

use std::io;

use asn1_cereal::err;
use asn1_cereal::byte;
use asn1_cereal::tag;
use asn1_cereal::serial;

fn main() {
  let mut buffer: Vec<u8> = Vec::new();
  {
    let mut writer = io::BufWriter::new(&mut buffer);
    serial::traits::Asn1Serialize::serialize(&3u64, &mut writer).unwrap();
  };
  println!("{:?}", buffer);
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
