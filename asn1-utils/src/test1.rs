#[macro_use]
extern crate asn1_cereal;

use asn1_cereal::{BerSerialize, BerDeserialize, DER, BER, BERAlt};

#[derive(Debug)]
pub struct Blah {
    test: u32,
}

ber_sequence!(
    Blah,
    "Blah",
    test;
);

fn main() {
  let bytes: &[u8] = &[
    0x30, 0x82, 0x04, 0x02, 0x30, 0x82, 0x02, 0xea, 0xa0, 0x03, 0x02, 0x01
  ];
  let mut reader = bytes.iter().map(|x| Ok(*x) as Result<u8, std::io::Error>);
  let res = Blah::deserialize_enc(DER,&mut reader);
  println!("res: {:?}", res);
}
