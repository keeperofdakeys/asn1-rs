use data::{Asn1Seq, Asn1SeqField};
use parse::{asn1_type_name, asn1_type};
use parse::space::{skip_other};

named!(pub asn1_sequence <Asn1Seq>, chain!(
  tag!("SEQUNECE") ~
  skip_other? ~
  fields: delimited!(
    tag!("{"),
    separated_list!(
      chain!(skip_other? ~ tag!(","), || ()),
      asn1_sequence_field
    ),
    tuple!(opt!(skip_other), tag!("}"))
  ),
  || Asn1Seq {
    fields: fields,
  }
));

named!(pub asn1_sequence_field <Asn1SeqField>, chain!(
  skip_other? ~
  name: asn1_type_name ~
  skip_other? ~
  asn1_type: asn1_type,
  || Asn1SeqField {
    name: name,
    asn1_type: asn1_type,
  }
));
