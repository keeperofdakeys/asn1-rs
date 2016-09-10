//! Implementation of the serialization traits for Rust integers.

use std::io;

use tag;
use err;
use serial;
use enc;
use byte::{read_byte, write_byte};

use std::cmp;

macro_rules! asn1_cereal_uint {
  ($rs_type:ty, $size:expr) => (
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
      fn serialize_bytes<E: enc::Asn1EncRules, W: io::Write>
          (&self, _: E, writer: &mut W) -> Result<(), err::EncodeError> {
        let mut started = false;
        // Loop through bytes in int backwards, start writing when first non-zero byte is encounted.
        for offset in (0..8).rev() {
          let shifted: $rs_type = self >> (offset * 8);
          let byte: u8 = (shifted & 0xff) as u8;
          if !started {
            if byte == 0 {
              continue;
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
      fn deserialize_bytes<E: enc::Asn1EncRules, I: Iterator<Item=io::Result<u8>>>
          (_: E, reader: &mut I, len: Option<tag::LenNum>) -> Result<Self, err::DecodeError> {
        let len_num = try!(len.ok_or(err::DecodeError::PrimIndef));

        let mut int: $rs_type = 0;
        let size = cmp::min($size, len_num);

        for _ in 0..size {
          let byte = try!(read_byte(reader));
          int = (int << 8) + (byte as $rs_type);
        }

        Ok(int)
      }
    }
  )
}

// Can't bitshift a u8 by 8 bits, need to update deserialize_bytes.
// asn1_cereal_int!(u8, 1);
asn1_cereal_uint!(u16, 2);
asn1_cereal_uint!(u32, 4);
asn1_cereal_uint!(u64, 8);
