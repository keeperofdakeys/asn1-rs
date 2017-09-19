# asn1-cereal
[![Crates.io](https://img.shields.io/crates/v/asn1-cereal.svg?maxAge=2592000)](https://crates.io/crates/asn1-cereal)

A collection of encoders and decoders for BER, DER and ASN.1.

The grains of this library are a collection of traits and macros that
allow serialization and deserialization of rust types to and from ASN.1.
For example, the following will encode this struct as a DER sequence.

```rust
#[macro_use]
extern crate asn1_cereal_derive;
extern crate asn1_cereal;
fn main() {
  #[derive(Asn1Info, BerSerialize, BerDeserialize)]
  #[asn1(asn1_type="SHORT_SEQUENCE", tag="[APPLICATION 8]")]
  struct ShortSequence {
    z: u64,
    y: u32,
  }

  use asn1_cereal::BerSerialize;

  let data = ShortSequence{ z: 1, y: 2 };
  let mut bytes: Vec<u8> = Vec::new();
  BerSerialize::serialize(&data, &mut bytes).unwrap();
}
```
