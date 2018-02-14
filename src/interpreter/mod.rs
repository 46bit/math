use super::*;
use std::collections::HashMap;

// FIXME: Function parameter names should all differ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownVariable(Name),
    UnknownFunction(Name),
    IncorrectArgumentCount {
        name: Name,
        params_count: usize,
        provided_count: usize,
    },
    IncorrectInputCount {
        inputs_count: usize,
        provided_count: usize,
    },
}

pub fn execute(program: &Program, inputs: &Vec<i64>) -> Result<Vec<i64>, Error> {
    let mut interpreter = Interpreter::new();
    interpreter.run(&program, inputs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function(Vec<Name>, Expression, HashMap<Name, Function>);

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub variables: HashMap<Name, i64>,
    pub functions: HashMap<Name, Function>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: &Program, inputs: &Vec<i64>) -> Result<Vec<i64>, Error> {
        if program.inputs.len() != inputs.len() {
            return Err(Error::IncorrectInputCount {
                inputs_count: program.inputs.len(),
                provided_count: inputs.len(),
            });
        }
        self.variables
            .extend(program.inputs.iter().cloned().zip(inputs.iter().cloned()));

        for statement in &program.statements.0 {
            self.statement(statement)?;
        }

        let mut outputs = Vec::new();
        for output in program.outputs.iter() {
            let variables = self.variables.clone();
            let variable = self.variable(output, &variables)?;
            outputs.push(variable);
        }
        Ok(outputs)
    }

    pub fn statement(&mut self, statement: &Statement) -> Result<(), Error> {
        match statement {
            &Statement::VarAssignment(ref name, ref expr) => {
                let variables = self.variables.clone();
                let functions = self.functions.clone();
                let expr_value = self.expression(expr, &variables, &functions)?;
                self.variables.insert(name.clone(), expr_value);
            }
            &Statement::FnDefinition(ref name, ref params, ref expr) => {
                let function = Function(params.clone(), expr.clone(), self.functions.clone());
                self.functions.insert(name.clone(), function);
            }
        }
        Ok(())
    }

    fn expression(
        &mut self,
        expr: &Expression,
        variables: &HashMap<Name, i64>,
        functions: &HashMap<Name, Function>,
    ) -> Result<i64, Error> {
        match expr {
            &Expression::Operand(ref operand) => self.operand(operand, variables, functions),
            &Expression::Operation(ref operator, ref expr1, ref expr2) => {
                self.operation(operator, expr1, expr2, variables, functions)
            }
        }
    }

    fn operation(
        &mut self,
        operator: &Operator,
        operand1: &Expression,
        operand2: &Expression,
        variables: &HashMap<Name, i64>,
        functions: &HashMap<Name, Function>,
    ) -> Result<i64, Error> {
        let value1 = self.expression(operand1, variables, functions)?;
        let value2 = self.expression(operand2, variables, functions)?;
        Ok(match operator {
            &Operator::Add => value1.saturating_add(value2),
            &Operator::Subtract => value1.saturating_sub(value2),
            &Operator::Multiply => value1.saturating_mul(value2),
            &Operator::Divide => if value2 == 0 {
                if value1 < 0 {
                    i64::min_value()
                } else {
                    i64::max_value()
                }
            } else if value1 == i64::min_value() && value2 == -1 {
                i64::max_value()
            } else {
                value1.wrapping_div(value2)
            },
        })
    }

    fn operand(
        &mut self,
        operand: &Operand,
        variables: &HashMap<Name, i64>,
        functions: &HashMap<Name, Function>,
    ) -> Result<i64, Error> {
        match operand {
            &Operand::I64(value) => Ok(value),
            &Operand::Group(ref expr) => self.expression(expr, variables, functions),
            &Operand::VarSubstitution(ref name) => self.variable(name, variables),
            &Operand::FnApplication(ref name, ref args) => {
                self.function_call(name, args, variables, functions)
            }
            &Operand::Match(ref match_) => self.match_(match_, variables, functions),
        }
    }

    fn variable(&mut self, name: &Name, variables: &HashMap<Name, i64>) -> Result<i64, Error> {
        variables
            .get(&name)
            .cloned()
            .ok_or_else(|| Error::UnknownVariable(name.clone()))
    }

    fn function_call(
        &mut self,
        name: &Name,
        arg_exprs: &Vec<Expression>,
        caller_variables: &HashMap<Name, i64>,
        caller_functions: &HashMap<Name, Function>,
    ) -> Result<i64, Error> {
        let Function(ref params, ref expr, ref functions) = caller_functions
            .get(&name)
            .ok_or_else(|| Error::UnknownFunction(name.clone()))?
            .clone();

        if params.len() != arg_exprs.len() {
            return Err(Error::IncorrectArgumentCount {
                name: name.clone(),
                params_count: params.len(),
                provided_count: arg_exprs.len(),
            });
        }

        let mut args = Vec::new();
        for arg_expr in arg_exprs {
            args.push(self.expression(arg_expr, caller_variables, caller_functions)?);
        }
        let variables = params.iter().cloned().zip(args.into_iter()).collect();
        let result = self.expression(expr, &variables, functions);
        return result;
    }

    fn match_(
        &mut self,
        match_: &Match,
        variables: &HashMap<Name, i64>,
        functions: &HashMap<Name, Function>,
    ) -> Result<i64, Error> {
        // with: Box<Expression>
        // clauses: Vec<(Matcher, Expression)>
        // default: Option<Box<Expression>>
        let with = self.expression(&match_.with, variables, functions)?;
        for &(ref matcher, ref expression) in &match_.clauses {
            match matcher {
                &Matcher::Value(ref value) => {
                    let value = self.expression(value, variables, functions)?;
                    if with == value {
                        return self.expression(expression, variables, functions);
                    }
                }
            }
        }
        self.expression(&match_.default, variables, functions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser::{expression, parse, statement};
    use rand::thread_rng;
    use quickcheck::{QuickCheck, StdGen};

    fn interprets_successfully_property(program: Program) -> bool {
        let inputs: Vec<i64> = (0..program.inputs.len()).map(|n| n as i64 - 3).collect();
        let result = execute(&program, &inputs);
        if let Ok(ref outputs) = result {
            assert_eq!(outputs.len(), program.outputs.len());
        }
        if let Err(ref e) = result {
            eprintln!("{:?}", e);
        }
        result.is_ok()
    }

    #[test]
    fn interprets_successfully() {
        // QuickCheck's default size creates infeasibly vast statements, and beyond some
        // point they stop exploring novel code paths. This does a much better job of
        // exploring potential edgecases.
        for size in 1..11 {
            let mut qc = QuickCheck::new().gen(StdGen::new(thread_rng(), size));
            qc.quickcheck(interprets_successfully_property as fn(Program) -> bool);
        }
    }

    #[test]
    fn unprovided_input_errors() {
        let mut i = Interpreter::new();
        assert_eq!(
            i.run(&parse(b"inputs a; b = 1; outputs b;").unwrap(), &vec![]),
            Err(Error::IncorrectInputCount {
                inputs_count: 1,
                provided_count: 0,
            })
        );
    }

    #[test]
    fn undefined_output_errors() {
        let mut i = Interpreter::new();
        assert_eq!(
            i.run(&parse(b"inputs; b = 1; outputs z;").unwrap(), &vec![]),
            Err(Error::UnknownVariable(as_name("z")))
        );
    }

    #[test]
    fn var_can_be_redefined() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"n = 1;").unwrap().1).unwrap();
        i.statement(&statement(b"n = 2;").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("n")], 2);
    }

    #[test]
    fn var_can_be_redefined_using_itself() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"a = 2;").unwrap().1).unwrap();
        i.statement(&statement(b"b = 3;").unwrap().1).unwrap();
        i.statement(&statement(b"a = a + b;").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("a")], 5);
    }

    #[test]
    fn var_errors_if_undefined() {
        let mut i = Interpreter::new();
        assert_eq!(
            i.statement(&statement(b"a = b;").unwrap().1),
            Err(Error::UnknownVariable(as_name("b")))
        );
    }

    #[test]
    fn fn_can_be_redefined() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a) = a;").unwrap().1).unwrap();
        i.statement(&statement(b"f(a) = 3 * a;").unwrap().1)
            .unwrap();
        let mut functions = HashMap::new();
        functions.insert(
            as_name("f"),
            Function(
                vec![as_name("a")],
                Expression::Operand(Operand::VarSubstitution(as_name("a"))),
                HashMap::new(),
            ),
        );
        assert_eq!(
            i.functions[&as_name("f")],
            Function(
                vec![as_name("a")],
                expression(b"3 * a;").unwrap().1,
                functions
            )
        );
    }

    #[test]
    fn fn_has_a_single_signature() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a) = a;").unwrap().1).unwrap();
        i.statement(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        let mut functions = HashMap::new();
        functions.insert(
            as_name("f"),
            Function(
                vec![as_name("a")],
                Expression::Operand(Operand::VarSubstitution(as_name("a"))),
                HashMap::new(),
            ),
        );
        assert_eq!(
            i.functions[&as_name("f")],
            Function(
                vec![as_name("a"), as_name("b")],
                expression(b"a + b;").unwrap().1,
                functions
            )
        );
    }

    #[test]
    fn fn_cannot_use_external_variables() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"n = 1;").unwrap().1).unwrap();
        i.statement(&statement(b"f(a) = a * n;").unwrap().1)
            .unwrap();
        assert_eq!(
            i.statement(&statement(b"j = f(2);").unwrap().1),
            Err(Error::UnknownVariable(as_name("n")))
        );
    }

    #[test]
    fn fn_params_can_reuse_external_variable_names() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"i = 1;").unwrap().1).unwrap();
        i.statement(&statement(b"f(i) = i;").unwrap().1).unwrap();
        i.statement(&statement(b"j = f(2);").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("j")], 2);
    }

    #[test]
    fn fn_args_can_use_external_variables() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"a = 1;").unwrap().1).unwrap();
        i.statement(&statement(b"f(b) = b;").unwrap().1).unwrap();
        i.statement(&statement(b"j = f(a);").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("j")], 1);
    }

    #[test]
    fn fn_args_use_current_external_variables() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"x = 1;").unwrap().1).unwrap();
        i.statement(&statement(b"f(a) = a;").unwrap().1).unwrap();
        i.statement(&statement(b"y = f(x);").unwrap().1).unwrap();
        i.statement(&statement(b"x = 2;").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("y")], 1);
    }

    #[test]
    fn fn_evaluates_functions_from_when_it_was_defined_1() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a, b) = a * b;").unwrap().1)
            .unwrap();
        println!("{:?}", i.functions);
        i.statement(&statement(b"g(x) = f(x, x);").unwrap().1)
            .unwrap();
        println!("{:?}", i.functions);
        i.statement(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        println!("{:?}", i.functions);
        i.statement(&statement(b"n = g(3);").unwrap().1).unwrap();
        println!("{:?}", i.functions);
        assert_eq!(i.variables[&as_name("n")], 9);
    }

    #[test]
    fn fn_evaluates_functions_from_when_it_was_defined_2() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        println!("{:?}", i.functions);
        i.statement(&statement(b"g(x) = f(x, x + 1);").unwrap().1)
            .unwrap();
        println!("{:?}", i.functions);
        i.statement(&statement(b"f(a) = a;").unwrap().1).unwrap();
        println!("{:?}", i.functions);
        i.statement(&statement(b"n = g(1);").unwrap().1).unwrap();
        println!("{:?}", i.functions);
        assert_eq!(i.variables[&as_name("n")], 3);
    }

    #[test]
    fn fn_can_call_previous_definition_of_same_name() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a) = a * 5;").unwrap().1).unwrap();
        i.statement(&statement(b"f(a, b) = f(a) + b;").unwrap().1).unwrap();
        i.statement(&statement(b"i = f(2, 1);").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("i")], 11);
    }

    #[test]
    fn fn_errors_if_undefined() {
        let mut i = Interpreter::new();
        assert_eq!(
            i.statement(&statement(b"a = f(1);").unwrap().1),
            Err(Error::UnknownFunction(as_name("f")))
        );
    }

    #[test]
    fn fn_too_few_args_errors() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"a(x) = x;").unwrap().1).unwrap();
        assert_eq!(
            i.statement(&statement(b"i = a();").unwrap().1),
            Err(Error::IncorrectArgumentCount {
                name: as_name("a"),
                params_count: 1,
                provided_count: 0,
            })
        );
    }

    #[test]
    fn fn_too_many_args_errors() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"b(x, y) = x - y;").unwrap().1)
            .unwrap();
        assert_eq!(
            i.statement(&statement(b"i = b(1, 2, 3);").unwrap().1),
            Err(Error::IncorrectArgumentCount {
                name: as_name("b"),
                params_count: 2,
                provided_count: 3,
            })
        );
    }

    #[test]
    fn var_and_fn_names_can_overlap() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"a = 7;").unwrap().1).unwrap();
        i.statement(&statement(b"a(x, y) = x / y;").unwrap().1)
            .unwrap();
        i.statement(&statement(b"b = a;").unwrap().1).unwrap();
        i.statement(&statement(b"c = a(3, 2);").unwrap().1).unwrap();
        i.statement(&statement(b"d = a * a(a, 3);").unwrap().1)
            .unwrap();
        assert_eq!(i.variables[&as_name("a")], 7);
        assert_eq!(i.variables[&as_name("b")], 7);
        assert_eq!(i.variables[&as_name("c")], 3 / 2);
        assert_eq!(i.variables[&as_name("d")], 7 * (7 / 3));
    }

    #[test]
    fn vars_use_current_fn_definitions() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a) = a;").unwrap().1).unwrap();
        i.statement(&statement(b"x = f(1);").unwrap().1).unwrap();
        i.statement(&statement(b"f(a) = a + 1;").unwrap().1)
            .unwrap();
        i.statement(&statement(b"y = f(1);").unwrap().1).unwrap();
        i.statement(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        i.statement(&statement(b"z = f(1, 2);").unwrap().1).unwrap();
        assert_eq!(i.variables[&as_name("x")], 1);
        assert_eq!(i.variables[&as_name("y")], 2);
        assert_eq!(i.variables[&as_name("z")], 3);
    }

    fn as_name(s: &str) -> Name {
        Name(s.to_string())
    }
}
