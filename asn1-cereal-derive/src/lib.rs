// #![feature(trace_macros)]
#![recursion_limit = "256"]
// trace_macros!(true);


extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate asn1_cereal;
#[macro_use]
extern crate nom;

use proc_macro::TokenStream;

use ::tag::parse_tag;
use ::alias::{ber_alias_serialize, ber_alias_deserialize};
use ::seq_of::{ber_sequence_of_serialize, ber_sequence_of_deserialize};
use ::seq::{ber_sequence_serialize, ber_sequence_deserialize};
use ::choice::{ber_choice_serialize, ber_choice_deserialize};

mod tag;
mod seq;
mod seq_of;
mod alias;
mod choice;

#[proc_macro_derive(Asn1Info, attributes(asn1))]
pub fn asn1_info(input: TokenStream) -> TokenStream {
  let source = input.to_string();
  let ast = syn::parse_macro_input(&source).expect("Couldn't parse input TokenSteam into AST");

  // Tag and asn1 type for this rust type.
  let mut tag = quote!(None);
  let mut asn1_type = ast.ident.as_ref().to_owned();
  let mut _logging = false;

  // Parse attributes.
  for attr in &ast.attrs.iter().find(|e| e.name() == "asn1") {
    if let syn::MetaItem::List(_, ref items) = attr.value {
      for item in items {
        match *item {
          syn::MetaItem::Word(ref _ident) if _ident == "log" => _logging = true,
          syn::MetaItem::NameValue(ref _name, syn::Lit::Str(ref value, _)) => {
            let name: &str = _name.as_ref();
            match name {
              "tag" => tag = parse_tag(value.as_bytes()).unwrap().1,
              "asn1_type" => asn1_type = value.clone(),
              _ => (),
            };
          },
          _ => (),
        };
      }
    }
  }

  // Used in the quasi-quotation below as `#name`
  let name = &ast.ident;

  // Helper is provided for handling complex generic types correctly and effortlessly
  let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

  // FIXME: We need the encoding settings here.
  let alias_constructed = quote! {
    fn asn1_constructed() -> bool {
    }
  };

  let derived = quote! {
    impl #impl_generics ::asn1_cereal::Asn1Info for #name #ty_generics #where_clause {
      fn asn1_tag() -> Option<::asn1_cereal::tag::Tag> {
        #tag
      }

      fn asn1_type() -> ::asn1_cereal::tag::Type {
        #asn1_type.to_owned()
      }
    }
  };

  derived.to_string().parse().expect("Failure parsing derived impl")
}

fn logging_enabled(ast: &syn::MacroInput) -> bool {
  // Parse attributes.
  for attr in &ast.attrs.iter().find(|e| e.name() == "asn1") {
    if let syn::MetaItem::List(_, ref items) = attr.value {
      for item in items {
        match *item {
          syn::MetaItem::Word(ref _ident) if _ident == "log" => return true,
          _ => (),
        };
      }
    }
  }
  false
}

#[proc_macro_derive(BerSerialize, attributes(asn1))]
pub fn ber_serialize(input: TokenStream) -> TokenStream {
  let source = input.to_string();
  let ast = syn::parse_macro_input(&source).unwrap();

  let body = ast.body.clone();
  let mut form = None;

  for attr in &ast.attrs.iter().find(|e| e.name() == "asn1") {
    if let syn::MetaItem::List(_, ref items) = attr.value {
      for item in items {
        match *item {
          syn::MetaItem::NameValue(ref _name, syn::Lit::Str(ref value, _)) => {
            let name: &str = _name.as_ref();
            match name {
              "form" => form = Some(value.clone()),
              _ => (),
            };
          },
          _ => (),
        };
      }
    }
  }
  
  let derived = if let Some(form) = form {
    match form.as_str() {
      "seq of" | "sequence of" | "set of" => ber_sequence_of_serialize(&ast),
      "alias" => ber_alias_serialize(&ast),
      "choice" => ber_choice_serialize(&ast),
      "seq" | "sequence" => ber_sequence_serialize(&ast),
      _ => panic!("Unknown serialize form {}", form),
    }
  } else {
    match body {
      syn::Body::Enum(_) => {
        ber_choice_serialize(&ast)
      },
      syn::Body::Struct(syn::VariantData::Tuple(fields)) => {
        if fields.len() == 1 {
          ber_alias_serialize(&ast)
        } else {
          ber_sequence_serialize(&ast)
        }
      },
      syn::Body::Struct(syn::VariantData::Struct(_fields)) => {
        ber_sequence_serialize(&ast)
      },
      _ => unimplemented!(),
    }
  };

  derived.to_string().parse().expect("Failure parsing derived impl")
}

#[proc_macro_derive(BerDeserialize, attributes(asn1))]
pub fn ber_deserialize(input: TokenStream) -> TokenStream {
  let source = input.to_string();
  let ast = syn::parse_macro_input(&source).expect("Couldn't parse input TokenSteam into AST");

  let body = ast.body.clone();
  let mut form = None;

  for attr in &ast.attrs.iter().find(|e| e.name() == "asn1") {
    if let syn::MetaItem::List(_, ref items) = attr.value {
      for item in items {
        match *item {
          syn::MetaItem::NameValue(ref _name, syn::Lit::Str(ref value, _)) => {
            let name: &str = _name.as_ref();
            match name {
              "form" => form = Some(value.clone()),
              _ => (),
            };
          },
          _ => (),
        };
      }
    }
  }
  
  let derived = if let Some(form) = form {
    match form.as_str() {
      "seq of" | "sequence of" | "set of" => ber_sequence_of_deserialize(&ast),
      "alias" => ber_alias_deserialize(&ast),
      "choice" => ber_choice_deserialize(&ast),
      "seq" | "sequence" => ber_sequence_deserialize(&ast),
      _ => panic!("Unknown deserialize form {}", form),
    }
  } else {
    match body {
      syn::Body::Enum(_) => {
        ber_choice_deserialize(&ast)
      },
      syn::Body::Struct(syn::VariantData::Tuple(fields)) => {
        if fields.len() == 1 {
          ber_alias_deserialize(&ast)
        } else {
          ber_sequence_deserialize(&ast)
        }
      },
      syn::Body::Struct(syn::VariantData::Struct(_fields)) => {
        ber_sequence_deserialize(&ast)
      },
      _ => unimplemented!(),
    }
  };

  derived.to_string().parse().expect("Failure parsing derived impl")
}

#[test]
fn test() {
}
