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
//!   ber_choice!(Enum1, [1], "CHOICE", A; B;);
//!
//!   // OR
//!
//!   asn1_info!(Enum2, 0x3, 0x1, true, "CHOICE2");
//!   ber_choice_serialize!(Enum2, A; B;);
//!   ber_choice_deserialize!(Enum2, A; B;);
//! }
//! ```

#[macro_export]
macro_rules! ber_choice {
  ($rs_type:ident, [$($opts:tt)*], $asn1_type:expr, $($args:tt)*) => (
    asn1_info!($rs_type, [$($opts)*], $asn1_type);
    // ber_choice_serialize!($rs_type, $($args)*);
    ber_choice_deserialize!($rs_type, $($args)*);
  );
  ($rs_type:ident, $asn1_type:expr, $($args:tt)*) => (
    asn1_info!($rs_type, $asn1_type);
    // ber_choice_serialize!($rs_type, $($args)*);
    ber_choice_deserialize!($rs_type, $($args)*);
  );
}

#[macro_export]
macro_rules! ber_choice_serialize {
  ($rs_type:ident, $($args:tt)*) => (
    impl $crate::BerSerialize for $rs_type {
      fn serialize_value<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        self.0.serialize_enc(e, writer);
      }
    }
  );
}

#[macro_export]
macro_rules! ber_choice_deserialize {
  (_ { $e:ident $reader:ident $tag:ident $len:ident [$($tags:expr, $funcs:expr,)*] }
      $item:ident; $($args:tt)*) => (
    let tag = $item.0.asn1_tag();
    let func = || $item(try!($crate::BerDeserialize::deserialize_with_tag($e, $reader, $tag, $len)));
    ber_choice_deserialize!(_ { $e $reader $tag $len [$($tags, $funcs,)* tag, func,] } $($args)*)
  );
  (_ { $e:ident $writer:ident $tag:ident $len:ident [$($tags:expr, $funcs:expr,)*] }
      [$($opts:tt)*] $item:ident; $($args:tt)*) => (
    let tag = asn1_spec_tag!([$($opts:tt)*]);
    let func = || $item(try!($crate::BerDeserialize::deserialize_enc($e, $reader)));
    ber_choice_deserialize!(_ { $e $reader $tag $len [$($tags, $funcs,)* tag, func,] } $($args)*)
  );
  (_ { $e:ident $writer:ident $tag:ident $len:ident [$($tags:expr, $funcs:expr,)*] }) => (
    Ok(match $tag {
      $( tag @ $crate::tag::Tag { .. } if tag == $tags => { $funcs() } ),*
    })
  );
  ($rs_type:ident, $($args:tt)*) => (
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

      fn deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len)
          -> Result<Self, $crate::err::DecodeError> {
        let (tag, len) = try!($crate::tag::read_taglen(reader));

        ber_choice_deserialize!(_ { e reader tag len [] } $($args)*);
      }
    }
  );
}
