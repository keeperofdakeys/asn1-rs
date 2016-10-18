use quote::Tokens;
use syn;

pub fn ber_sequence_of_serialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  quote! {
    impl #impl_generics ::asn1_cereal::BerSerialize for #name #ty_generics #where_clause {
      fn serialize_value<E: ::asn1_cereal::BerEncRules, W: ::std::io::Write>
          (&self, e: E, writer: &mut W) -> Result<(), ::asn1_cereal::err::EncodeError> {
        // Call serialize_enc on each item.
        for item in self {
          try!(::asn1_cereal::BerSerialize::serialize_enc(item, e, writer));
        }
        Ok(())
      }
    }
  }
}


pub fn ber_sequence_of_deserialize(ast: &syn::MacroInput) -> Tokens {
  let name = &ast.ident;
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  quote! {
    impl #impl_generics ::asn1_cereal::BerDeserialize for #name #ty_generics #where_clause {
      fn deserialize_with_tag<E: ::asn1_cereal::BerEncRules, I: Iterator<Item=::std::io::Result<u8>>>
          (e: E, reader: &mut I, tag: ::asn1_cereal::tag::Tag, len: ::asn1_cereal::tag::Len) ->
          Result<Self, ::asn1_cereal::err::DecodeError> {
        struct SeqOfDecoder<T, F, J: Iterator<Item=::std::io::Result<u8>>> {
          len: ::asn1_cereal::tag::Len,
          reader: ::asn1_cereal::byte::ByteReader<J>,
          e: F,
          _p: ::std::marker::PhantomData<T>,
        }

        impl<T, F, J> Iterator for SeqOfDecoder<T, F, J> where
            F: ::asn1_cereal::BerEncRules,
            J: Iterator<Item=::std::io::Result<u8>>,
            T: ::asn1_cereal::BerDeserialize {
          type Item = Result<T, ::asn1_cereal::err::DecodeError>;

          #[inline]
          fn next(&mut self) -> Option<Self::Item> {
            // Compare decoded length with length in tag.
            // Put this first to handle zero-length elements.
            match self.len.partial_cmp(&self.reader.count) {
              // Return an error when we've decoded too much.
              Some(::std::cmp::Ordering::Less) =>
                return Some(Err(::asn1_cereal::err::DecodeError::GreaterLen)),
              // Finish loop when equal, we must be finished.
              Some(::std::cmp::Ordering::Equal) => return None,
              // Continue when we are still decoding, or using
              // indefinite length encoding.
              Some(::std::cmp::Ordering::Greater) | None => {},
            }

            let (tag, len) = match ::asn1_cereal::tag::read_taglen(&mut self.reader) {
              Ok(t) => t,
              Err(e) => return Some(Err(e)),
            };

            // Handle end of indefinite length encoding.
            if tag.tagnum == 0 && tag.class == ::asn1_cereal::tag::Class::Universal &&
               len == ::asn1_cereal::tag::Len::Def(0) {
              return None;
            }

            Some(::asn1_cereal::BerDeserialize::deserialize_value(self.e, &mut self.reader, len))
          }
        }


        if Some(tag) != <Self as ::asn1_cereal::Asn1Info>::asn1_tag() {
          return Err(::asn1_cereal::err::DecodeError::TagTypeMismatch);
        }

        if len == ::asn1_cereal::tag::Len::Indef &&
           E::len_rules() == ::asn1_cereal::ber::enc::LenEnc::Definite {
          return Err(::asn1_cereal::err::DecodeError::IndefiniteLen);
        }

        let mut decoder = SeqOfDecoder {
          e: e,
          len: len.into(),
          reader: ::asn1_cereal::byte::ByteReader::new(reader),
          _p: ::std::marker::PhantomData,
        };
        let v: Result<#name, _> = ::std::iter::FromIterator::from_iter(decoder.by_ref());
        Ok(try!(v))
      }
    }
  }
}
