#[cfg_attr(rustfmt, rustfmt_skip)]

mod name;
mod expression;
mod statement;
mod shunting_yard;
mod program;

pub use self::name::*;
pub use self::expression::*;
pub use self::statement::*;
pub use self::program::*;

use super::*;
use nom::{simple_errors, IResult, Needed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    RemainingInput(Vec<u8>, Program),
    Nom(simple_errors::Err<u32>),
    NomIncomplete(Needed),
    DuplicateInput(Name),
}

pub fn parse(s: &[u8]) -> Result<Program, Error> {
    let program = match program(s) {
        IResult::Done(&[], program) => Ok(program),
        IResult::Done(i, o) => Err(Error::RemainingInput(i.to_vec(), o)),
        IResult::Error(e) => Err(Error::Nom(e)),
        IResult::Incomplete(n) => Err(Error::NomIncomplete(n)),
    }?;
    let mut input_map = HashSet::new();
    for input in &program.inputs {
        if input_map.contains(input) {
            return Err(Error::DuplicateInput(input.clone()));
        }
        input_map.insert(input.clone());
    }
    Ok(program)
}

pub fn parse_one(s: &[u8]) -> Result<Statement, Error> {
    match statement(s) {
        IResult::Done(&[], statement) => Ok(statement),
        IResult::Done(i, o) => Err(Error::RemainingInput(
            i.to_vec(),
            Program::new(vec![], Statements(vec![o]), vec![]),
        )),
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

    fn parses_correctly_prop(input: Program) -> bool {
        format!("{}", parse(format!("{}", input).as_bytes()).unwrap()) == format!("{}", input)
    }

    #[test]
    fn parses_correctly() {
        // QuickCheck's default size creates infeasibly vast statements, and beyond some
        // point they stop exploring novel code paths. This does a much better job of
        // exploring potential edgecases.
        for size in 1..11 {
            let mut qc = QuickCheck::new().gen(StdGen::new(thread_rng(), size));
            qc.quickcheck(parses_correctly_prop as fn(Program) -> bool);
        }
    }
}
