//! Macros to generate the implementation of the serialization traits for Rust
//! collections, as ASN.1 sequence of.

#[macro_export]
macro_rules! asn1_sequence_of {
  ($rs_type:ty) => (
    asn1_sequence_of_serialize!($rs_type);
    asn1_sequence_of_deserialize!($rs_type);
  )
}

#[macro_export]
macro_rules! asn1_sequence_of_serialize {
  ($rs_type:ty) => (
    impl $crate::serial::Asn1Serialize for $rs_type
        where $rs_type: std::iter::IntoIterator {
      fn serialize_bytes<E: $crate::enc::Asn1EncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), $crate::err::EncodeError> {
        // Call serialize_enc on each item.
        for item in self.iter() {
          try!($crate::serial::Asn1Serialize::serialize_enc(item, e, writer));
        }
        Ok(())
      }
    }
  )
}

#[macro_export]
macro_rules! asn1_sequence_of_deserialize {
  ($rs_type:ty) => (
    impl $crate::serial::Asn1Deserialize for $rs_type {
      fn deserialize_bytes<E: $crate::enc::Asn1EncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, len: Option<$crate::tag::LenNum>) -> Result<Self, $crate::err::DecodeError> {
        struct SeqOfDecoder<T, F, J: Iterator<Item=std::io::Result<u8>>> {
          len: Option<$crate::tag::LenNum>,
          reader: $crate::byte::ByteReader<J>,
          e: F,
          _p: Option<T>,
        }

        impl<T, F, J> Iterator for SeqOfDecoder<T, F, J> where
            F: $crate::enc::Asn1EncRules,
            J: Iterator<Item=std::io::Result<u8>>,
            T: $crate::serial::Asn1Deserialize {
          type Item = Result<T, $crate::err::DecodeError>;

          #[inline]
          fn next(&mut self) -> Option<Self::Item> {
            // Compare decoded length with length in tag.
            // Put this first to handle zero-length elements.
            match self.len.partial_cmp(&Some(self.reader.count)) {
              // Return an error when we've decoded too much.
              Some(std::cmp::Ordering::Less) => return Some(Err($crate::err::DecodeError::GreaterLen)),
              // Finish loop when equal, we must be finished.
              Some(std::cmp::Ordering::Equal) => return None,
              // Continue when we are still decoding.
              Some(std::cmp::Ordering::Greater) => {},
              // Continue when using indefinite length encoding.
              None => {},
            }

            let tag = match $crate::tag::Tag::read_tag(&mut self.reader) {
              Ok(t) => t,
              Err(e) => return Some(Err(e)),
            };
            let len = match $crate::tag::Len::read_len(&mut self.reader) {
              Ok(l) => l,
              Err(e) => return Some(Err(e)),
            };

            // Handle end of indefinite length encoding.
            if tag.tagnum == 0 && tag.class == 0.into() &&
               len.as_num() == Some(0) {
              return None;
            }

            return Some($crate::serial::Asn1Deserialize::deserialize_bytes(self.e, &mut self.reader, len.as_num()));
          }
        }

        let mut decoder = SeqOfDecoder { e: e, len: len, reader: $crate::byte::ByteReader::new(reader), _p: None };
        let v: Result<$rs_type, _> = std::iter::FromIterator::from_iter(decoder.by_ref());
        Ok(try!(v))
      }
    }
  )
}

use std;

asn1_info!(Vec<u64>, 0x3, 0x1, true, "TYPE2");
asn1_sequence_of!(Vec<u64>);
