use super::*;
use super::super::*;

named!(pub inputs<&[u8], Vec<Name>>,
  do_parse!(
    ws!(tag!("inputs")) >>
    input_names: call!(variable_names) >>
    ws!(tag!(";")) >>
    (input_names)
  ));

named!(pub outputs<&[u8], Vec<Name>>,
  do_parse!(
    ws!(tag!("outputs")) >>
    output_names: call!(variable_names) >>
    ws!(tag!(";")) >>
    (output_names)
  ));

named!(pub program<&[u8], Program>,
  do_parse!(
    inputs: call!(inputs) >>
    statements: call!(statements) >>
    outputs: call!(outputs) >>
    (Program::new(inputs, statements, outputs))
  ));
