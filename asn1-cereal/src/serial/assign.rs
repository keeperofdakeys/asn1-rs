//! The macros in this module can implement the serialisation traits for a
//! simple ASN.1 type definition/assignment, respresnted by a Rust newtype
//! (anonymous struct with a single field).
//!
//! You can either define both asn1_info! and asn1_newtype!, or all three
//! of asn1_info!, asn1_newtype_serialize! and asn1_newtype_deserialize!.
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
//!   asn1_info!(Type3, 0x3, 0x1, true, "TYPE2");
//!   asn1_newtype_serialize!(Type3);
//!   asn1_newtype_deserialize!(Type3);
//! }
//! ```

#[macro_export]
/// This macro is a compact way of defining both of the
/// Asn1 serialization traits - Asn1Serialize and Asn1Deserialize
/// - for a rust newtype, that represents an ASN.1 type definition.
macro_rules! asn1_newtype {
  ($rs_type:ident) => (
    asn1_newtype_serialize!($rs_type);
    asn1_newtype_deserialize!($rs_type);
  )
}

#[macro_export]
/// This macro defines the Asn1Serialize trait for a rust newtype.
macro_rules! asn1_newtype_serialize {
  ($rs_type:ident) => (
    impl $crate::serial::Asn1Serialize for $rs_type {
      fn serialize_enc<E: $crate::enc::Asn1EncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        // If encoding uses implicit tag, skip our tag.
        if E::tag_rules() == $crate::enc::TagEnc::Implicit {
          return self.0.serialize_enc(e, writer);
        }

        let mut bytes: Vec<u8> = Vec::new();
        try!(self.serialize_bytes(e, &mut bytes));

        try!(<Self as $crate::serial::Asn1Info>::asn1_tag().write_tag(writer));
        try!($crate::tag::Len::write_len(
          Some(bytes.len() as $crate::tag::LenNum).into(),
          writer
        ));
        try!(writer.write_all(&bytes));

        Ok(())
      }

      fn serialize_bytes<E: $crate::enc::Asn1EncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        self.0.serialize_enc(e, writer)
      }
    }
  )
}

#[macro_export]
/// This macro defines the Asn1Serialize trait for a rust newtype.
macro_rules! asn1_newtype_deserialize {
  ($rs_type:ident) => (
    impl $crate::serial::Asn1Deserialize for $rs_type {
      /// Reimplement this function to handle implicit tagging.
      fn deserialize_enc_tag<E: $crate::enc::Asn1EncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag)
          -> Result<Self, $crate::err::DecodeError> {
        let my_tag = <Self as $crate::serial::Asn1Info>::asn1_tag();

        // If we're decoding using Implicit tagging rules, throw an error if this isn't an implicit tag.
        if E::tag_rules() == $crate::enc::TagEnc::Implicit && tag == my_tag {
          return Err($crate::err::DecodeError::ExplicitTag);
        }

        // If the tag doesn't match our tag, decode it as the inner type.
        if tag != my_tag {
          return Ok($rs_type(try!(
            $crate::serial::Asn1Deserialize::deserialize_enc_tag(e, reader, tag)
          )));
        }

        let len = try!($crate::tag::Len::read_len(reader));

        // If element is primitive, and length is indefinite, we can't decode it.
        if !tag.constructed && len == $crate::tag::Len::Indef {
          Err($crate::err::DecodeError::PrimIndef)
        } else {
          Self::deserialize_bytes(e, reader, len.as_num())
        }
      }
      fn deserialize_bytes<E: $crate::enc::Asn1EncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, _: Option<$crate::tag::LenNum>) -> Result<Self, $crate::err::DecodeError> {
        Ok($rs_type(try!($crate::serial::Asn1Deserialize::deserialize_enc(e, reader))))
      }
    }
  )
}