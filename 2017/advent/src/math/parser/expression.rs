use super::*;
use math::*;
use nom::is_digit;

named!(pub expressions<&[u8], Vec<Expression>>,
  separated_list!(ws!(tag!(",")), call!(expression)));

named!(pub expression<&[u8], Expression>,
  map!(
    pair!(call!(operand), many0!(pair!(
      ws!(call!(operator)),
      call!(operand)))),
    |t| Expression(t.0, t.1)));

named!(operator<&[u8], Operator>,
  map!(one_of!("+-*/"), |o| match o {
    '+' => Operator::Add,
    '-' => Operator::Subtract,
    '*' => Operator::Multiply,
    '/' => Operator::Divide,
    _ => unreachable!()
  }));

named!(operand<&[u8], Operand>,
  alt_complete!(
    map!(i64, Operand::I64) |
    map!(variable_substitution, Operand::VarSubstitution) |
    map!(function_application, |t| Operand::FnApplication(t.0, t.1))));

named!(i64<&[u8], i64>,
  map!(
    take_while1!(|b: u8| is_digit(b) || b == b'-'),
    |i| to_str(i).unwrap().parse().unwrap()));

named!(variable_substitution<&[u8], Name>,
  do_parse!(
    name: call!(variable_name) >>
    ws!(peek!(none_of!("("))) >>
    (name)));

named!(function_application<&[u8], (Name, Vec<Expression>)>,
  do_parse!(
    name: call!(function_name) >>
    ws!(tag!("(")) >>
    expressions: call!(expressions) >>
    ws!(tag!(")")) >>
    (name, expressions)));
