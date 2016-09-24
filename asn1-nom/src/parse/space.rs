use nom::{multispace, eol};

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
  assert_eq!(
    "".as_bytes(),
    comment("--\n".as_bytes()).unwrap().0
  );
  assert_eq!(
    "".as_bytes(),
    comment("--foo bar asdf\n".as_bytes()).unwrap().0
  );
  assert_eq!(
    "  ".as_bytes(),
    comment("--\n  ".as_bytes()).unwrap().0
  );
  assert_eq!(
    "foobar".as_bytes(),
    comment("--asdf\nfoobar".as_bytes()).unwrap().0
  );
  assert!(comment("foo --\n".as_bytes()).is_err());
  assert!(comment(" bar --\n".as_bytes()).is_err());
  assert!(comment(" -asdf-\n".as_bytes()).is_err());
  assert!(comment("-".as_bytes()).is_incomplete());
  assert!(comment("--".as_bytes()).is_incomplete());
}

#[test]
fn test_skip_other() {
  assert_eq!(
    "".as_bytes(),
    skip_other("  --\n".as_bytes()).unwrap().0
  );
  assert_eq!(
    "".as_bytes(),
    skip_other("--\n  ".as_bytes()).unwrap().0
  );
  assert_eq!(
    "".as_bytes(),
    skip_other("--\n \n ".as_bytes()).unwrap().0
  );
  assert_eq!(
    "".as_bytes(),
    skip_other("--\n -- adf\n ".as_bytes()).unwrap().0
  );
  assert!(skip_other(" -".as_bytes()).is_incomplete());
  assert!(skip_other(" --".as_bytes()).is_incomplete());
  assert!(skip_other(" --\n -".as_bytes()).is_done());
}
