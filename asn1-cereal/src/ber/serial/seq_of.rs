//! Macros to generate the implementation of the serialization traits for Rust
//! iterators, as ASN.1 sequence of.

#[macro_export]
macro_rules! ber_sequence_of {
  ($rs_type:ty) => (
    ber_sequence_of_serialize!($rs_type);
    ber_sequence_of_deserialize!($rs_type);
  );
  ($rs_type:ty => $gen:ident) => (
    ber_sequence_of_serialize!($rs_type => $gen);
    ber_sequence_of_deserialize!($rs_type => $gen);
  );
}

#[macro_export]
macro_rules! ber_sequence_of_serialize {
  (impl: $rs_type:ty) => (
    fn serialize_value<E: $crate::BerEncRules, W: ::std::io::Write>
        (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
      // Call serialize_enc on each item.
      for item in (self).into_iter() {
        try!($crate::BerSerialize::serialize_enc(item, e, writer));
      }
      Ok(())
    }
  );
  ($rs_type:ty) => (
    impl $crate::BerSerialize for $rs_type {
      ber_sequence_of_serialize!{impl: $rs_type}
    }
  );
  ($rs_type:ty => $gen:ident) => (
    impl<$gen: $crate::BerSerialize> $crate::BerSerialize for $rs_type {
      ber_sequence_of_serialize!{impl: $rs_type}
    }
  );
}

#[macro_export]
macro_rules! ber_sequence_of_deserialize {
  (impl: $rs_type:ty) => (
    fn deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=::std::io::Result<u8>>>
        (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len) ->
        Result<Self, $crate::err::DecodeError> {
      struct SeqOfDecoder<T, F, J: Iterator<Item=::std::io::Result<u8>>> {
        len: $crate::tag::Len,
        reader: $crate::byte::ByteReader<J>,
        e: F,
        _p: Option<T>,
      }

      impl<T, F, J> Iterator for SeqOfDecoder<T, F, J> where
          F: $crate::BerEncRules,
          J: Iterator<Item=::std::io::Result<u8>>,
          T: $crate::BerDeserialize {
        type Item = Result<T, $crate::err::DecodeError>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
          // Compare decoded length with length in tag.
          // Put this first to handle zero-length elements.
          match self.len.partial_cmp(&self.reader.count) {
            // Return an error when we've decoded too much.
            Some(::std::cmp::Ordering::Less) => return Some(Err($crate::err::DecodeError::GreaterLen)),
            // Finish loop when equal, we must be finished.
            Some(::std::cmp::Ordering::Equal) => return None,
            // Continue when we are still decoding, or using
            // indefinite length encoding.
            Some(::std::cmp::Ordering::Greater) | None => {},
          }

          let (tag, len) = match $crate::tag::read_taglen(&mut self.reader) {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
          };

          // Handle end of indefinite length encoding.
          if tag.tagnum == 0 && tag.class == $crate::tag::Class::Universal &&
             len == $crate::tag::Len::Def(0) {
            return None;
          }

          Some($crate::BerDeserialize::deserialize_value(self.e, &mut self.reader, len))
        }
      }

      if Some(tag) != <Self as $crate::Asn1Info>::asn1_tag() {
        return Err($crate::err::DecodeError::TagTypeMismatch);
      }

      if len == $crate::tag::Len::Indef &&
         E::len_rules() == $crate::ber::enc::LenEnc::Definite {
        return Err($crate::err::DecodeError::IndefiniteLen);
      }

      let mut decoder = SeqOfDecoder {
        e: e,
        len: len.into(),
        reader: $crate::byte::ByteReader::new(reader),
        _p: None
      };
      let v: Result<$rs_type, _> = ::std::iter::FromIterator::from_iter(decoder.by_ref());
      Ok(try!(v))
    }
  );
  ($rs_type:ty) => (
    impl $crate::BerDeserialize for $rs_type {
      ber_sequence_of_deserialize!{impl: $rs_type}
    }
  );
  ($rs_type:ty => $gen:ident) => (
    impl<$gen: $crate::BerDeserialize> $crate::BerDeserialize for $rs_type {
      ber_sequence_of_deserialize!{impl: $rs_type}
    }
  );
}

asn1_info!(Vec<T> => T, ::tag::Class::Universal, 16, true, "SEQUENCE OF");
ber_sequence_of!(Vec<T> => T);
