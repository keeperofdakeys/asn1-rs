#[macro_use]
extern crate nom;

use nom::{space, is_alphanumeric};

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

named!(type_name <&[u8], String>, chain!(
  s: take_while!(nom::is_alphanumeric),
  || String::from_utf8(Vec::from(s)).unwrap()
));

named!(type_assignment <&[u8], Asn1Def>, chain!(
  next_token? ~
  name: type_name ~
  next_token ~
  tag!("::=") ~
  next_token ~
  assign: type_name,
  || Asn1Def {
    name: name,
    assign: Asn1Type::Type(assign),
  }
));

named!(type_sequence <&[u8], Asn1Seq>, chain!(
  tag!("SEQUNECE"),
  || Asn1Seq {
    fields: Vec::new(),
  }
));

named!(strip_comments <String>, chain!(
  test: take_until!("--"),
  || String::from_utf8(test.to_owned()).unwrap()
));

named!(strip_comments_2, take_until!("--"));

pub fn is_eol(byte: &u8) -> bool {
  let chr = *byte as char;
  println!("{}", byte);
  chr == '\n' || chr == '\r'
}

named!(next_token <()>, chain!(
  space? ~
  chain!(
    tag!("--") ~
    take_till!(is_eol) ~
    tag!("\n") ~
    space?,
    || ()
  )? ,
  || ()
));

named!(test_ten, chain!(
  next_token ~
  next_token ~
  s: take_while!(nom::is_alphanumeric),
  || s
));

fn main() {
  println!("{:#?}", type_assignment("test -- ::=\n::= hi".as_bytes()));
  println!("{:#?}", String::from_utf8(test_ten("  --fds\ndlsjfs\nfds ::= hi".as_bytes()).unwrap().0.to_owned()).unwrap());
}
