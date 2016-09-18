#[macro_use]
extern crate nom;

// LDAPString ::= OCTET STRING -- UTF-8 encoded,
//                             -- [ISO10646] characters

#[derive(Debug)]
struct Asn1Type {
  name: String,
  assign: String
}

named!(type_name <&[u8], String>, chain!(
  s: take_while!(nom::is_alphanumeric),
  || String::from_utf8(Vec::from(s)).unwrap()
));

named!(type_assignment <&[u8], Asn1Type>, chain!(
  opt!(nom::multispace) ~
  name: type_name ~
  take_until_and_consume!("::=") ~
  opt!(nom::multispace) ~
  assign: type_name,
  || Asn1Type {
    name: name,
    assign: assign,
  }
));

fn main() {
  println!("{:#?}", type_assignment("test ::= hi".as_bytes()));
}
