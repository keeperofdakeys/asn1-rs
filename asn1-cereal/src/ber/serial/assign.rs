//! Macros to generate the implementation of the serialization traits for Rust
//! newtypes, as ASN.1 type assignments.
//!
//! You can either define both `asn1_info!` and `asn1_newtype!`, or all three
//! of `asn1_info!`, `asn1_newtype_serialize!` and `asn1_newtype_deserialize!`.
//!
//! IE:
//!
//! ```text
//! TYPE1 ::= SEQUENCE {
//!   a PrintableString
//! }
//!
//! TYPE1 ::= TYPE2
//! ```
//! 
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct Type1 {
//!     a: String
//!   }
//!
//!   asn1_sequence_info!(Type1, "TYPE1");
//!   asn1_sequence_serialize!(Type1, a);
//!   asn1_sequence_deserialize!(Type1, a);
//!
//!   struct Type2 (Type1);
//!   asn1_info!(Type2, 0x3, 0x1, true, "TYPE2");
//!   asn1_newtype!(Type2);
//!
//!   // OR
//!
//!   struct Type3 (Type1);
//!   asn1_info!(Type3, 0x3, 0x1, true, "TYPE3");
//!   asn1_newtype_serialize!(Type3);
//!   asn1_newtype_deserialize!(Type3);
//! }
//! ```

#[macro_export]
/// This macro is a compact way of defining both of the
/// Asn1 serialization traits - BerSerialize and BerDeserialize
/// - for a rust newtype, that represents an ASN.1 type definition.
macro_rules! asn1_newtype {
  ($rs_type:ident) => (
    asn1_newtype_serialize!($rs_type);
    asn1_newtype_deserialize!($rs_type);
  )
}

#[macro_export]
/// This macro defines the BerSerialize trait for a rust newtype.
macro_rules! asn1_newtype_serialize {
  ($rs_type:ident) => (
    impl $crate::BerSerialize for $rs_type {
      fn serialize_enc<E: $crate::ber::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        // If encoding uses implicit tag, skip our tag.
        if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit {
          return self.0.serialize_enc(e, writer);
        }

        let tag = <Self as $crate::Asn1Info>::asn1_tag();
        try!(tag.write_tag(writer));

        // If this is indefinite length and constructed, write the data directly.
        if E::len_rules() == $crate::ber::enc::LenEnc::Indefinite &&
           tag.constructed {
          try!($crate::tag::Len::Indef.write_len(writer));
          try!(self.serialize_value(e, writer));
          try!($crate::tag::Len::write_indef_end(writer));
        // Otherwise write to a Vec first, so we can write the length.
        } else {
          let mut bytes: Vec<u8> = Vec::new();
          try!(self.serialize_value(e, &mut bytes));
          try!($crate::tag::Len::write_len(Some(bytes.len() as $crate::tag::LenNum).into(), writer));
          try!(writer.write_all(&bytes));
        }

        Ok(())
      }

      fn serialize_value<E: $crate::ber::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        self.0.serialize_enc(e, writer)
      }
    }
  )
}

#[macro_export]
/// This macro defines the BerSerialize trait for a rust newtype.
macro_rules! asn1_newtype_deserialize {
  ($rs_type:ident) => (
    impl $crate::BerDeserialize for $rs_type {
      fn _deserialize_with_tag<E: $crate::ber::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len)
          -> Option<Result<Self, $crate::err::DecodeError>> {
        let my_tag = <Self as $crate::Asn1Info>::asn1_tag();

        // If we're decoding using Implicit tagging rules, throw an error if this isn't an implicit tag.
        if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit && tag == my_tag {
          return Some(Err($crate::err::DecodeError::ExplicitTag));
        }

        // If the tag doesn't match our tag, decode it as the inner type.
        if tag != my_tag {
          let res =
            $crate::BerDeserialize::deserialize_with_tag(e, reader, tag, len)
            .and_then(|s| Ok($rs_type(s)));
          Some(res)
        } else {
          None
        }
      }

      fn deserialize_value<E: $crate::ber::enc::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, _: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        Ok($rs_type(try!($crate::BerDeserialize::deserialize_enc(e, reader))))
      }
    }
  )
}
