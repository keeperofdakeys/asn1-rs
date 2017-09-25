use syn;

/// Parsed asn1 properties of a field.
pub struct Field {
  pub tag: Option<::quote::Tokens>,
  pub optional: bool,
  pub default: bool,
}

impl Field {
  /// Parse a collection of attributes from a field.
  pub fn parse<'a, I: IntoIterator<Item=&'a syn::Attribute>>(attrs: I)
      -> Field {
    let mut tag = None;
    let mut optional = false;
    let mut default = false;

    for attr in attrs.into_iter().find(|e| e.name() == "asn1") {
      let items = if let syn::MetaItem::List(_, ref items) = attr.value {
        items
      } else {
        continue;
      };

      for item in items {
        if let syn::NestedMetaItem::MetaItem(ref item) = *item {
          match *item {
            syn::MetaItem::Word(ref ident)
              if ident == "optional" =>
                optional = true,
            syn::MetaItem::Word(ref ident)
              if ident == "default" =>
                default = true,
            syn::MetaItem::NameValue(ref ident, syn::Lit::Str(ref value, _))
              if ident == "tag" => {
                let _tag = ::tag::parse_tag(value.as_bytes());
                if !_tag.is_done() {
                  panic!("Failed to parse tag");
                }
                tag = Some(_tag.unwrap().1)
              },
            _ => (),
          };
        }
      }
    }

    Field {
      tag: tag,
      optional: optional,
      default: default,
    }
  }
}
