//! Macros to generate the implementation of the serialization traits for Rust
//! enums, as ASN.1 choice.
//!
//! You can either define both `asn1_info!` and `ber_choice!`, or all three
//! of `asn1_info!`, `ber_choice_serialize!` and `ber_choice_deserialize!`.
//!
//! ```
//! #[macro_use] extern crate asn1_cereal; fn main() {
//!   enum Enum1 {
//!     A(u64),
//!     B(u32),
//!   };
//!
//!   enum Enum2 {
//!     A(u64),
//!     B(u32),
//!   };
//!
//!   ber_choice!(Enum1, [1], "CHOICE", A, B);
//!
//!   // OR
//!
//!   asn1_info!(Enum2, 0x3, 0x1, true, "CHOICE2");
//!   ber_choice_serialize!(Enum2);
//!   ber_choice_deserialize!(Enum2, A, B);
//! }
//! ```

#[macro_export]
macro_rules! ber_choice {
  ($rs_type:ident, [$($args:tt)*], $asn1_type:expr, $($item:ident),*) => (
    asn1_info!($rs_type, [$($args)*], $asn1_type);
    ber_choice_serialize!($rs_type);
    ber_choice_deserialize!($rs_type, $($item),*);
  );
  ($rs_type:ident, $asn1_type:expr, $($item:ident),*) => (
    asn1_info!($rs_type, $asn1_type);
    ber_choice_serialize!($rs_type);
    ber_choice_deserialize!($rs_type, $($item),*);
  );
}

#[macro_export]
macro_rules! ber_choice_serialize {
  ($rs_type:ident) => (
    impl $crate::BerSerialize for $rs_type {
      fn serialize_enc<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        // FIXME: Can't call self.0 to call function on inner type.
        //
        // // Skip choice tag, we don't need to encode this.
        // self.0.serialize_value(e, writer)
        unimplemented!();
      }

      fn serialize_value<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        // FIXME: Can't call self.0 to call function on inner type.
        //
        // // Return inner types encoding, we don't care which variant it is.
        // self.0.serialize_enc(e, writer)
        unimplemented!();
      }
    }
  )
}

#[macro_export]
macro_rules! ber_choice_deserialize {
  ($rs_type:ident, $($item:ident),*) => (
    impl $crate::BerDeserialize for $rs_type {
      // FIXME: We can't call asn1_tag() on the inner type of the enum variant,
      // so we can't compare the tag with the one we get.
      //
      // fn deserialize_enc_tag<E: $crate::enc::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
      //     (e: E, reader: &mut I, tag: $crate::tag::Tag)
      //     -> Result<Self, $crate::err::DecodeError> {
      //   // Decode inner type based on tag.
      //   match tag {
      //     $(
      //       Self::$item.asn1_tag() => Self::$item(deserialize_enc_tag(e, reader, tag)),
      //     )*
      //   }
      // }

      fn deserialize_value<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, _: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
        // This should never be called?
        unimplemented!();
      }
    }
  )
}
