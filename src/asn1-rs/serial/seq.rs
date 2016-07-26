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

macro_rules! asn1_sequence {
  ($rs_type:ty, $asn1_ty:expr, $($items:ident),*) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_type() -> tag::Type {
        $asn1_ty.into()
      }
      
      fn asn1_class() -> tag::Class {
      }

      fn asn1_tagnum() -> tag::TagNum {

      }

      fn asn1_constructed() -> bool {
        true
      }
    }
  )
}
