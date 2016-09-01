//! The macros in this module can implement the serialisation traits for Sequences and
//! Sets.
//!
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct SomeSequence {
//!     a: u64,
//!     b: u32,
//!     c: String,
//!   }
//!
//!   asn1_sequence_info!(SomeSequence, "SOME_SEQUENCE");
//!   asn1_sequence_serialize!(SomeSequence, a, b, c);
//!   asn1_sequence_deserialize!(SomeSequence, a, b, c);
//! }
//! ```
//!
//! Note that these macros can also be used for a SET, but not a
//! SEQUENECE OF, or SET OF. (These are more appropriate as Vec
//! or Set).

#[macro_export]
/// This macro is a compact way of defining asn1_info for a type.
macro_rules! asn1_info {
  ($rs_type:ty, $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_tag() -> $crate::tag::Tag {
        $crate::tag::Tag {
          class: ($class as u8).into(),
          tagnum: $tagnum as $crate::tag::TagNum,
          constructed: $constructed,

        }
      }

      fn asn1_type() -> $crate::tag::Type {
        $crate::tag::Type::from($asn1_ty)
      }
    }
  )
}

#[macro_export]
/// This macro is a compact way of defining the Asn1Info trait implementation
/// for a struct that represents an ASN.1 structure. If a custom class or tag
/// number is required, asn1_info! should be used instead.
macro_rules! asn1_sequence_info {
  ($rs_type:ty, $asn1_ty:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_tag() -> $crate::tag::Tag {
        $crate::tag::Tag {
          class: $crate::tag::Class::Universal,
          tagnum: (0x10 as u8).into(),
          constructed: true,
        }
      }

      fn asn1_type() -> $crate::tag::Type {
        $asn1_ty.into()
      }
    }
  )
}

#[macro_export]
/// This macro will generate an Asn1Serialize implementation for a struct
/// that represents an ASN.1 structure.
///
/// Note that struct fields need to be manually specificied, and serialization
/// will occur in that order. If you don't list a field, it won't get
/// serialized. Each of these fields need to implement Asn1Serialize,
/// otherwise compilation will fail.
///
/// If some form of procedural macros are eventually stabilised,
/// listing the fields in the macro might not be required.
macro_rules! asn1_sequence_serialize {
  ($rs_type:ty, $($item:ident),*) => (
    impl $crate::serial::traits::Asn1Serialize for $rs_type {
      fn serialize_imp<W: std::io::Write>(&self, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        let mut bytes = Vec::new();
        let mut count: u64 = 0;
        // For each declared sequence member, serialize it onto the stream.
        $(
          count += 1;
          try!(
            $crate::serial::traits::Asn1Serialize::serialize_exp(&self.$item, &mut bytes)
          );
          let tag = $crate::tag::TagLen {
            tag: $crate::tag::Tag {
              class: $crate::tag::Class::ContextSpecific,
              tagnum: count.into(),
              constructed: true,
            },
            len: Some(bytes.len() as $crate::tag::LenNum).into(),
          };
          try!(tag.write_taglen(writer));
          try!(writer.write(&mut bytes));

          bytes.clear();
        )*
        Ok(())
      }
    }
  )
}

#[macro_export]
/// This macro will generate an Asn1Deserialize implementation for a struct
/// that represents an ASN.1 structure.
///
/// Note that all struct fields need to be manually specificied, and
/// deserialization will occur in that order. Unlike the deserialize macro,
/// all fields must be specified. Each of these fields need to implement
/// Asn1Deserialize, otherwise compilation will fail.
///
/// If some form of procedural macros are eventually stabilised,
/// listing the fields in the macro might not be required.
macro_rules! asn1_sequence_deserialize {
  ($rs_type:ident, $($item:ident),*) => (
    impl $crate::serial::traits::Asn1Deserialize for $rs_type {
      fn deserialize_imp<I: Iterator<Item=std::io::Result<u8>>>(reader: &mut I, _: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        Ok( $rs_type { $(
          $item: {
            let tag = try!($crate::tag::TagLen::read_taglen(reader));
            let _ = tag;
            try!($crate::serial::traits::Asn1Deserialize::deserialize_exp(reader))
          },
        )* })
      }
    }
  )
}
