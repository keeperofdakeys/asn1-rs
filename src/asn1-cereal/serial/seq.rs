#[macro_export]
macro_rules! asn1_info {
  ($rs_type:ty, $asn1_ty:expr, $class:expr, $tagnum:expr, $constructed:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_tag() -> $crate::tag::Tag {
        $crate::tag::Tag {
          class: $class.into(),
          tagnum: $tagnum.into(),
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
macro_rules! asn1_sequence_serialize {
  ($rs_type:ty, $($item:ident),*) => (
    impl serial::traits::Asn1Serialize for $rs_type {
      fn serialize_imp<W: io::Write>(&self, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        let mut bytes = Vec::new();
        let mut count: u64 = 0;
        // For each declared sequence member, serialize it onto the stream.
        $(
          count += 1;
          try!(
            serial::traits::Asn1Serialize::serialize_exp(&self.$item, &mut bytes)
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
macro_rules! asn1_sequence_deserialize {
  ($rs_type:ident, $($item:ident),*) => (
    impl serial::traits::Asn1Deserialize for $rs_type {
      fn deserialize_imp<I: Iterator<Item=io::Result<u8>>>(reader: &mut I, _: $crate::tag::Len) -> Result<Self, $crate::err::DecodeError> {
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
