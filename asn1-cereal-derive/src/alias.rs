use quote::Tokens;
use syn;

use logging_enabled;

// FIXME: Documenation

pub fn asn1_alias_info_constructed(ast: &syn::MacroInput) -> Tokens {
  let fields = if let syn::Body::Struct(ref body) = ast.body {
    match *body {
      syn::VariantData::Tuple(ref fields)
        if fields.len() == 1 => fields,
      _ => panic!("Expected a tuple struct with 1 fields for alias"),
    }
  } else {
    panic!("Expected a struct for alias");
  };
  let inner_ty = &fields[0].ty;

  quote! {
    fn asn1_constructed<E: ::asn1_cereal::BerEncRules>(e: E) -> bool {
      let tag = Self::asn1_tag();
      if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit ||
         tag.is_none() {

        <#inner_ty as ::asn1_cereal::Asn1Info>::asn1_constructed(e)
      } else {
        tag.map_or(false, |t| t.constructed)
      }
    }
  }
}

pub fn ber_alias_serialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
  let logging = logging_enabled(&ast);

  let mut implicit_msg = Tokens::new();

  if logging {
    implicit_msg = quote!(debug!("Skipping encoding of implicit tag");)
  }

  quote! {
    impl #impl_generics ::asn1_cereal::BerSerialize for #name #ty_generics #where_clause {
      fn serialize_value<E: ::asn1_cereal::BerEncRules, W: std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), ::asn1_cereal::err::EncodeError> {

        // FIXME: We should be conditionally setting the constructed flag.

        if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit {
          #implicit_msg
          self.0.serialize_value(e, writer)
        }  else {
          self.0.serialize_enc(e, writer)
        }
      }
    }
  }
}

pub fn ber_alias_deserialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
  let logging = logging_enabled(&ast);

  let mut implicit_msg = Tokens::new();

  if logging {
    implicit_msg = quote!(debug!("Skipping decoding of implicit tag"););
  }

  quote! {
    impl #impl_generics ::asn1_cereal::BerDeserialize for #name #ty_generics #where_clause {
      fn deserialize_value<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=std::io::Result<u8>>>
          (e: E, reader: &mut I, len: ::asn1_cereal::tag::Len) -> Result<Self, ::asn1_cereal::err::DecodeError> {
        if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit {
          #implicit_msg
          Ok(#name(try!(::asn1_cereal::BerDeserialize::deserialize_value(e, reader, len))))
        }  else {
          Ok(#name(try!(::asn1_cereal::BerDeserialize::deserialize_enc(e, reader))))
        }
      }
    }
  }
}
