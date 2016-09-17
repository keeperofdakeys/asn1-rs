use tag;

/// Provides ASN.1 information about a Rust type, including the BER tag and ASN.1 type.
pub trait Asn1Info {
  /// Get the ASN.1 tag for this Rust type.
  fn asn1_tag() -> tag::Tag;

  /// Get the ASN.1 type for this Rust type.
  fn asn1_type() -> tag::Type;
}

#[macro_export]
/// This macro defines the Asn1Info trait for a rust type.
///
/// This information is used to match tag information during deserialization,
/// so it should match the expected values in the ASN.1 stream.
macro_rules! asn1_info {
  (impl: $rs_type:ty, $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
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
  );
  ($rs_type:ty => $gen:ident, $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
    impl<$gen> $crate::Asn1Info for $rs_type {
      asn1_info!{impl: $rs_type, $class, $tagnum, $constructed, $asn1_ty}
    }
  );
  ($rs_type:ty, $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
    impl $crate::Asn1Info for $rs_type {
      asn1_info!{impl: $rs_type, $class, $tagnum, $constructed, $asn1_ty}
    }
  );
}
