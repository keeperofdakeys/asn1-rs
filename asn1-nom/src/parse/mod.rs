pub mod space;
pub mod seq;

use nom::{space,is_alphanumeric};
use parse::space::{skip_other};
use parse::seq::asn1_seq;

named!(pub asn1_type_name <String>, chain!(
  s: take_while!(is_alphanumeric),
  || String::from_utf8(Vec::from(s)).unwrap()
));

named!(pub asn1_type_def <::Asn1Def>, chain!(
  skip_other? ~
  name: asn1_type_name ~
  space? ~
  tag!("::=") ~
  space? ~
  asn1_type: asn1_type,
  || $crate::Asn1Def {
    name: name,
    assign: asn1_type,
  }
));

named!(pub asn1_type <::Asn1Type>, alt!(
  chain!(s: asn1_seq, || $crate::Asn1Type::Seq(s)) |
  chain!(t: asn1_assignment, || $crate::Asn1Type::Type(t))
));

named!(pub asn1_assignment <String>, chain!(
  t: asn1_type_name,
  || t
));
