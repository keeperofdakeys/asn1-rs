use tag;

/// Provides ASN.1 information about a Rust type, including the BER tag and ASN.1 type.
pub trait Asn1Info {
  /// Get the ASN.1 tag (if defined) for this Rust type. Some types don't have a tag, eg. CHOICE.
  fn asn1_tag() -> Option<tag::Tag>;

  /// Get the ASN.1 type for this Rust type.
  fn asn1_type() -> tag::Type;
}

#[macro_export]
/// This macro defines the Asn1Info trait for a rust type.
///
/// This information is used to match tag information during deserialization,
/// so it should match the expected values in the ASN.1 stream.
///
/// ```
/// #[macro_use] extern crate asn1_cereal; fn main() {
/// // For A ::= [APPLICATION 3] u64
/// struct A(u64);
/// asn1_info!(A, [APPLICATION 3], "A");
///
/// // For B ::= [PRIVATE 3] u64
/// struct B(u64);
/// asn1_info!(B, [PRIVATE 3], "B");
///
/// // For a primitive type, with application tag 3.
/// struct C(u64);
/// // (The false here sets the constructed flag).
/// asn1_info!(C, asn1_cereal::tag::Class::Application, 3, false, "C");
/// }
/// ```
macro_rules! asn1_info {
  ($rs_type:ty => $gen:ident, $($args:tt)*) => (
    impl<$gen> $crate::Asn1Info for $rs_type {
      asn1_info!{__impl $($args)*}
    }
  );
  (__impl [$($args:tt)*], $asn1_ty:expr) => (
    fn asn1_tag() -> Option<$crate::tag::Tag> {
      Some(asn1_spec_tag!([$($args)*]))
    }
    asn1_info!(__type $asn1_ty);
  );
  (__impl $class:expr, $tagnum:expr, $constructed:expr, $asn1_ty:expr) => (
    fn asn1_tag() -> Option<$crate::tag::Tag> {
      Some($crate::tag::Tag {
        class: ($class as u8).into(),
        tagnum: $tagnum as $crate::tag::TagNum,
        constructed: $constructed,
      })
    }
    asn1_info!(__type $asn1_ty);
  );
  (__impl $asn1_ty:expr) => (
    fn asn1_tag() -> Option<$crate::tag::Tag> {
      None
    }
    asn1_info!(__type $asn1_ty);
  );
  (__type $asn1_ty:expr) => (
    fn asn1_type() -> $crate::tag::Type {
      $crate::tag::Type::from($asn1_ty)
    }
  );
  ($rs_type:ty, $($args:tt)*) => (
    impl $crate::Asn1Info for $rs_type {
      asn1_info!{__impl $($args)*}
    }
  );
}

#[macro_export]
/// This macro parses an ASN.1 tag specification, and returns the appropriate Tag.
macro_rules! asn1_spec_tag {
  ({ $count:ident }) => (
    asn1_spec_tag!([])
  );
  ({ $count:ident } []) => ({
    let _count = $count;
    $count += 1;
    asn1_spec_tag!([CONTEXT _count])
  });
  ({ $count:ident } [$($args:tt)*]) => (
    asn1_spec_tag!([$($args)*])
  );
  ([$tagnum:expr]) => (
    asn1_spec_tag!([CONTEXT $tagnum]);
  );
  ([UNIVERSAL $tagnum:expr]) => (
    $crate::tag::Tag {
      class: $crate::tag::Class::Universal,
      tagnum: $tagnum,
      constructed: true,
    }
  );
  ([CONTEXT $tagnum:expr]) => (
    $crate::tag::Tag {
      class: $crate::tag::Class::ContextSpecific,
      tagnum: $tagnum,
      constructed: true,
    }
  );
  ([APPLICATION $tagnum:expr]) => (
    $crate::tag::Tag {
      class: $crate::tag::Class::Application,
      tagnum: $tagnum,
      constructed: true,
    }
  );
  ([PRIVATE $tagnum:expr]) => (
    $crate::tag::Tag {
      class: $crate::tag::Class::Private,
      tagnum: $tagnum,
      constructed: true,
    }
  );
}
