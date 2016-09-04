pub mod traits;
pub mod int;
pub mod prim;
pub mod seq;
pub mod str;
pub mod assign;

pub use self::traits::{Asn1Serialize, Asn1Deserialize, Asn1Info};

#[macro_export]
/// This macro is a compact way of defining asn1_info for a type.
macro_rules! asn1_info {
  ($rs_type:ty, $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
    impl $crate::serial::traits::Asn1Info for $rs_type {
      fn asn1_tag() -> $crate::tag::Tag {
        $crate::tag::Tag {
          class: ($class as u8).into(),
          tagnum: $tagnum as $crate::tag::TagNum,
          constructed: $constructed,

        }
      }

      fn asn1_type() -> $crate::tag::Type {
        $crate::tag::Type::from($asn1_ty)
      }
    }
  )
}

