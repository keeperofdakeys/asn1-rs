#[derive(PartialEq, Debug)]
pub enum Asn1Type {
  Type(String),
  Seq(Asn1Seq),
}

#[derive(PartialEq, Debug)]
pub struct Asn1Def {
  pub name: String,
  pub assign: Asn1Type,
}

#[derive(PartialEq, Debug)]
pub struct Asn1Seq {
  pub fields: Vec<Asn1SeqField>,
}

#[derive(PartialEq, Debug)]
pub struct Asn1SeqField {
  pub name: String,
  pub asn1_type: Asn1Type,
}

