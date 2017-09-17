use syn;
use quote;
use nom::{space, digit};
use std::str::from_utf8;

use asn1_cereal::tag;

// FIXME: Documenation


/// Parse a string as an ASN.1 tag definition.
///
/// IE: /\[ (UNIVERSAL|APPLICATION|CONTEXT|PRIVATE|) [0-9]+ \]/
named!(pub parse_tag<quote::Tokens>, chain!(
  space? ~
  tag: delimited!(
    tag!("["),
    chain!(
      space? ~
      class: alt!(
        tag!("UNIVERSAL") |
        tag!("APPLICATION") |
        tag!("CONTEXT") |
        tag!("PRIVATE")
      )? ~
      space? ~
      tagnum: digit ~
      space? ~
      constructed: tag!("PRIMITIVE")? ~
      space?,
      || {
        let class = match class.map(|e| from_utf8(e).unwrap()).unwrap_or("CONTEXT") {
          "UNIVERSAL" =>   "::asn1_cereal::tag::Class::Universal",
          "APPLICATION" => "::asn1_cereal::tag::Class::Application",
          "CONTEXT" =>     "::asn1_cereal::tag::Class::ContextSpecific",
          "PRIVATE" =>     "::asn1_cereal::tag::Class::Private",
          class => panic!("Unknown class variant {}", class),
        };
        let class_tokens = syn::parse_path(class).unwrap();
        let tagnum: tag::TagNum = from_utf8(tagnum).unwrap().parse().unwrap();
        let constructed = constructed.is_none();
        quote!(
          Some(::asn1_cereal::tag::Tag {
            class: #class_tokens,
            tagnum: #tagnum,
            constructed: #constructed,
          })
        )
      }
    ),
    tag!("]")
  ),
  || tag
));
