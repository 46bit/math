use super::super::*;
use std::str;
use nom::is_space;

named!(pub name<&[u8], Name>,
  map!(
    verify!(
      map!(
        take_till1!(|b: u8| is_space(b) || RESERVED_NAME_BYTES.contains(&b)),
        |bytes| str::from_utf8(&bytes).unwrap()
      ),
      |s| !RESERVED_NAMES.contains(&s)
    ),
    |s| Name::new(s)
  ));

named!(pub names<&[u8], Vec<Name>>,
  do_parse!(
    first_variable: opt!(call!(name)) >>
    other_variables: many0!(preceded!(ws!(tag!(",")), call!(name))) >>
    ({
      if first_variable.is_none() {
        vec![]
      } else {
        let mut v = vec![first_variable.unwrap()];
        v.extend(other_variables);
        v
      }
    })));
