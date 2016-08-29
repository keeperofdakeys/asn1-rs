use std::io;

use tag;
use err;

#[macro_export]
macro_rules! asn1_info {
  ($rs_type:ty, $asn1_ty:expr, $class:expr, $tagnum:expr, $constructed:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type() -> $crate::tag::Type {
        $crate::tag::Type::from($asn1_ty)
      }
      
      fn asn1_class() -> $crate::tag::Class {
        $crate::tag::Class::from($class)
      }

      fn asn1_tagnum() -> $crate::tag::TagNum {
        $crate::tag::TagNum::from($tagnum)
      }

      fn asn1_constructed() -> bool {
        $constructed
      }
    }
  )
}

#[macro_export]
macro_rules! asn1_sequence_info {
  ($rs_type:ty, $asn1_ty:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type() -> $crate::tag::Type {
        $asn1_ty.into()
      }
      
      fn asn1_class() -> $crate::tag::Class {
        $crate::tag::Class::Universal
      }

      fn asn1_tagnum() -> $crate::tag::TagNum {
        $crate::tag::TagNum::from(0x10 as u8)
      }

      fn asn1_constructed() -> bool {
        true
      }
    }
  )
}

#[macro_export]
macro_rules! asn1_sequence_serialize {
  ($rs_type:ty, $($item:ident),*) => (
    impl serial::traits::Asn1Serialize for $rs_type {
      fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
        let mut bytes = Vec::new();
        let mut count: u64 = 0;
        // For each declared sequence member, serialize it onto the stream.
        $(
          count += 1;
          try!(
            serial::traits::Asn1Serialize::serialize_exp(&self.$item, &mut bytes)
          );
          let tag = $crate::tag::Tag {
            class: $crate::tag::Class::ContextSpecific,
            tagnum: count.into(),
            len: Some(bytes.len() as $crate::tag::LenNum).into(),
            constructed: true,
          };
          try!(tag.encode_tag(writer));
          try!(writer.write(&mut bytes));

          bytes.clear();
        )*
        Ok(())
      }
    }
  )
}

#[macro_export]
macro_rules! asn1_sequence_deserialize {
  ($rs_type:ident, $($item:ident),*) => (
    impl serial::traits::Asn1Deserialize for $rs_type {
      fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, len: $crate::tag::Len) -> Result<Self, err::DecodeError> {
        let mut total_len: $crate::tag::LenNum = 0;
        Ok( $rs_type { $(
          $item: {
            let tag = try!($crate::tag::Tag::decode_tag(reader));
            try!($crate::serial::traits::Asn1Deserialize::deserialize_exp(reader))
          },
        )* })
      }
    }
  )
}
