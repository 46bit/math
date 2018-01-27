pub mod parser;
pub mod interpreter;
pub mod compiler;

use std::fmt;
use std::collections::{HashMap, HashSet};
use quickcheck::{Arbitrary, Gen};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ParseError(parser::Error),
    InterpreterError(interpreter::Error),
    CompilerError(compiler::Error),
}

pub fn interpret(s: &[u8], inputs: &Vec<i64>) -> Result<Vec<i64>, Error> {
    let program = parser::parse(s).map_err(Error::ParseError)?;
    let outputs = interpreter::execute(&program, inputs).map_err(Error::InterpreterError)?;
    return Ok(outputs);
}

pub fn compile(s: &[u8], emit: compiler::Emit) -> Result<String, Error> {
    let program = parser::parse(s).map_err(Error::ParseError)?;
    let results = unsafe { compiler::compile(&program, emit).map_err(Error::CompilerError)? };
    return Ok(results);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(pub String);

impl Name {
    pub fn new(s: &str) -> Name {
        Name(s.to_string())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Arbitrary for Name {
    fn arbitrary<G: Gen>(g: &mut G) -> Name {
        arbitrary_name(g, 0)
    }
}

fn arbitrary_name<G: Gen>(g: &mut G, level: usize) -> Name {
    let size = g.size().saturating_sub(level).max(1);
    let len = 0..g.gen_range(1, size + 1);
    Name(len.map(|i| {
        let chars = match i {
            0 => "abcdefghijklmnopqrstuvwxyz",
            _ => "abcdefghijklmnopqrstuvwxyz_",
        };
        *g.choose(chars.as_bytes()).unwrap() as char
    }).collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    inputs: Vec<Name>,
    statements: Statements,
    outputs: Vec<Name>,
}

impl Program {
    pub fn new(inputs: Vec<Name>, statements: Statements, outputs: Vec<Name>) -> Program {
        Program {
            inputs: inputs,
            statements: statements,
            outputs: outputs,
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "inputs")?;
        if !self.inputs.is_empty() {
            write!(
                f,
                " {}",
                self.inputs
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(f, ";\n")?;

        if !self.statements.0.is_empty() {
            write!(f, "{}\n", self.statements)?;
        }

        write!(f, "outputs")?;
        if !self.outputs.is_empty() {
            write!(
                f,
                " {}",
                self.outputs
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(f, ";")
    }
}

impl Arbitrary for Program {
    fn arbitrary<G: Gen>(g: &mut G) -> Program {
        arbitrary_program(g, &mut HashSet::new(), &mut HashMap::new())
    }
}

fn arbitrary_program<G: Gen>(
    g: &mut G,
    mut vars: &mut HashSet<Name>,
    fns: &mut HashMap<Name, usize>,
) -> Program {
    let size = g.size();
    let input_len = 0..g.gen_range(0, size + 1);
    let inputs: Vec<Name> = input_len.map(|_| arbitrary_name(g, 1)).collect();

    vars.extend(inputs.iter().cloned());
    let statements = arbitrary_statements(g, &mut vars, fns);

    let output_len = g.gen_range(0, size + 1);
    let mut outputs = vars.clone().into_iter().collect::<Vec<_>>();
    let output_slice = outputs.as_mut_slice();
    g.shuffle(output_slice);
    let outputs = output_slice
        .into_iter()
        .take(output_len)
        .map(|o| o.clone())
        .collect();

    Program::new(inputs, statements, outputs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statements(pub Vec<Statement>);

impl fmt::Display for Statements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl Arbitrary for Statements {
    fn arbitrary<G: Gen>(g: &mut G) -> Statements {
        arbitrary_statements(g, &mut HashSet::new(), &mut HashMap::new())
    }
}

fn arbitrary_statements<G: Gen>(
    g: &mut G,
    vars: &mut HashSet<Name>,
    fns: &mut HashMap<Name, usize>,
) -> Statements {
    let size = g.size();
    let statements_len = 0..g.gen_range(0, size + 1);
    statements_len
        .fold(
            (Statements(Vec::new()), vars, fns),
            |(mut statements, mut vars, mut fns), _| {
                statements
                    .0
                    .push(arbitrary_statement(g, 1, &mut vars, &mut fns));
                (statements, vars, fns)
            },
        )
        .0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    VarAssignment(Name, Expression),
    FnDefinition(Name, Vec<Name>, Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Statement::VarAssignment(ref n, ref e) => write!(f, "{} = {}", n, e)?,
            Statement::FnDefinition(ref n, ref params, ref e) => {
                write!(f, "{}(", n)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") = {}", e)?;
            }
        }
        write!(f, ";")
    }
}

impl Arbitrary for Statement {
    fn arbitrary<G: Gen>(g: &mut G) -> Statement {
        arbitrary_statement(g, 0, &mut HashSet::new(), &mut HashMap::new())
    }
}

fn arbitrary_statement<G: Gen>(
    g: &mut G,
    level: usize,
    vars: &mut HashSet<Name>,
    fns: &mut HashMap<Name, usize>,
) -> Statement {
    let size = g.size().saturating_sub(level).max(1);
    match g.gen_range(0, 2) {
        0 => {
            let var_name = Name::arbitrary(g);
            let statement = Statement::VarAssignment(
                var_name.clone(),
                arbitrary_expression(g, level + 1, vars, fns),
            );
            vars.insert(var_name);
            statement
        }
        1 => {
            let fn_name = Name::arbitrary(g);
            let params: HashSet<_> = (0..g.gen_range(0, size + 1))
                .map(|_| Name::arbitrary(g))
                .collect();
            let params_count = params.len();
            // Removing any previous function by this name prevents the expression from
            // using the previously defined function
            fns.remove(&fn_name);
            let expr = arbitrary_expression(g, level + 1, &params.clone(), fns);
            // FIXME: Remove or reduce parameters not used in the expression?
            let statement =
                Statement::FnDefinition(fn_name.clone(), params.into_iter().collect(), expr);
            fns.insert(fn_name, params_count);
            statement
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Operand(Operand),
    Operation(Operator, Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Operand(Operand),
    Operator(Operator),
}

impl Expression {
    pub fn tokens(&self) -> Vec<Token> {
        match *self {
            Expression::Operand(ref operand) => vec![Token::Operand(operand.clone())],
            Expression::Operation(operator, ref exp0, ref exp1) => {
                let mut tokens = exp0.tokens();
                tokens.push(Token::Operator(operator));
                tokens.extend(exp1.tokens());
                tokens
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::Operand(ref v) => write!(f, "{}", v),
            Expression::Operation(ref operator, ref expr1, ref expr2) => {
                if let &box Expression::Operation(ref inner_operator, _, _) = expr1 {
                    if inner_operator < operator {
                        write!(f, "({})", expr1)?;
                    } else {
                        write!(f, "{}", expr1)?;
                    }
                } else {
                    write!(f, "{}", expr1)?;
                }
                write!(f, " {} ", operator)?;
                if let &box Expression::Operation(ref inner_operator, _, _) = expr2 {
                    if inner_operator < operator {
                        write!(f, "({})", expr2)?;
                    } else {
                        write!(f, "{}", expr2)?;
                    }
                } else {
                    write!(f, "{}", expr2)?;
                }
                Ok(())
            }
        }
    }
}

impl Arbitrary for Expression {
    fn arbitrary<G: Gen>(g: &mut G) -> Expression {
        arbitrary_expression(g, 0, &HashSet::new(), &HashMap::new())
    }
}

fn arbitrary_expression<G: Gen>(
    g: &mut G,
    level: usize,
    vars: &HashSet<Name>,
    fns: &HashMap<Name, usize>,
) -> Expression {
    let size = g.size().saturating_sub(level);
    // Terminate expressions of sufficient depth.
    if size <= 1 {
        return Expression::Operand(arbitrary_operand(g, level + 1, vars, fns));
    }
    match g.gen_range(0, 2) {
        0 => Expression::Operand(arbitrary_operand(g, level + 1, vars, fns)),
        1 => {
            let operator = Operator::arbitrary(g);
            let mut exp0 = arbitrary_expression(g, level + 1, vars, fns);
            let mut exp1 = arbitrary_expression(g, level + 1, vars, fns);
            if let Expression::Operation(inner_operator, _, _) = exp0.clone() {
                if inner_operator < operator {
                    exp0 = Expression::Operand(Operand::Group(box exp0));
                }
            }
            if let Expression::Operation(inner_operator, _, _) = exp1.clone() {
                if inner_operator < operator {
                    exp1 = Expression::Operand(Operand::Group(box exp1));
                }
            }
            Expression::Operation(operator, box exp0, box exp1)
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    I64(i64),
    Group(Box<Expression>),
    VarSubstitution(Name),
    FnApplication(Name, Vec<Expression>),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::I64(n) => write!(f, "{}", n),
            Operand::Group(ref exp) => write!(f, "({})", exp),
            Operand::VarSubstitution(ref name) => write!(f, "{}", name),
            Operand::FnApplication(ref name, ref args) => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl Arbitrary for Operand {
    fn arbitrary<G: Gen>(g: &mut G) -> Operand {
        arbitrary_operand(g, 0, &HashSet::new(), &HashMap::new())
    }
}

fn arbitrary_operand<G: Gen>(
    g: &mut G,
    level: usize,
    vars: &HashSet<Name>,
    fns: &HashMap<Name, usize>,
) -> Operand {
    let size = g.size().saturating_sub(level);
    // Terminate expressions of sufficient depth.
    if size <= 1 {
        return Operand::I64(i64::arbitrary(g));
    }
    match g.gen_range(0, 3) {
        0 => Operand::I64(i64::arbitrary(g)),
        1 => g.choose(vars.iter().collect::<Vec<_>>().as_slice())
            .map(|var_name| Operand::VarSubstitution(var_name.clone().clone()))
            .unwrap_or_else(|| Operand::I64(i64::arbitrary(g))),
        2 => g.choose(fns.iter().collect::<Vec<_>>().as_slice())
            .map(|&(ref fn_name, params_count)| {
                Operand::FnApplication(
                    fn_name.clone().clone(),
                    (0..*params_count)
                        .map(|_| arbitrary_expression(g, level + 1, vars, fns))
                        .collect(),
                )
            })
            .unwrap_or_else(|| Operand::I64(i64::arbitrary(g))),
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
    Subtract,
    Add,
    Divide,
    Multiply,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Operator::Subtract => "-",
                Operator::Add => "+",
                Operator::Divide => "/",
                Operator::Multiply => "*",
            }
        )
    }
}

impl Arbitrary for Operator {
    fn arbitrary<G: Gen>(g: &mut G) -> Operator {
        match g.gen_range(0, 4) {
            0 => Operator::Subtract,
            1 => Operator::Add,
            2 => Operator::Divide,
            3 => Operator::Multiply,
            _ => unreachable!(),
        }
    }
}
