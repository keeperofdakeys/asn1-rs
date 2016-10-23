use quote::Tokens;
use syn;

pub fn ber_sequence_serialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  let fields = if let syn::Body::Struct(ref body) = ast.body {
    match *body {
      syn::VariantData::Struct(ref fields) => fields,
      syn::VariantData::Tuple(ref fields) => fields,
      _ => panic!("Expected a struct with fields, but type {} has no fields", name),
    }
  } else {
    panic!("Expected a struct, but type {} was not a struct", name);
  };

  let actions: Vec<_> = fields.iter().map(|v| {
    let ident = &v.ident.as_ref().expect("Requires named idents");
    quote! {
      let tag = ::asn1_cereal::tag::Tag {
        class: ::asn1_cereal::tag::Class::ContextSpecific,
        tagnum: _count,
        constructed: true,
      };
      _count += 1;

      if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit {
        try!(::asn1_cereal::BerSerialize::serialize_enc(&self.#ident, e, writer));
      } else {
        try!(::asn1_cereal::BerSerialize::serialize_enc(&self.#ident, e, &mut bytes));
        let len: ::asn1_cereal::tag::Len = Some(bytes.len() as ::asn1_cereal::tag::LenNum).into();
        try!(::asn1_cereal::tag::write_taglen(tag, len, writer));
        try!(writer.write_all(&mut bytes));
        bytes.clear();
      }
    }
  }).collect();

  quote! {
    impl #impl_generics ::asn1_cereal::BerSerialize for #name #ty_generics #where_clause {
      fn serialize_value<E: ::asn1_cereal::BerEncRules, W: ::std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), ::asn1_cereal::err::EncodeError> {
        let mut bytes = Vec::new();
        let mut _count = 0u64;

        #( { #actions }; )*
        Ok(())
      }
    }
  }
}


pub fn ber_sequence_deserialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  let fields = if let syn::Body::Struct(ref body) = ast.body {
    match *body {
      syn::VariantData::Struct(ref fields) => fields,
      syn::VariantData::Tuple(ref fields) => fields,
      _ => panic!("Expected a struct with fields, but type {} has no fields", name),
    }
  } else {
    panic!("Expected a struct, but type {} was not a struct", name);
  };

  let build: Vec<_> = fields.iter().map(|v| {
    let ident = &v.ident;
    let f_ident: syn::Ident = format!("field_{}", ident.as_ref().expect("Requires named idents")).into();
    quote! {
      let #f_ident = {
        let this_tag = try!(::asn1_cereal::tag::Tag::read_tag(reader));
        let our_tag = ::asn1_cereal::tag::Tag {
          class: ::asn1_cereal::tag::Class::ContextSpecific,
          tagnum: _count,
          constructed: true,
        };
        _count += 1;

        if E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit && this_tag == our_tag {
          return Err(::asn1_cereal::err::DecodeError::ExplicitTag);
        }

        let len = try!(::asn1_cereal::tag::Len::read_len(reader));

        // If the tag matches our tag, decode the len and call the normal deserialize function.
        if this_tag == our_tag {
          // We don't have anything to do with the len, technically we should use it to
          // check the length decoded.
          let _ = len;
          try!(::asn1_cereal::BerDeserialize::deserialize_enc(e, reader))
        // Otherwise decode it as the inner type. (We give the tag that we
        // decoded, and the function will decode the length itself).
        } else {
          try!(::asn1_cereal::BerDeserialize::deserialize_with_tag(e, reader, this_tag, len))
        }
      };
    }
  }).collect();

  let assignments: Vec<_> = fields.iter().map(|v| {
    let ident = &v.ident;
    let f_ident: syn::Ident = format!("field_{}", ident.as_ref().expect("Requires named idents")).into();
    quote!(#ident: #f_ident)
  }).collect();

  quote! {
    impl #impl_generics ::asn1_cereal::BerDeserialize for #name #ty_generics #where_clause {
      fn deserialize_value<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=::std::io::Result<u8>>>
          (e: E, reader: &mut I, _: ::asn1_cereal::tag::Len) -> Result<Self, ::asn1_cereal::err::DecodeError> {
        let mut _count = 0u64;
        let mut _tag: Option<::asn1_cereal::tag::Tag> = None;

        #( #build )*

        Ok(#name {
          #(#assignments),*
        })
      }
    }
  }
}
