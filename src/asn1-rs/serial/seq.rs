use std::io;

use tag;
use err;

macro_rules! asn1_info {
  ($rs_type:ty, $asn1_ty:expr, $class:expr, $tagnum:expr, $constructed:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type() -> tag::Type {
        tag::Type::from($asn1_ty)
      }
      
      fn asn1_class() -> tag::Class {
        tag::class::from($class)
      }

      fn asn1_tagnum() -> tag::TagNum {
        tag::TagNum::from($tagnum)
      }

      fn asn1_constructed() -> bool {
        $constructed
      }
    }
  )
}

macro_rules! asn1_sequence_info {
  ($rs_type:ty, $asn1_ty:expr, $($items:ident),*) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type() -> tag::Type {
        $asn1_ty.into()
      }
      
      fn asn1_class() -> tag::Class {
        tag::Class::Class::Universal
      }

      fn asn1_tagnum() -> tag::TagNum {
        tag::TagNum::from(0x10)
      }

      fn asn1_constructed() -> bool {
        true
      }
    }
  )
}

macro_rules! asn1_sequence_serialize {
  ($rs_type:ty, $($item:ident),*) => (
    impl serial::traits::Asn1Serialize for $rs_type {
      fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
        let tag = tag::Tag {
          class: self.asn1_class(),
          tagnum: self.asn1_tagnum(),
          constructed: self.asn1_constructed(),
          len: tag::Len::Indef,
        };
        try!(
          // Write initial tag.
          tag.encode_tag(writer);
          // For each declared sequence member, serialize it onto the stream.
          $(
            serial::traits::Asn1Serialize::serialize(self.$item, writer);
          )*
          // Write the end of indef length marker.
          byte::write_byte(writer, 0x00);
          byte::write_byte(writer, 0x00);
        );
      }
    }
  )
}

macro_rules! asn1_sequence_deserialize {
  ($rs_type:ty) => (
    impl serial::traits::Asn1Deserialize for u64 {
      fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I) -> Result<Self, err::DecodeError> {
        unimplemented!();
      }
    }
  )
}
