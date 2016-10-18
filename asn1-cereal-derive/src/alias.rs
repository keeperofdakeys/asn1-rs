use proc_macro::TokenStream;
use quote::Tokens;
use syn;

use logging_enabled;

pub fn ber_alias_serialize(ast: syn::MacroInput) -> TokenStream {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
  let logging = logging_enabled(&ast);

  let mut implicit_msg = Tokens::new();

  if logging {
    implicit_msg = quote!(debug!("Skipping encoding of implicit tag");)
  }

  let expanded = quote! {
    #ast

    impl #impl_generics ::asn1_cereal::BerSerialize for #name #ty_generics #where_clause {
      fn _serialize_enc<E: ::asn1_cereal::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Option<Result<(), ::asn1_cereal::err::EncodeError>> {
        let tag = <Self as ::asn1_cereal::Asn1Info>::asn1_tag();
        // If we have a tag, and encoding uses implicit tags, skip our tag.
        if tag.is_some() && E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit {
          #implicit_msg
          Some(self.serialize_value(e, writer))
        } else {
          None
        }
      }

      fn serialize_value<E: ::asn1_cereal::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), ::asn1_cereal::err::EncodeError> {
        self.0.serialize_enc(e, writer)
      }
    }
  };
  expanded.to_string().parse().expect("Failure parsing derived impl")
}

pub fn ber_alias_deserialize(ast: syn::MacroInput) -> TokenStream {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
  let logging = logging_enabled(&ast);

  let mut no_tag_msg = Tokens::new();
  let mut implicit_msg = Tokens::new();

  if logging {
    no_tag_msg = quote!(debug!("Skipping decoding of empty tag"););
    implicit_msg = quote!(debug!("Skipping decoding of implicit tag"););
  }

  let expanded = quote! {
    #ast

    impl #impl_generics ::asn1_cereal::BerDeserialize for #name #ty_generics #where_clause {
      fn _deserialize_with_tag<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: ::asn1_cereal::tag::Tag, len: ::asn1_cereal::tag::Len)
          -> Option<Result<Self, ::asn1_cereal::err::DecodeError>> {
        // If we don't have a tag, decode the internal item.
        let my_tag = match <Self as ::asn1_cereal::Asn1Info>::asn1_tag() {
          Some(tag) => tag,
          None => {
            #no_tag_msg
            return Some(
              ::asn1_cereal::BerDeserialize::deserialize_with_tag(e, reader, tag, len)
              .and_then(|s| Ok(#name(s)))
            )
          },
        };

        // If we're decoding using Implicit tagging rules, throw an error if this isn't an implicit tag.
        if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit && tag == my_tag {
          return Some(Err(::asn1_cereal::err::DecodeError::ExplicitTag));
        }

        if tag != my_tag {
          #implicit_msg
          let res =
            ::asn1_cereal::BerDeserialize::deserialize_with_tag(e, reader, tag, len)
            .and_then(|s| Ok(#name(s)));
          Some(res)
        } else {
          None
        }
      }

      fn deserialize_value<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, _len: ::asn1_cereal::tag::Len) -> Result<Self, ::asn1_cereal::err::DecodeError> {
        Ok(#name(try!(::asn1_cereal::BerDeserialize::deserialize_enc(e, reader))))
      }
    }
  };
  expanded.to_string().parse().expect("Failure parsing derived impl")
}
