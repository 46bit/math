use super::super::*;
use std::string;
use nom::is_space;

named!(pub variable_name<&[u8], Name>,
  map!(
    take_till1!(|b: u8| is_space(b) || b == b'=' || b == b'(' || b == b')' || b == b',' || b == b';'),
    |bytes| Name(to_str(bytes).unwrap())));

named!(pub variable_names<&[u8], Vec<Name>>,
  do_parse!(
    first_variable: opt!(call!(variable_name)) >>
    other_variables: many0!(preceded!(ws!(tag!(",")), call!(variable_name))) >>
    ({
      if first_variable.is_none() {
        vec![]
      } else {
        let mut v = vec![first_variable.unwrap()];
        v.extend(other_variables);
        v
      }
    })));

named!(pub function_name<&[u8], Name>,
  map!(
    take_till!(|b: u8| is_space(b) || b == b'=' || b == b'(' || b == b')' || b == b',' || b == b';'),
    |bytes| Name(to_str(bytes).unwrap())));

pub fn to_str(u8s: &[u8]) -> Result<String, string::FromUtf8Error> {
    String::from_utf8(u8s.to_vec())
}
