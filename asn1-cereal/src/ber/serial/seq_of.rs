//! Macros to generate the implementation of the serialization traits for Rust
//! iterators, as ASN.1 sequence of.
//!
//! This macro has already been implemented for Vec and HashSet, using the tags
//! SEQUENCE OF and SET OF respectively.

#[macro_export]
/// Generate a SEQUENCE OF implementation for an iterator type. `asn1_info!` must
/// be called manually.
///
/// ```rust,ignore
/// // Assuming A implements IntoIterator and FromIterator.
///
/// asn1_info!(A, [PRIVATE 5], "A");
///
///
/// ber_sequence_of!(A);
///
///  // OR
///
/// ber_sequence_of_serialize!(A);
/// ber_sequence_of_deserialize!(A);
///
///
/// // If A has a generic, you can instead do.
/// ber_sequence_of!(A<T> => T);
/// ```
macro_rules! ber_sequence_of {
  ($($token:tt)*) => (
    ber_sequence_of_serialize!($($token)*);
    ber_sequence_of_deserialize!($($token)*);
  );
}

#[macro_export]
/// Implement BerSerialize for a type, by iterating over each element, and
/// calling serialize_enc on each element.
///
/// Requires that the type implement IntoIterator, and can be used to implement
/// SEQUENCE/SET OF.
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
  ($rs_type:ty => $gen:ident, $($where_attr:tt)*) => (
    impl<$gen: $crate::BerSerialize> $crate::BerSerialize for $rs_type where $($where_attr)* {
      ber_sequence_of_serialize!{impl: $rs_type}
    }
  );
}

#[macro_export]
/// Implement BerDeserialize for a type, by collecting the elements from an iterator
/// built by calling deserialize_enc on the stream continually.
///
/// Requires that the type implement FromIterator, and can be used to implement
/// SEQUENCE/SET OF.
macro_rules! ber_sequence_of_deserialize {
  (impl: $rs_type:ty) => (
    fn deserialize_with_tag<E: $crate::BerEncRules, I: Iterator<Item=::std::io::Result<u8>>>
        (e: E, reader: &mut I, tag: $crate::tag::Tag, len: $crate::tag::Len) ->
        Result<Self, $crate::err::DecodeError> {
      struct SeqOfDecoder<T, F, J: Iterator<Item=::std::io::Result<u8>>> {
        len: $crate::tag::Len,
        reader: $crate::byte::ByteReader<J>,
        e: F,
        _p: ::std::marker::PhantomData<T>,
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
        _p: ::std::marker::PhantomData,
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
  ($rs_type:ty => $gen:ident, $($where_attr:tt)*) => (
    impl<$gen: $crate::BerDeserialize> $crate::BerDeserialize for $rs_type where $($where_attr)* {
      ber_sequence_of_deserialize!{impl: $rs_type}
    }
  );
}

use std::collections::HashSet;
use std::hash::Hash;

asn1_info!(Vec<T> => T, ::tag::Class::Universal, 16, true, "SEQUENCE OF");
ber_sequence_of!(Vec<T> => T);
asn1_info!(HashSet<T> => T, ::tag::Class::Universal, 17, true, "SET OF");
ber_sequence_of!(HashSet<T> => T, T: Eq + Hash);
