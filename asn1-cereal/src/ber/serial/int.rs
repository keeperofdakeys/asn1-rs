//! Implementation of the serialization traits for Rust integers.

use std::io;

use ::{BerSerialize, BerDeserialize, Asn1Info};
use tag;
use err;
use byte::{read_byte, write_byte};

use std::cmp;

/// Generate the ASN.1 int implementation for an int type.
macro_rules! ber_cereal_int {
  ($rs_type:ty, 1, $unsigned:expr) => (
    impl Asn1Info for $rs_type {
      fn asn1_tag() -> tag::Tag {
        tag::Tag {
          class: tag::Class::Universal,
          tagnum: 2u8.into(),
          constructed: false,
        }
      }

      fn asn1_type() -> tag::Type {
        tag::Type::from("INTEGER")
      }
    }

    impl BerSerialize for $rs_type {
      fn serialize_value<E: ::BerEncRules, W: io::Write>
          (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
        try!(write_byte(writer, *self as u8));
        Ok(())
      }
    }
  );
  ($rs_type:ty, $size:expr, $unsigned:expr) => (
    impl Asn1Info for $rs_type {
      fn asn1_tag() -> tag::Tag {
        tag::Tag {
          class: tag::Class::Universal,
          tagnum: 2u8.into(),
          constructed: false,
        }
      }

      fn asn1_type() -> tag::Type {
        tag::Type::from("INTEGER")
      }
    }

    impl BerSerialize for $rs_type {
      fn serialize_value<E: ::BerEncRules, W: io::Write>
          (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
        let mut started = false;
        // Loop through bytes in int backwards, start writing when first non-zero byte is encounted.
        for offset in (0..$size).rev() {
          let shifted: $rs_type = self >> (offset * 8);
          let byte: u8 = (shifted & 0xff) as u8;
          if !started {
            // Skip starting 0 bytes.
            if byte == 0 {
              continue;
            // If starting byte has the highest bit set to 1, and this is an unsigned
            // number, write a zero byte first. This allows the decoder to know that
            // this is actually a positive number in ones complement.
            } else if byte & 0x80 != 0 && $unsigned {
              try!(write_byte(writer, 0x00));
            }
            started = true;
          }
          try!(write_byte(writer, byte));
        }

        // If we haven't written anything, write a zero byte.
        if !started {
          try!(write_byte(writer, 0 as u8));
        }
        Ok(())
      }
    }

    impl BerDeserialize for $rs_type {
      fn deserialize_value<E: ::BerEncRules, I: Iterator<Item=io::Result<u8>>>
          (_: E, reader: &mut I, len: tag::Len) -> Result<Self, err::DecodeError> {
        let len_num = try!(len.as_num().ok_or(err::DecodeError::PrimIndef));

        let mut int: $rs_type = 0;
        let size = cmp::min($size, len_num);

        // If this is a u8/i8, just read a single byte.
        if $size == 1 {
          // FIXME: This doesn't use all len_num bytes.
          int = try!(read_byte(reader)) as $rs_type;
        } else {
          for _ in 0..size {
            let byte = try!(read_byte(reader));
            int = (int << 8) + (byte as $rs_type);
          }
        }

        Ok(int)
      }
    }
  );
}

ber_cereal_int!(i8, 1, false);
ber_cereal_int!(i16, 2, false);
ber_cereal_int!(i32, 4, false);
ber_cereal_int!(i64, 8, false);

ber_cereal_int!(u8, 1, true);
ber_cereal_int!(u16, 2, true);
ber_cereal_int!(u32, 4, true);
ber_cereal_int!(u64, 8, true);

#[cfg(target_pointer_width = "16")]
ber_cereal_int! { isize, 2, false }
#[cfg(target_pointer_width = "32")]
ber_cereal_int! { isize, 4, false }
#[cfg(target_pointer_width = "64")]
ber_cereal_int! { isize, 8, false }

#[cfg(target_pointer_width = "16")]
ber_cereal_int! { usize, 2, true }
#[cfg(target_pointer_width = "32")]
ber_cereal_int! { usize, 4, true }
#[cfg(target_pointer_width = "64")]
ber_cereal_int! { usize, 8, true }
