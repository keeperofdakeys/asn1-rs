//! Macros to generate the implementation of the serialization traits for Rust
//! structs, as ASN.1 sequences.
//!
//! You can either use the shortcut `ber_sequence!` macro, or each of
//! `asn1_sequence_info!`, `ber_sequence_serialize!` and `ber_sequence_deserialize!`.
//!
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   struct ShortSequence {
//!     z: u64,
//!     y: u32,
//!   }
//!
//!   ber_sequence!(
//!     ShortSequence,
//!     "SHORT_SEQUENCE",
//!     z;
//!     y;
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
//!   ber_sequence_serialize!(SomeSequence, a; b; c;);
//!   ber_sequence_deserialize!(SomeSequence, a; b; c;);
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
macro_rules! ber_sequence {
  ($rs_type:ident, $asn1_ty:expr, $($args:tt)*) => (
    asn1_sequence_info!($rs_type, $asn1_ty);
    ber_sequence_serialize!($rs_type, $($args)*);
    ber_sequence_deserialize!($rs_type, $($args)*);
  );
  ($rs_type:ident, [$($args:tt)*], $asn1_ty:expr, $($args:tt)*) => (
    asn1_sequence_info!($rs_type, [$($args:tt)*], $asn1_ty);
    ber_sequence_serialize!($rs_type, $($args)*);
    ber_sequence_deserialize!($rs_type, $($args)*);
  );
}

#[macro_export]
/// This macro defines the Asn1Info trait for a rust struct. This allows the other
/// traits to get information about this type. If you need to provide a custom
/// class or tag, consider using the asn1_info! macro.
macro_rules! asn1_sequence_info {
  ($rs_type:ident, [$($args:tt)*], $asn1_ty:expr) => (
    asn1_info!($rs_type, [$($args:tt)*], $asn1_ty);
  );
  ($rs_type:ident, $asn1_ty:expr) => (
    asn1_info!($rs_type, [UNIVERSAL 16], $asn1_ty);
  );
}

#[macro_export]
/// This macro defines the BerSerialize trait for a rust struct. The code generated
/// will serialize the specified fields in the order that they are given.
macro_rules! ber_sequence_serialize {
  ($rs_type:ty, $($args:tt)*) => (
    impl $crate::BerSerialize for $rs_type {
      fn serialize_value<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        let mut bytes = Vec::new();
        let mut _count: u64 = 0;
        // For each declared sequence member, serialize it onto the stream.
        ber_sequence_serialize!(_ { self e writer bytes _count } $($args)*);
        Ok(())
      }
    }
  );

  // Handle field creation.
  // Expand a field with no options.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident }
      $item:ident; $($args:tt)*) => (
    ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $item (); $($args)*);
  );
  // Expand an OPTIONAL field with no tag.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident }
      $item:ident (OPTIONAL); $($args:tt)*) => (
    ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $item ([] OPTIONAL); $($args)*);
  );
  // Expand an OPTIONAL field with a tag.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident }
      $item:ident ([$($opts:tt)*] OPTIONAL); $($args:tt)*) => (
    let tag = asn1_spec_tag!({ $count } [$($opts)*]);
    if let Some(ref val) = $this.$item {
      ber_sequence_serialize!(_ { $this $e $writer $bytes $count tag, val } $($args)*);
    } else {
      ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $($args)*);
    }
  );
  // Expand a field with a DEFAULT.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident }
      $item:ident (DEFAULT $default:expr); $($args:tt)*) => (
    ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $item ([] DEFAULT $default); $($args)*);
  );
  // Expand a field with a DEFAULT and a tag.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident }
      $item:ident ([$($opts:tt)*] DEFAULT $default:expr); $($args:tt)*) => (
    let tag = asn1_spec_tag!({ $count } [$($opts)*]);
    if &$this.$item != &$default {
      ber_sequence_serialize!(_ { $this $e $writer $bytes $count tag, &$this.$item } $($args)*);
    } else {
      ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $($args)*);
    }
  );
  // Expand a field that has options.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident }
      $item:ident ($($opts:tt)*); $($args:tt)*) => (
    let tag = asn1_spec_tag!({ $count } [$($opts)*]);

    // If encoding uses implicit tag, skip context-specific tag.
    if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit {
      try!($crate::BerSerialize::serialize_enc(&$this.$item, $e, $writer));
      ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $($args)*);
    // Otherwise encode the context-specific tag.
    } else {
      ber_sequence_serialize!(_ { $this $e $writer $bytes $count tag, &$this.$item } $($args)*);
    }
  );
  // Write a field with a tag.
  (_ { $this:ident $e:ident $writer:ident $bytes:ident $count:ident $tag:expr, $value:expr }
      $($args:tt)*) => (
    try!($crate::BerSerialize::serialize_enc($value, $e, &mut $bytes));
    let len: $crate::tag::Len = Some($bytes.len() as $crate::tag::LenNum).into();
    try!($crate::tag::write_taglen($tag, len, $writer));
    try!($writer.write_all(&mut $bytes));
    $bytes.clear();

    ber_sequence_serialize!(_ { $this $e $writer $bytes $count } $($args)*);
  );
  (_ { $($args:tt)* } ) => ();
}

#[macro_export]
/// This macro defines the BerDeserialize trait for a rust struct. The code generated
/// will deserialize the specified fields in the order that they are given.
macro_rules! ber_sequence_deserialize {
  // Handle fields.
  // Expand a field with no options.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $item:ident; $($args:tt)*) => (
    ber_sequence_deserialize!(
      _ { $rs_type $e $reader $count $tag [$($fields)*] } $item ([]); $($args)*
    );
  );
  // Expand a field with empty options.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $item:ident (); $($args:tt)*) => (
    ber_sequence_deserialize!(
      _ { $rs_type $e $reader $count $tag [$($fields)*] } $item ([]); $($args)*
    );
  );
  // Expand a field with empty options and a tag.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $field:ident ([$($opts:tt)*]); $($args:tt)*) => (
    let $field = {
      let tag = if let Some(t) = $tag {
        $tag = None;
        t
      } else {
        try!($crate::tag::Tag::read_tag($reader))
      };
      let our_tag = asn1_spec_tag!({ $count } [$($opts)*]);

      ber_sequence_deserialize!(_ { $rs_type $e $reader $count } tag, our_tag )
    };
    ber_sequence_deserialize!(_ { $rs_type $e $reader $count $tag [$($fields)* $field] } $($args)*);
  );
  // Expand an OPTIONAL field with no tag.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $item:ident (OPTIONAL); $($args:tt)*) => (
    ber_sequence_deserialize!(
      _ { $rs_type $e $reader $count $tag [$($fields)*] } $item ([] OPTIONAL); $($args)*
    );
  );
  // Expand an OPTIONAL field with a tag.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $field:ident ([$($opts:tt)*] OPTIONAL); $($args:tt)*) => (
    let $field = {
      let tag = match $tag {
        Some(t) => t,
        None => {
          let t = try!($crate::tag::Tag::read_tag($reader));
          $tag = Some(t); t
        },
      };

      let our_tag = asn1_spec_tag!({ $count } [$($opts)*]);

      if tag == our_tag {
        $tag = None;
        let _ = try!($crate::tag::Len::read_len($reader));
        Some(try!($crate::BerDeserialize::deserialize_enc($e, $reader)))
      } else {
        None
      }
    };
    ber_sequence_deserialize!(_ { $rs_type $e $reader $count $tag [$($fields)* $field] } $($args)*);
  );
  // Expand a DEFAULT field with no tag.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $item:ident (DEFAULT $default:expr); $($args:tt)*) => (
    ber_sequence_deserialize!(
      _ { $rs_type $e $reader $count $tag [$($fields)*] } $item ([] DEFAULT $default); $($args)*
    );
  );
  // Expand a DEFAULT field with a tag.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [$($fields:ident)*] }
      $field:ident ([$($opts:tt)*] DEFAULT $default:expr); $($args:tt)*) => (
    let $field = {
      let tag = match $tag {
        Some(t) => t,
        None => {
          let t = try!($crate::tag::Tag::read_tag($reader));
          $tag = Some(t); t
        },
      };

      let our_tag = asn1_spec_tag!({ $count } [$($opts)*]);

      if tag == our_tag {
        $tag = None;
        let _ = try!($crate::tag::Len::read_len($reader));
        try!($crate::BerDeserialize::deserialize_enc($e, $reader))
      } else {
        $default
      }
    };
    ber_sequence_deserialize!(_ { $rs_type $e $reader $count $tag [$($fields)* $field] } $($args)*);
  );
  // Create the implementation for a field.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident } $tag:expr, $our_tag:expr) => ({
    // If encoding uses implicit tagging, throw an error if this isn't an implicit tag.
    if E::tag_rules() == $crate::ber::enc::TagEnc::Implicit && $tag == $our_tag {
      return Err($crate::err::DecodeError::ExplicitTag);
    }

    let len = try!($crate::tag::Len::read_len($reader));

    // If the tag matches our tag, decode the len and call the normal deserialize function.
    if $tag == $our_tag {
      // We don't have anything to do with the len, technically we should use it to
      // check the length decoded.
      let _ = len;
      try!($crate::BerDeserialize::deserialize_enc($e, $reader))
    // Otherwise decode it as the inner type. (We give the tag that we
    // decoded, and the function will decode the length itself).
    } else {
      try!($crate::BerDeserialize::deserialize_with_tag($e, $reader, $tag, len))
    }
  });
  // When no fields are left, build the struct.
  (_ { $rs_type:ident $e:ident $reader:ident $count:ident $tag:ident [ $($field:ident)* ] }) => (
    return Ok($rs_type { $(
      $field: $field
    ),* })
  );

  ($rs_type:ident, $($args:tt)*) => (
    impl $crate::BerDeserialize for $rs_type {
      fn deserialize_value<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, _: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        let mut _count: u64 = 0;
        let mut _tag: Option<$crate::tag::Tag> = None;
        ber_sequence_deserialize!(_ { $rs_type e reader _count _tag [ ] } $($args)*);
      }
    }
  );
}
