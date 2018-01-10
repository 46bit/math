#[cfg_attr(rustfmt, rustfmt_skip)]

mod name;
mod expression;
mod statement;
mod shunting_yard;

pub use self::name::*;
pub use self::expression::*;
pub use self::statement::*;

use super::*;
use nom::{simple_errors, IResult, Needed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    RemainingInput(Vec<u8>, Statements),
    Nom(simple_errors::Err<u32>),
    NomIncomplete(Needed),
}

pub fn parse(s: &[u8]) -> Result<Statements, Error> {
    match statements(s) {
        IResult::Done(&[], statements) => Ok(statements),
        IResult::Done(i, o) => Err(Error::RemainingInput(i.to_vec(), o)),
        IResult::Error(e) => Err(Error::Nom(e)),
        IResult::Incomplete(n) => Err(Error::NomIncomplete(n)),
    }
}

pub fn parse_one(s: &[u8]) -> Result<Statement, Error> {
    match statement(s) {
        IResult::Done(&[], statement) => Ok(statement),
        IResult::Done(i, o) => Err(Error::RemainingInput(i.to_vec(), Statements(vec![o]))),
        IResult::Error(e) => Err(Error::Nom(e)),
        IResult::Incomplete(n) => Err(Error::NomIncomplete(n)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use quickcheck::{QuickCheck, StdGen};

    #[test]
    fn assignment() {
        parse_one(b"a = 1;").unwrap();
    }

    #[test]
    fn assignment_with_operation() {
        parse_one(b"b = 1 + 2;").unwrap();
    }

    #[test]
    fn definition() {
        parse_one(b"f(a) = a * 3;").unwrap();
    }

    fn parses_correctly_prop(input: Statements) -> bool {
        println!("{}", input.clone());
        format!("{}", parse(format!("{}", input).as_bytes()).unwrap()) == format!("{}", input)
    }

    #[test]
    fn parses_correctly() {
        let mut qc = QuickCheck::new().gen(StdGen::new(thread_rng(), 11));
        qc.quickcheck(parses_correctly_prop as fn(Statements) -> bool);
    }
}
