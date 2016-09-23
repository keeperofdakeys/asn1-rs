#[macro_use]
extern crate nom;

use nom::{space,multispace,is_alphanumeric,eol};

// LDAPString ::= OCTET STRING -- UTF-8 encoded,
//                             -- [ISO10646] characters

#[derive(Debug)]
enum Asn1Type {
  Type(String),
  Seq(Asn1Seq),
}

#[derive(Debug)]
struct Asn1Def {
  name: String,
  assign: Asn1Type,
}

#[derive(Debug)]
struct Asn1Seq {
  fields: Vec<Asn1Def>,
}

named!(asn1_type_name <String>, chain!(
  s: take_while!(nom::is_alphanumeric),
  || String::from_utf8(Vec::from(s)).unwrap()
));

named!(asn1_type_def <Asn1Def>, chain!(
  skip_other? ~
  multispace? ~
  name: asn1_type_name ~
  multispace? ~
  tag!("::=") ~
  multispace? ~
  asn1_type: asn1_type,
  || Asn1Def {
    name: name,
    assign: asn1_type,
  }
));

named!(asn1_type <Asn1Type>, alt!(
  chain!(s: asn1_sequence, || Asn1Type::Seq(s)) |
  chain!(t: asn1_assignment, || Asn1Type::Type(t))
));

named!(asn1_assignment <String>, chain!(
  t: asn1_type_name,
  || t
));

named!(asn1_sequence <&[u8], Asn1Seq>, chain!(
  tag!("SEQUNECE"),
  || Asn1Seq {
    fields: Vec::new(),
  }
));

pub fn is_eol(byte: &u8) -> bool {
  let chr = *byte as char;
  println!("{}", byte);
  chr == '\n' || chr == '\r'
}

named!(comment <()>, chain!(
  complete!(tag!("--")) ~
  take_till!(is_eol) ~
  eol,
  || ()
));

named!(skip_other <()>, chain!(
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

named!(test_ten, chain!(
  skip_other ~
  s: take_while!(nom::is_alphanumeric),
  || s
));

fn main() {
  println!("{:#?}", asn1_type_def("--\ntest ::=  hi".as_bytes()));
  println!("{:#?}", String::from_utf8(test_ten(" --fds\n --\ndlsjfs\nfds ::= hi".as_bytes()).unwrap().0.to_owned()).unwrap());
}
