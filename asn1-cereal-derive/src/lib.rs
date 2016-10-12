#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate asn1_cereal;
#[macro_use]
extern crate nom;

mod seq;

use proc_macro::TokenStream;
use syn::{MetaItem, Lit};
use nom::{space, digit};
use std::str::from_utf8;
use asn1_cereal::tag;

/// Parse a string as an ASN.1 tag definition.
///
/// IE: /\[ \(UNIVERSAL|APPLICATION|CONTEXT|PRIVATE|) [0-9]+ \]/
named!(parse_tag<quote::Tokens>, chain!(
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
        let mut class_tokens = quote::Tokens::new();
        class_tokens.append(class);
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

#[proc_macro_derive(Asn1Info)]
pub fn asn1_info(input: TokenStream) -> TokenStream {
  let source = input.to_string();
  let ast = syn::parse_macro_input(&source).unwrap();

  // Tag and asn1 type for this rust type.
  let mut tag = quote!(None);
  let mut asn1_type = ast.ident.as_ref().to_owned();

  // Parse attributes.
  for attr in &ast.attrs {
    if attr.name() != "asn1" { continue; }
    if let MetaItem::List(_, ref items) = attr.value {
      for item in items {
        match *item {
          MetaItem::NameValue(ref _name, Lit::Str(ref value, _)) => {
            let name: &str = _name.as_ref();
            match name {
              "tag" => tag = parse_tag(value.as_bytes()).unwrap().1,
              "asn1_type" => asn1_type = value.clone(),
              _ => (),
            };
          }
          _ => (),
        };
      }
    }
  }

  // Used in the quasi-quotation below as `#name`
  let name = &ast.ident;

  // Helper is provided for handling complex generic types correctly and effortlessly
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  let expanded = quote! {
    // Preserve the input struct unmodified
    #ast

    // The generated impl
    impl #impl_generics ::asn1_cereal::Asn1Info for #name #ty_generics #where_clause {
      fn asn1_tag() -> Option<::asn1_cereal::tag::Tag> {
        #tag
      }

      fn asn1_type() -> ::asn1_cereal::tag::Type {
        #asn1_type.to_owned()
      }
    }
  };
  expanded.to_string().parse().unwrap()
}
