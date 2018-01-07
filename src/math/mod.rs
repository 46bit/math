pub mod parser;
pub mod interpreter;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ParseError(parser::Error),
    InterpreterError(interpreter::Error),
}

pub fn interpret(s: &[u8]) -> Result<HashMap<Name, i64>, Error> {
    let statements = parser::parse(s).map_err(Error::ParseError)?;
    let results = interpreter::execute(statements).map_err(Error::InterpreterError)?;
    return Ok(results);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    VarAssignment(Name, Expression),
    FnDefinition(Name, Vec<Name>, Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression(Operand, Vec<(Operator, Operand)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    I64(i64),
    VarSubstitution(Name),
    FnApplication(Name, Vec<Expression>),
}
