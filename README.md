# asn1-cereal
A collection of encoders and decoders for BER, DER and ASN.1.

The grains of this library are a collection of traits and macros, that
allow serialization and deserialization of rust types to and from ASN.1.
For example, the following will encode this struct as an ASN.1 sequence.

    #[macro_use]
    extern crate asn1_cereal;

    struct ShortSequence {
      z: u64,
      y: u32,
    }
    
    asn1_sequence!(
      ShortSequence,
      "SHORT_SEQUENCE",
      z,
      y
    );

The asn1\_sequence! macro generates the implementation for each trait,
which in turn call the serialize/deserialize method for each struct
field. For custom primitive ASN.1 elements, you may need to provide
your own implementation.
