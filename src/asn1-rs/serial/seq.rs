use std::io;

use tag;
use err;

#[macro_export]
macro_rules! asn1_info {
  ($rs_type:ty, $asn1_ty:expr, $class:expr, $tagnum:expr, $constructed:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type() -> tag::Type {
        tag::Type::from($asn1_ty)
      }
      
      fn asn1_class() -> tag::Class {
        tag::Class::from($class)
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

#[macro_export]
macro_rules! asn1_sequence_info {
  ($rs_type:ty, $asn1_ty:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type(&self) -> tag::Type {
        $asn1_ty.into()
      }
      
      fn asn1_class(&self) -> tag::Class {
        tag::Class::Universal
      }

      fn asn1_tagnum(&self) -> tag::TagNum {
        tag::TagNum::from(0x10 as u8)
      }

      fn asn1_constructed(&self) -> bool {
        true
      }
    }
  )
}

#[macro_export]
macro_rules! asn1_sequence_serialize {
  ($rs_type:ty, $($item:ident),*) => (
    impl serial::traits::Asn1Serialize for $rs_type {
      fn serialize_bytes<W: io::Write>(&self, writer: &mut W) -> Result<(), err::EncodeError> {
        let mut bytes = Vec::new();
        let mut count: u64 = 0;
        // For each declared sequence member, serialize it onto the stream.
        $(
          count += 1;
          try!(
            serial::traits::Asn1Serialize::serialize(&self.$item, &mut bytes)
          );
          let tag = tag::Tag {
            class: tag::Class::ContextSpecific,
            tagnum: count.into(),
            len: Some(bytes.len() as tag::LenNum).into(),
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
  ($rs_type:ty) => (
    impl serial::traits::Asn1Deserialize for u64 {
      fn deserialize<I: Iterator<Item=io::Result<u8>>>(reader: I) -> Result<Self, err::DecodeError> {
        unimplemented!();
      }
    }
  )
}
