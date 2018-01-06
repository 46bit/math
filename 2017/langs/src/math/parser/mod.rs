#[cfg_attr(rustfmt, rustfmt_skip)]

mod name;
mod expression;
mod statement;

pub use self::name::*;
pub use self::expression::*;
pub use self::statement::*;

use super::*;
use nom::{simple_errors, IResult, Needed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    RemainingInput(Vec<u8>, Vec<Statement>),
    Nom(simple_errors::Err<u32>),
    NomIncomplete(Needed),
}

pub fn parse(s: &[u8]) -> Result<Vec<Statement>, Error> {
    match statements(s) {
        IResult::Done(&[], statements) => Ok(statements),
        IResult::Done(i, o) => Err(Error::RemainingInput(i.to_vec(), o)),
        IResult::Error(e) => Err(Error::Nom(e)),
        IResult::Incomplete(n) => Err(Error::NomIncomplete(n)),
    }
}
