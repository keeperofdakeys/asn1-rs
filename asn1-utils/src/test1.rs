#[macro_use]
extern crate asn1_cereal;
#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use]
extern crate asn1_cereal_derive;

use log::LogLevel;

use asn1_cereal::{BerSerialize, BerDeserialize, DER, BER, BERAlt};

#[derive(Debug, Asn1Info, BerSerialize, BerDeserialize)]
pub struct Blah {
    test: u32,
}

fn main() {
  env_logger::init().unwrap();
  let bytes: &[u8] = &[
    0x30, 0x82, 0x04, 0x02, 0x30, 0x82, 0x02, 0xea, 0xa0, 0x03, 0x02, 0x01
  ];
  let mut reader = bytes.iter().map(|x| Ok(*x) as Result<u8, std::io::Error>);
  let res = Blah::deserialize_enc(DER,&mut reader);
  println!("res: {:?}", res);
}
