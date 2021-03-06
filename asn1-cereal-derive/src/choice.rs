use quote::Tokens;
use syn;

// FIXME: Documenation

pub fn ber_choice_serialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  let fields = if let syn::Body::Enum(ref fields) = ast.body {
    fields
  } else {
    panic!("Expected an enum, but type {} was not an enum", name);
  };

  let match_pattern: Vec<_> = fields.iter().map(|v| {
    let ident = &v.ident;
    quote!(#name::#ident(ref item))
  }).collect();
  let match_action: Vec<_> = fields.iter().map(|_v| {
    // TODO: Handle asn1 attributes
    quote! {
      if false {
        // let mut bytes = Vec::new();
        // try!(item.serialize_enc(e, &mut bytes));
        // try!(tag.write_tag(writer));
        // let len: ::asn1_cereal::tag::Len = Some(bytes.len() as ::asn1_cereal::tag::LenNum).into();
        // try!(writer.write_all(&mut bytes));
      } else {
        // FIXME: Does implicit tagging + CHOICE mean always explicit tag?
        // Maybe not with context-specific tags?
        // FIXME: We should be conditionally setting the constructed flag.
        // if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit {
        //   try!(item.serialize_value(e, writer));
        // } else {
          try!(item.serialize_enc(e, writer));
        // }
      }
    }
  }).collect();

  quote! {
    impl #impl_generics ::asn1_cereal::BerSerialize for #name #ty_generics #where_clause {
      fn serialize_value<E: ::asn1_cereal::BerEncRules, W: ::std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), ::asn1_cereal::err::EncodeError> {
        match *self {
          #(#match_pattern => #match_action),*
        }
        Ok(())
      }
    }
  }
}

pub fn ber_choice_deserialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  let fields = if let syn::Body::Enum(ref fields) = ast.body {
    fields
  } else {
    panic!("Expected an enum, but type {} was not an enum", name);
  };

  let match_pattern: Vec<_> = fields.iter().map(|v| {
    let ident = &v.ident;
    let inner = if let syn::VariantData::Tuple(ref fields) = v.data {
      fields[0].clone()
    } else {
      panic!("Expected a tuple, but type {} variant {} was not a tuple", name, ident);
    };
    // TODO: Handle asn1 attributes
    quote! {
      tag @ ::asn1_cereal::tag::Tag { .. }
        if tag == <#inner as ::asn1_cereal::Asn1Info>::asn1_tag()
                    .expect("All types used for CHOICE must have a defined tag")
    }
  }).collect();

  let match_action: Vec<_> = fields.iter().map(|v| {
    let ident = &v.ident;
    // TODO: Handle asn1 attributes
    quote! {
      Ok(#name::#ident(
        // FIXME: Does implicit tagging + CHOICE mean always explicit tag?
        // Maybe not with context-specific tags?
        // FIXME: We should be conditionally setting the constructed flag.
        // if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit {
        //   try!(::asn1_cereal::BerDeserialize::deserialize_value(e, reader, len))
        // } else {
        try!(::asn1_cereal::BerDeserialize::deserialize_with_tag(e, reader, tag, len))
        // }
      ))
    }
  }).collect();

  quote! {
    impl #impl_generics ::asn1_cereal::BerDeserialize for #name #ty_generics #where_clause {
      fn _deserialize_with_tag<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=::std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: ::asn1_cereal::tag::Tag, len: ::asn1_cereal::tag::Len) ->
          Option<Result<Self, ::asn1_cereal::err::DecodeError>> {
        let mut res = ||
          match tag {
            #(#match_pattern => #match_action),*,
            _ => panic!("Choice: Unknown tag {}", tag),
          };
        Some(res())
      }

      fn deserialize_value<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=::std::io::Result<u8>>>
          (e: E, reader: &mut I, _len: ::asn1_cereal::tag::Len) ->
          Result<Self, ::asn1_cereal::err::DecodeError> {
        let (tag, len) = ::asn1_cereal::tag::read_taglen(reader)?;
        Self::_deserialize_with_tag(e, reader, tag, len).unwrap()
      }
    }
  }
}
