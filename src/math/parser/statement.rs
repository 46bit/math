use super::*;
use super::super::*;

named!(pub statements<&[u8], Statements>,
  map!(complete!(many0!(ws!(call!(statement)))), Statements));

named!(pub statement<&[u8], Statement>,
  do_parse!(
    statement: alt_complete!(
      map!(variable_assignment, |t| Statement::VarAssignment(t.0, t.1)) |
      map!(function_definition, |t| Statement::FnDefinition(t.0, t.1, t.2))) >>
    ws!(tag!(";")) >>
    (statement)));

named!(variable_assignment<&[u8], (Name, Expression)>,
  do_parse!(
    name: call!(variable_name) >>
    ws!(tag!("=")) >>
    expression: call!(expression) >>
    peek!(ws!(tag!(";"))) >>
    (name, expression)));

named!(function_definition<&[u8], (Name, Vec<Name>, Expression)>,
  do_parse!(
    name: call!(function_name) >>
    ws!(tag!("(")) >>
    parameters: call!(variable_names) >>
    ws!(tag!(")")) >>
    ws!(tag!("=")) >>
    expression: call!(expression) >>
    peek!(ws!(tag!(";"))) >>
    (name, parameters, expression)));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    #[test]
    fn assignment() {
        statement(b"a = 1;").unwrap();
    }

    #[test]
    fn assignment_with_operation() {
        statement(b"b = 1 + 2;").unwrap();
    }

    #[test]
    fn definition() {
        assert_eq!(
            statement(b"f(a) = a * 3;"),
            as_done(
                b"",
                Statement::FnDefinition(
                    as_name("f"),
                    vec![as_name("a")],
                    Expression::Operation(
                        Operator::Multiply,
                        box Expression::Operand(Operand::VarSubstitution(as_name("a"))),
                        box Expression::Operand(Operand::I64(3))
                    )
                )
            )
        );
    }

    #[test]
    fn zero_parameters() {
        assert_eq!(
            function_definition(b"f() = -11;"),
            as_done(
                b";",
                (as_name("f"), vec![], Expression::Operand(Operand::I64(-11)))
            )
        );

        assert_eq!(
            statement(b"f() = -11;"),
            as_done(
                b"",
                Statement::FnDefinition(
                    as_name("f"),
                    vec![],
                    Expression::Operand(Operand::I64(-11))
                )
            )
        );
    }

    fn as_name(s: &str) -> Name {
        Name(s.to_string())
    }

    fn as_done<O, E>(remaining: &[u8], output: O) -> IResult<&[u8], O, E> {
        IResult::Done(&remaining[..], output)
    }
}
