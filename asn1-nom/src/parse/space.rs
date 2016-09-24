use nom::{multispace, eol, newline};

named!(pub comment <()>, chain!(
  tag!("--") ~
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

pub fn is_eol(byte: u8) -> bool {
  let chr = byte as char;
  chr == '\n' || chr == '\r'
}

#[test]
fn test_comment() {
  println!("{:?}", comment("--\n".as_bytes()));
  assert_eq!(
    "".as_bytes(),
    comment("--\n".as_bytes()).unwrap().0
  );
}
