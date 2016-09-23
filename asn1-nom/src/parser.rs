use nom::{space,multispace,is_alphanumeric,eol};

named!(pub asn1_type_name <String>, chain!(
  s: take_while!(is_alphanumeric),
  || String::from_utf8(Vec::from(s)).unwrap()
));

named!(pub asn1_type_def <::Asn1Def>, chain!(
  skip_other? ~
  multispace? ~
  name: asn1_type_name ~
  multispace? ~
  tag!("::=") ~
  multispace? ~
  asn1_type: asn1_type,
  || $crate::Asn1Def {
    name: name,
    assign: asn1_type,
  }
));

named!(pub asn1_type <::Asn1Type>, alt!(
  chain!(s: asn1_sequence, || $crate::Asn1Type::Seq(s)) |
  chain!(t: asn1_assignment, || $crate::Asn1Type::Type(t))
));

named!(pub asn1_assignment <String>, chain!(
  t: asn1_type_name,
  || t
));

named!(pub asn1_sequence <&[u8], ::Asn1Seq>, chain!(
  tag!("SEQUNECE"),
  || $crate::Asn1Seq {
    fields: Vec::new(),
  }
));

pub fn is_eol(byte: &u8) -> bool {
  let chr = *byte as char;
  println!("{}", byte);
  chr == '\n' || chr == '\r'
}

named!(pub comment <()>, chain!(
  complete!(tag!("--")) ~
  take_till!(is_eol) ~
  eol,
  || ()
));

named!(pub skip_other <()>, chain!(
  multispace? ~
  comment? ~
  multispace? ~
  chain!(
     complete!(peek!(tag!("--"))) ~
     complete!(skip_other),
     || ()
  )?,
  || ()
));
