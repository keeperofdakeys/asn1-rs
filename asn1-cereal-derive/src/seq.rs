use quote::Tokens;
use syn;

// FIXME: Documenation

fn field_is_optional(attrs: &[syn::Attribute]) -> bool {
  for attr in attrs.iter().find(|e| e.name() == "asn1") {
    if let syn::MetaItem::List(_, ref items) = attr.value {
      for item in items {
        if let syn::NestedMetaItem::MetaItem(ref item) = *item {
          match *item {
            syn::MetaItem::Word(ref _ident)
              if _ident == "optional"
                => return true,
            _ => (),
          };
        }
      }
    }
  }
  return false;
}

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
    let ty = &v.ty;
    let tag_encode = quote!(
      if is_implicit {
        try!(::asn1_cereal::BerSerialize::serialize_value(value, e, &mut bytes));
      } else {
        try!(::asn1_cereal::BerSerialize::serialize_enc(value, e, &mut bytes));
      }

      let len: ::asn1_cereal::tag::Len = Some(bytes.len() as ::asn1_cereal::tag::LenNum).into();
      try!(::asn1_cereal::tag::write_taglen(tag, len, writer));
      try!(writer.write_all(&mut bytes));
      bytes.clear();
    );
    let encode = if field_is_optional(&v.attrs) {
      quote!(
        if let &Some(ref value) = &self.#ident {
          #tag_encode
        }
      )
    } else {
      quote!(
        let value = &self.#ident;
        #tag_encode
      )
    };
    quote! {
      let is_implicit =
        E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit;
      let tag = ::asn1_cereal::tag::Tag {
        class: ::asn1_cereal::tag::Class::ContextSpecific,
        tagnum: _count,
        constructed:
          if is_implicit {
            <#ty as ::asn1_cereal::Asn1Info>::asn1_constructed(e)
          } else {
            true
          },
      };

      _count += 1;

      #encode
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
    let ty = &v.ty;
    let tag_decode = quote!(
      {
        _tag = None;
        let len = try!(::asn1_cereal::tag::Len::read_len(reader));
        // If we are decoding with an implicit tag, deserialize value directly.
        if is_implicit {
          try!(::asn1_cereal::BerDeserialize::deserialize_value(e, reader, len))
        } else {
          try!(::asn1_cereal::BerDeserialize::deserialize_enc(e, reader))
        }
      }
    );
    // TODO: Actually implement this, it's OPTIONAL, but the no match
    // case is the default value.
    // let default = quote!(
    //   if this_tag == our_tag {
    //     #tag_decode
    //   } else {
    //
    //   }
    // );
    let decode = if field_is_optional(&v.attrs) {
      quote!(
        if this_tag == our_tag {
          Some(
            #tag_decode
          )
        } else {
          None
        }
      )
    } else {
      quote!(
        if this_tag != our_tag {
          return Err(::asn1_cereal::err::DecodeError::TagTypeMismatch);
        }
        #tag_decode
      )
    };
    // TODO: Add lenth check using ByteReader. We need to check our decoded
    // length, otherwise an OPTIONAL or DEFAULT as the final field means we'll
    // look beyond our element.
    // TODO: Make context-specific tags optional, implement default
    quote! {
      let #f_ident = {
        let this_tag = match _tag {
          Some(t) => t,
          None => {
            let t = ::asn1_cereal::tag::Tag::read_tag(reader)?;
            _tag = Some(t);
            t
          }
        };

        let is_implicit =
          E::tag_rules() == ::asn1_cereal::ber::enc::TagEnc::Implicit;
        // TODO: Handle entries without a context specific tag.
        let our_tag = ::asn1_cereal::tag::Tag {
          class: ::asn1_cereal::tag::Class::ContextSpecific,
          tagnum: _count,
          constructed:
            if is_implicit {
              <#ty as ::asn1_cereal::Asn1Info>::asn1_constructed(e)
            } else {
              true
            },
        };
        _count += 1;

        #decode
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
          (e: E, reader: &mut I, len: ::asn1_cereal::tag::Len) -> Result<Self, ::asn1_cereal::err::DecodeError> {
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
