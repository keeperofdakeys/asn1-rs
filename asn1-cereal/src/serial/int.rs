//! Implementation of the serialization traits for Rust integers.

use std::io;

use tag;
use err;
use serial;
use enc;
use byte::{read_byte, write_byte};

use std::cmp;

/// Generate the ASN.1 int implementation for an int type.
macro_rules! asn1_cereal_int {
  ($rs_type:ty, 1, $unsigned:expr) => (
    impl serial::Asn1Info for $rs_type {
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

    impl serial::Asn1Serialize for $rs_type {
      fn serialize_value<E: enc::Asn1EncRules, W: io::Write>
          (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
        try!(write_byte(writer, *self as u8));
        return Ok(());
      }
    }
  );
  ($rs_type:ty, $size:expr, $unsigned:expr) => (
    impl serial::Asn1Info for $rs_type {
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

    impl serial::Asn1Serialize for $rs_type {
      fn serialize_value<E: enc::Asn1EncRules, W: io::Write>
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

    impl serial::Asn1Deserialize for $rs_type {
      fn deserialize_value<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
          (_: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError> {
        let len_num = try!(len.ok_or(err::DecodeError::PrimIndef));

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

asn1_cereal_int!(i8, 1, false);
asn1_cereal_int!(i16, 2, false);
asn1_cereal_int!(i32, 4, false);
asn1_cereal_int!(i64, 8, false);

asn1_cereal_int!(u8, 1, true);
asn1_cereal_int!(u16, 2, true);
asn1_cereal_int!(u32, 4, true);
asn1_cereal_int!(u64, 8, true);

#[cfg(target_pointer_width = "16")]
asn1_cereal_int! { isize, 2, false }
#[cfg(target_pointer_width = "32")]
asn1_cereal_int! { isize, 4, false }
#[cfg(target_pointer_width = "64")]
asn1_cereal_int! { isize, 8, false }

#[cfg(target_pointer_width = "16")]
asn1_cereal_int! { usize, 2, true }
#[cfg(target_pointer_width = "32")]
asn1_cereal_int! { usize, 4, true }
#[cfg(target_pointer_width = "64")]
asn1_cereal_int! { usize, 8, true }
