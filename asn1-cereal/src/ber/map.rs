use std::collections::BTreeMap;

type Map<S, T> = BTreeMap<S, T>;

enum Value {
  Primitive(Vec<u8>),
  Constructed(Map<::tag::Tag, Value>),
}

impl Value {
  fn is_prim(&self) -> bool {
    if let Value::Primitive(_) = *self {
      true
    } else {
      false
    }
  }

  fn as_prim(&self) -> Option<&[u8]> {
    if let Value::Primitive(ref vec) = *self {
      Some(vec.as_slice())
    } else {
      None
    }
  }

  fn is_constructed(&self) -> bool {
    if let Value::Constructed(_) = *self {
      true
    } else {
      false
    }
  }

  fn as_constructed(&self) -> Option<&Map<::tag::Tag, Value>> {
    if let Value::Constructed(ref map) = *self {
      Some(map)
    } else {
      None
    }
  }
}
