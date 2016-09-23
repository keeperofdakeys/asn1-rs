#[macro_use]
extern crate nom;

pub mod parser;
mod data;

pub use data::{Asn1Type, Asn1Def, Asn1Seq};

fn main() {
  println!("{:#?}", parser::asn1_type_def("--\ntest ::=  hi".as_bytes()));
}
