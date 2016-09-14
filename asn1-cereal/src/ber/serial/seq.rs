//! Macros to generate the implementation of the serialization traits for Rust
//! structs, as ASN.1 sequences.
//!
//! You can either use the shortcut `asn1_sequence!` macro, or each of
//! `asn1_sequence_info!`, `asn1_sequence_serialize!` and `asn1_sequence_deserialize!`.
//!
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct ShortSequence {
//!     z: u64,
//!     y: u32,
//!   }
//!
//!   asn1_sequence!(
//!     ShortSequence,
//!     "SHORT_SEQUENCE",
//!     z,
//!     y
//!   );
//!
//!   // OR
//!
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
//! Note that these macros won't handle SEQUENCE OF.

#[macro_export]
/// This macro is a compact way of defining all three of the
/// Asn1 traits - Asn1Info, BerSerialize and BerDeserialize -
/// for a rust struct, that represents an ASN.1 sequence.
///
/// Note that the order the fields are placed in will affect the order
/// that they are encoded to, and decoded from ASN.1. If some form of
/// procedural macros are eventually stabilised, listing the fields
/// in the macro might no longer be required.
macro_rules! asn1_sequence {
  ($rs_type:ident, $asn1_ty:expr, $($item:ident),*) => (
    asn1_sequence_info!($rs_type, $asn1_ty);
    asn1_sequence_serialize!($rs_type, $($item),*);
    asn1_sequence_deserialize!($rs_type, $($item),*);
  )
}

#[macro_export]
/// This macro defines the Asn1Info trait for a rust struct. This allows the other
/// traits to get information about this type. If you need to provide a custom
/// class or tag, consider using the asn1_info! macro.
macro_rules! asn1_sequence_info {
  ($rs_type:ident, $asn1_ty:expr) => (
    impl $crate::Asn1Info for $rs_type {
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
/// This macro defines the BerSerialize trait for a rust struct. The code generated
/// will serialize the specified fields in the order that they are given.
macro_rules! asn1_sequence_serialize {
  ($rs_type:ty, $($item:ident),*) => (
    impl $crate::BerSerialize for $rs_type {
      fn serialize_value<E: $crate::ber::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        let mut bytes = Vec::new();
        let mut count: u64 = 0;
        // For each declared sequence member, serialize it onto the stream.
        $(
          count += 1;
          // If encoding uses implicit tag, skip context-specific tag.
          if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit {
            try!($crate::BerSerialize::serialize_enc(&self.$item, e, writer));
          // Otherwise encode the context-specific tag.
          } else {
            try!($crate::BerSerialize::serialize_enc(&self.$item, e, &mut bytes));
            let tag = $crate::tag::Tag {
              class: $crate::tag::Class::ContextSpecific,
              tagnum: count.into(),
              constructed: true,
            };
            let len: $crate::tag::Len = Some(bytes.len() as $crate::tag::LenNum).into();
            try!($crate::tag::write_taglen(tag, len, writer));
            try!(writer.write_all(&mut bytes));

            bytes.clear();
          }
        )*
        Ok(())
      }
    }
  )
}

#[macro_export]
/// This macro defines the BerDeserialize trait for a rust struct. The code generated
/// will deserialize the specified fields in the order that they are given.
macro_rules! asn1_sequence_deserialize {
  ($rs_type:ident, $($item:ident),*) => (
    impl $crate::BerDeserialize for $rs_type {
      fn deserialize_value<E: $crate::ber::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, _: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        let mut count: u64 = 0;
        $(
          // Iterate count.
          // FIXME: Does this start from 0 or 1?
          count += 1;

          // Use field name as variable name, due to hygiene this won't conflict with any
          // defined locally.
          let $item;
          {
            // Create a copy of what our tag context-specific tag would look like.
            let our_tag = $crate::tag::Tag {
              class: $crate::tag::Class::ContextSpecific,
              tagnum: count.into(),
              constructed: true,
            };
            let tag = try!($crate::tag::Tag::read_tag(reader));

            // If encoding uses implicit tagging, throw an error if this isn't an implicit tag.
            if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit && tag == our_tag {
              return Err($crate::err::DecodeError::ExplicitTag);
            }

            let len = try!($crate::tag::Len::read_len(reader));

            // If the tag matches our tag, decode the len and call the normal deserialize function.
            $item = if tag == our_tag {
              // We don't have anything to do with the len, technically we should use it to
              // check the length decoded.
              let _ = len;
              try!($crate::BerDeserialize::deserialize_enc(e, reader))
            // Otherwise decode it as the inner type. (We give the tag that we
            // decoded, and the function will decode the length itself).
            } else {
              try!($crate::BerDeserialize::deserialize_with_tag(e, reader, tag, len))
            };
          }
        )*
        Ok( $rs_type { $(
          $item: $item,
        )* })
      }
    }
  )
}
