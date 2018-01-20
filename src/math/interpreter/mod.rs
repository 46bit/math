use super::*;
use std::collections::HashMap;

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

pub struct Interpreter {
    pub variables: HashMap<Name, i64>,
    pub functions: HashMap<Name, (Vec<Name>, Expression)>,
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
            outputs.push(self.variable(output)?);
        }
        Ok(outputs)
    }

    pub fn statement(&mut self, statement: &Statement) -> Result<(), Error> {
        match statement {
            &Statement::VarAssignment(ref name, ref expr) => {
                let expr_value = self.expression(expr)?;
                self.variables.insert(name.clone(), expr_value);
            }
            &Statement::FnDefinition(ref name, ref params, ref expr) => {
                self.functions
                    .insert(name.clone(), (params.clone(), expr.clone()));
            }
        }
        Ok(())
    }

    fn expression(&mut self, expr: &Expression) -> Result<i64, Error> {
        match expr {
            &Expression::Operand(ref operand) => self.operand(operand),
            &Expression::Operation(ref operator, ref expr1, ref expr2) => {
                self.operation(operator, expr1, expr2)
            }
        }
    }

    fn operation(
        &mut self,
        operator: &Operator,
        operand1: &Expression,
        operand2: &Expression,
    ) -> Result<i64, Error> {
        let value1 = self.expression(operand1)?;
        let value2 = self.expression(operand2)?;
        // FIXME: Document these behaviours or raise errors
        Ok(match operator {
            &Operator::Add => value1.saturating_add(value2),
            &Operator::Subtract => value1.saturating_sub(value2),
            &Operator::Multiply => value1.saturating_mul(value2),
            &Operator::Divide => if value2 == 0 {
                i64::max_value()
            } else {
                value1.wrapping_div(value2)
            },
        })
    }

    fn operand(&mut self, operand: &Operand) -> Result<i64, Error> {
        match operand {
            &Operand::I64(value) => Ok(value),
            &Operand::Group(ref expr) => self.expression(expr),
            &Operand::VarSubstitution(ref name) => self.variable(name),
            &Operand::FnApplication(ref name, ref args) => self.function_call(name, args),
        }
    }

    fn variable(&mut self, name: &Name) -> Result<i64, Error> {
        self.variables
            .get(&name)
            .cloned()
            .ok_or_else(|| Error::UnknownVariable(name.clone()))
    }

    fn function_call(&mut self, name: &Name, arg_exprs: &Vec<Expression>) -> Result<i64, Error> {
        let (ref params, ref expr) = self.functions
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
            args.push(self.expression(arg_expr)?);
        }

        let backup_of_global_variables = self.variables.clone();
        self.variables = params.iter().cloned().zip(args.into_iter()).collect();
        let result = self.expression(&expr);
        self.variables = backup_of_global_variables;
        return result;
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
        assert_eq!(
            i.functions[&as_name("f")],
            (vec![as_name("a")], expression(b"3 * a;").unwrap().1,)
        );
    }

    #[test]
    fn fn_has_a_single_signature() {
        let mut i = Interpreter::new();
        i.statement(&statement(b"f(a) = a;").unwrap().1).unwrap();
        i.statement(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        assert_eq!(
            i.functions[&as_name("f")],
            (
                vec![as_name("a"), as_name("b")],
                expression(b"a + b;").unwrap().1,
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
    fn fn_params_can_reuse_external_names() {
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
