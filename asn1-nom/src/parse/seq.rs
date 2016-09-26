use data::{Asn1Seq, Asn1SeqField};
use parse::{asn1_type_name, asn1_type};
use parse::space::{skip_other};

named!(pub asn1_seq_field <Asn1SeqField>, chain!(
  skip_other? ~
  name: asn1_type_name ~
  skip_other? ~
  asn1_type: asn1_type,
  || Asn1SeqField {
    name: name,
    asn1_type: asn1_type,
  }
));

named!(pub asn1_seq <Asn1Seq>, chain!(
  tag!("SEQUENCE") ~
  skip_other? ~
  fields: delimited!(
    tag!("{"),
    separated_list!(
      chain!(skip_other? ~ tag!(","), || ()),
      asn1_seq_field
    ),
    tuple!(opt!(skip_other), tag!("}"))
  ),
  || Asn1Seq {
    fields: fields,
  }
));

#[test]
fn test_asn1_sequence_field() {
  let field1 = Asn1SeqField {
    name: "foo".into(),
    asn1_type: ::data::Asn1Type::Type("Bar".into()),
  };
  let field2 = Asn1SeqField {
    name: "asdf".into(),
    asn1_type: ::data::Asn1Type::Type("INTEGER".into()),
  };
  assert_eq!(
    field1,
    asn1_seq_field("foo Bar".as_bytes()).unwrap().1
  );
  assert_eq!(
    field2,
    asn1_seq_field("asdf INTEGER,".as_bytes()).unwrap().1
  );
  assert_eq!(
    field1,
    asn1_seq_field("foo--test\n Bar".as_bytes()).unwrap().1
  );
}

#[test]
fn test_seq_fields() {
  let seq = Asn1Seq {
    fields: vec![
      Asn1SeqField {
        name: "foo".into(),
        asn1_type: ::data::Asn1Type::Type("Bar".into()),
      },
      Asn1SeqField {
        name: "asdf".into(),
        asn1_type: ::data::Asn1Type::Type("INTEGER".into()),
      }
    ],
  };
  assert_eq!(
    seq,
    asn1_seq("\
      SEQUENCE {\
        foo Bar,\
        asdf INTEGER\
      }\
    ".as_bytes()).unwrap().1
  );
  assert_eq!(
    seq,
    asn1_seq("\
      SEQUENCE {
        foo Bar --,
        , asdf INTEGER
      }
    ".as_bytes()).unwrap().1
  );
  assert!(asn1_seq("SEQUENC ".as_bytes()).is_err());
}
