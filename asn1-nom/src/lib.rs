#[macro_use]
extern crate nom;

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
  opt!(nom::multispace) ~
  name: type_name ~
  take_until_and_consume!("::=") ~
  opt!(nom::multispace) ~
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

fn main() {
  println!("{:#?}", type_assignment("test ::= hi".as_bytes()));
}
