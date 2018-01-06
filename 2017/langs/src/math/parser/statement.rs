use super::*;
use super::super::*;

named!(pub statements<&[u8], Vec<Statement>>,
  complete!(many0!(ws!(call!(statement)))));

named!(pub statement<&[u8], Statement>,
  alt_complete!(
    map!(variable_assignment, |t| Statement::VarAssignment(t.0, t.1)) |
    map!(function_definition, |t| Statement::FnDefinition(t.0, t.1, t.2))));

named!(variable_assignment<&[u8], (Name, Expression)>,
  do_parse!(
    name: call!(variable_name) >>
    ws!(tag!("=")) >>
    expression: call!(expression) >>
    ws!(tag!(";")) >>
    (name, expression)));

named!(function_definition<&[u8], (Name, Vec<Name>, Expression)>,
  do_parse!(
    name: call!(function_name) >>
    ws!(tag!("(")) >>
    parameters: separated_list!(
      ws!(tag!(",")),
      call!(variable_name)) >>
    ws!(tag!(")")) >>
    ws!(tag!("=")) >>
    expression: call!(expression) >>
    ws!(tag!(";")) >>
    (name, parameters, expression)));
