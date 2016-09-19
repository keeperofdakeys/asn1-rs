# asn1-cereal
[![Crates.io](https://img.shields.io/crates/v/asn1-cereal.svg?maxAge=2592000)](https://crates.io/crates/asn1-cereal)

A collection of encoders and decoders for BER, DER and ASN.1.

The grains of this library are a collection of traits and macros that
allow serialization and deserialization of rust types to and from ASN.1.
For example, the following will encode this struct as a DER sequence.

```rust
#[macro_use]
extern crate asn1_cereal;

struct ShortSequence {
  z: u64,
  y: u32,
}

ber1_sequence!(
  ShortSequence,
  "SHORT_SEQUENCE",
  z,
  y
);

use asn1_cereal::BerSerialize;

let data = ShortSequence{ z: 1, y: 2 };
let mut bytes: Vec<u8> = Vec::new();
BerSerialize::serialize(&data, &mut bytes).unwrap();

```

The ber1\_sequence! macro generates the trait implementation for each type,
which in turn calls the serialize/deserialize method for each struct
field.

A more complex example that implements the ASN.1 provided is the following.
```ASN1
SomeInt ::= [APPLICATRION 1] INTEGER

OtherSeq ::= SEQUENCE {
  someInt    SomeInt,
  thingySeq  SEQUENCE OF INTEGER
}
```

```rust
#[macro_use]
extern crate asn1_cereal;

struct SomeInt(u64);

// This creates the implementation of ASN1Info, so asn1-cereal knows about
// the tag for this type.
asn1_info!(
  SomeInt,
  asn1_cereal::tag::Class::Application,
  1,
  true, // This indicates that the SomeInt type is constructed (contains ASN.1).
  "SomeInt"
);

// This implements BerSerialize and BerDeserialize for this type.
// The implementation of these traits for u64 is built-in to asn1-cereal.
ber_newtype(SomeInt);

struct OtherSeq {
  someInt: SomeInt,
  // The implementation of SEQUENCE OF is already defined for Vec.
  thingySeq: Vec<u32>,
}

// This immplements the traits for the struct, as a sequence.
ber\_sequence!(
  OtherSeq,
  "OtherSeq",
  someInt,
  thingySeq
);

let data = OtherSeq{ z: SomeInt(5), thingySeq: Vec< };
let mut bytes: Vec<u8> = Vec::new();

BerSerialize::serialize(&data, &mut bytes).unwrap();
```

For custom primitive ASN.1 elements, you may need to provide
your own implementations.
