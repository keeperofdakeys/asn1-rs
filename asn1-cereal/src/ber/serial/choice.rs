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
//!     B(String),
//!   };
//!
//!   enum Enum2 {
//!     A(u64),
//!     B(u64),
//!   };
//!
//!   // A choice type with no tag.
//!   // CHOICE { a INTEGER, b PRINTABLE STRING }
//!   ber_choice!(Enum1, "CHOICE", A, u64; B, String;);
//!
//!   // OR
//!
//!   // A choice type with a custom tag on a variant.
//!   // CHOICE { a INTEGER, b [0] INTEGER }
//!   asn1_info!(Enum2, "CHOICE2");
//!   ber_choice_serialize!(Enum2, A, u64; B, u64;);
//!   ber_choice_deserialize!(Enum2, A, u64; [0] B, u64;);
//! }
//! ```

#[macro_export]
/// Generate the Asn1Info implemention for an ASN.1 Choice type, represented
/// by a Rust enum.
macro_rules! ber_choice {
  ($rs_type:ident, [$($opts:tt)*], $asn1_type:expr, $($args:tt)*) => (
    asn1_info!($rs_type, [$($opts)*], $asn1_type);
    ber_choice_serialize!($rs_type, $($args)*);
    ber_choice_deserialize!($rs_type, $($args)*);
  );
  ($rs_type:ident, $asn1_type:expr, $($args:tt)*) => (
    asn1_info!($rs_type, $asn1_type);
    ber_choice_serialize!($rs_type, $($args)*);
    ber_choice_deserialize!($rs_type, $($args)*);
  );
}

#[macro_export]
/// Generate the BerSerialize implemention for an ASN.1 Choice type, represented
/// by a Rust enum.
macro_rules! ber_choice_serialize {
  (_ { $rs_type:ident $this:ident $e:ident $writer:ident [$($items:ident, $tags:expr,)*] }
      [$($opts:tt)*] $item:ident, $inner:ty; $($args:tt)*) => (
    let tag = Some(asn1_spec_tag!([$($opts)*]));
    ber_choice_serialize!(_ { $rs_type $this $e $writer [$($items, $tags,)* $item, tag,] } $($args)*)
  );
  (_ { $rs_type:ident $this:ident $e:ident $writer:ident [$($items:ident, $tags:expr,)*] }
      $item:ident, $inner:ty; $($args:tt)*) => (
    let tag: Option<$crate::tag::Tag> = None;
    ber_choice_serialize!(_ { $rs_type $this $e $writer [$($items, $tags,)* $item, tag,] } $($args)*)
  );
  (_ { $rs_type:ident $this:ident $e:ident $writer:ident [$($items:ident, $tags:expr,)*] }) => (
    match *$this {
      $( $rs_type::$items(ref item) => {
        if let Some(tag) = $tags {
          let mut bytes = Vec::new();
          try!(item.serialize_enc($e, &mut bytes));
          try!(tag.write_tag($writer));
          let len: $crate::tag::Len = Some(bytes.len() as $crate::tag::LenNum).into();
          try!(len.write_len($writer));
          try!($writer.write_all(&mut bytes));
        } else {
          try!(item.serialize_enc($e, $writer));
        }
      } ),*
    };
  );
  ($rs_type:ident, $($args:tt)*) => (
    impl $crate::BerSerialize for $rs_type {
      fn serialize_value<E: $crate::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        ber_choice_serialize!(_ { $rs_type self e writer [] } $($args)*);
        Ok(())
      }
    }
  );
}

#[macro_export]
/// Generate the BerDeserialize implemention for an ASN.1 Choice type, represented
/// by a Rust enum.
macro_rules! ber_choice_deserialize {
  (_ { $rs_type:ident $e:ident $reader:ident $tag:ident $len:ident [$($tags:expr, $funcs:expr,)*] }
      [$($opts:tt)*] $item:ident, $inner:ty; $($args:tt)*) => (
    let tag = asn1_spec_tag!([$($opts)*]);
    let func = |reader| Ok($rs_type::$item(
      try!($crate::BerDeserialize::deserialize_enc($e, reader))
    ));
    ber_choice_deserialize!(_ { $rs_type $e $reader $tag $len [$($tags, $funcs,)* tag, func,] } $($args)*)
  );
  (_ { $rs_type:ident $e:ident $reader:ident $tag:ident $len:ident [$($tags:expr, $funcs:expr,)*] }
      $item:ident, $inner:ty; $($args:tt)*) => (
    let tag = <$inner as $crate::Asn1Info>::asn1_tag().unwrap();
    let func = |reader| Ok($rs_type::$item(
      try!($crate::BerDeserialize::deserialize_with_tag($e, reader, $tag, $len))
    ));
    ber_choice_deserialize!(_ { $rs_type $e $reader $tag $len [$($tags, $funcs,)* tag, func,] } $($args)*)
  );
  (_ { $rs_type:ident $e:ident $reader:ident $tag:ident $len:ident [$($tags:expr, $funcs:expr,)*] }) => (
    return match $tag {
      $( tag @ $crate::tag::Tag { .. } if tag == $tags => $funcs($reader) ),*,
      _ => unimplemented!(),
    };
  );
  ($rs_type:ident, $($args:tt)*) => (
    impl $crate::BerDeserialize for $rs_type {
      fn deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len)
          -> Result<Self, $crate::err::DecodeError> {
        ber_choice_deserialize!(_ { $rs_type e reader tag len [] } $($args)*);
      }
    }
  );
}
