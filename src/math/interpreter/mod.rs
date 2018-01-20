use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownVariable(Name),
    UnknownFunction(Name),
    IncorrectArgumentCount {
        name: Name,
        params_count: usize,
        args_count: usize,
    },
}

pub fn execute(program: &Program, inputs: &Vec<i64>) -> Result<Vec<(Name, i64)>, Error> {
    let mut executor = Executor::new();
    Ok(executor.run(&program, inputs))
}

pub struct Executor {
    pub variables: HashMap<Name, i64>,
    pub functions: HashMap<Name, (Vec<Name>, Expression)>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: &Program, inputs: &Vec<i64>) -> Vec<(Name, i64)> {
        // FIXME: Error handling
        assert!(program.inputs.len() <= inputs.len());
        self.variables
            .extend(program.inputs.iter().cloned().zip(inputs.iter().cloned()));
        for statement in &program.statements.0 {
            self.execute(statement);
        }
        // FIXME: Error handling
        program
            .outputs
            .iter()
            .map(|output_name| (output_name.clone(), self.variables[output_name]))
            .collect()
    }

    pub fn execute(&mut self, statement: &Statement) -> Result<(), Error> {
        match statement {
            &Statement::VarAssignment(ref name, ref expr) => {
                let expr_value = self.evaluate_expression(expr)?;
                self.variables.insert(name.clone(), expr_value);
            }
            &Statement::FnDefinition(ref name, ref params, ref expr) => {
                self.functions
                    .insert(name.clone(), (params.clone(), expr.clone()));
            }
        }
        Ok(())
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<i64, Error> {
        match expr {
            &Expression::Operand(ref operand) => self.evaluate_operand(operand),
            &Expression::Operation(ref operator, ref expr1, ref expr2) => {
                self.evaluate_operation(operator, expr1, expr2)
            }
        }
    }

    fn evaluate_operation(
        &mut self,
        operator: &Operator,
        operand1: &Expression,
        operand2: &Expression,
    ) -> Result<i64, Error> {
        let value1 = self.evaluate_expression(operand1)?;
        let value2 = self.evaluate_expression(operand2)?;
        Ok(match operator {
            &Operator::Add => value1 + value2,
            &Operator::Subtract => value1 - value2,
            &Operator::Multiply => value1 * value2,
            &Operator::Divide => value1 / value2,
        })
    }

    fn evaluate_operand(&mut self, operand: &Operand) -> Result<i64, Error> {
        match operand {
            &Operand::I64(value) => Ok(value),
            &Operand::Group(ref expr) => self.evaluate_expression(expr),
            &Operand::VarSubstitution(ref name) => self.variables
                .get(&name)
                .cloned()
                .ok_or_else(|| Error::UnknownVariable(name.clone())),
            &Operand::FnApplication(ref name, ref args) => self.evaluate_function(name, args),
        }
    }

    // FIXME: This allows functions to access global variables. Hmmm.
    fn evaluate_function(
        &mut self,
        name: &Name,
        arg_exprs: &Vec<Expression>,
    ) -> Result<i64, Error> {
        let (ref params, ref expr) = self.functions
            .get(&name)
            .ok_or_else(|| Error::UnknownFunction(name.clone()))?
            .clone();

        if params.len() != arg_exprs.len() {
            return Err(Error::IncorrectArgumentCount {
                name: name.clone(),
                params_count: params.len(),
                args_count: arg_exprs.len(),
            });
        }

        let mut args = Vec::new();
        for arg_expr in arg_exprs {
            args.push(self.evaluate_expression(arg_expr)?);
        }

        let backup_of_global_variables = self.variables.clone();
        self.variables = params.iter().cloned().zip(args.into_iter()).collect();
        let result = self.evaluate_expression(&expr);
        self.variables = backup_of_global_variables;
        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser::{expression, statement};

    #[test]
    fn var_can_be_redefined() {
        let mut e = Executor::new();
        e.execute(&statement(b"n = 1;").unwrap().1).unwrap();
        e.execute(&statement(b"n = 2;").unwrap().1).unwrap();
        assert_eq!(e.variables[&as_name("n")], 2);
    }

    #[test]
    fn var_can_be_redefined_using_itself() {
        let mut e = Executor::new();
        e.execute(&statement(b"a = 2;").unwrap().1).unwrap();
        e.execute(&statement(b"b = 3;").unwrap().1).unwrap();
        e.execute(&statement(b"a = a + b;").unwrap().1).unwrap();
        assert_eq!(e.variables[&as_name("a")], 5);
    }

    #[test]
    fn var_errors_if_undefined() {
        let mut e = Executor::new();
        assert_eq!(
            e.execute(&statement(b"a = b;").unwrap().1),
            Err(Error::UnknownVariable(as_name("b")))
        );
    }

    #[test]
    fn fn_can_be_redefined() {
        let mut e = Executor::new();
        e.execute(&statement(b"f(a) = a;").unwrap().1).unwrap();
        e.execute(&statement(b"f(a) = 3 * a;").unwrap().1).unwrap();
        assert_eq!(
            e.functions[&as_name("f")],
            (vec![as_name("a")], expression(b"3 * a;").unwrap().1,)
        );
    }

    #[test]
    fn fn_has_a_single_signature() {
        let mut e = Executor::new();
        e.execute(&statement(b"f(a) = a;").unwrap().1).unwrap();
        e.execute(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        assert_eq!(
            e.functions[&as_name("f")],
            (
                vec![as_name("a"), as_name("b")],
                expression(b"a + b;").unwrap().1,
            )
        );
    }

    #[test]
    fn fn_cannot_use_external_variables() {
        let mut e = Executor::new();
        e.execute(&statement(b"n = 1;").unwrap().1).unwrap();
        e.execute(&statement(b"f(a) = a * n;").unwrap().1).unwrap();
        assert_eq!(
            e.execute(&statement(b"j = f(2);").unwrap().1),
            Err(Error::UnknownVariable(as_name("n")))
        );
    }

    #[test]
    fn fn_params_can_reuse_external_names() {
        let mut e = Executor::new();
        e.execute(&statement(b"i = 1;").unwrap().1).unwrap();
        e.execute(&statement(b"f(i) = i;").unwrap().1).unwrap();
        e.execute(&statement(b"j = f(2);").unwrap().1).unwrap();
        assert_eq!(e.variables[&as_name("j")], 2);
    }

    #[test]
    fn fn_args_can_use_external_variables() {
        let mut e = Executor::new();
        e.execute(&statement(b"a = 1;").unwrap().1).unwrap();
        e.execute(&statement(b"f(b) = b;").unwrap().1).unwrap();
        e.execute(&statement(b"j = f(a);").unwrap().1).unwrap();
        assert_eq!(e.variables[&as_name("j")], 1);
    }

    #[test]
    fn fn_args_use_current_external_variables() {
        let mut e = Executor::new();
        e.execute(&statement(b"x = 1;").unwrap().1).unwrap();
        e.execute(&statement(b"f(a) = a;").unwrap().1).unwrap();
        e.execute(&statement(b"y = f(x);").unwrap().1).unwrap();
        e.execute(&statement(b"x = 2;").unwrap().1).unwrap();
        assert_eq!(e.variables[&as_name("y")], 1);
    }

    #[test]
    fn fn_errors_if_undefined() {
        let mut e = Executor::new();
        assert_eq!(
            e.execute(&statement(b"a = f(1);").unwrap().1),
            Err(Error::UnknownFunction(as_name("f")))
        );
    }

    #[test]
    fn var_and_fn_names_can_overlap() {
        let mut e = Executor::new();
        e.execute(&statement(b"a = 7;").unwrap().1).unwrap();
        e.execute(&statement(b"a(x, y) = x / y;").unwrap().1)
            .unwrap();
        e.execute(&statement(b"b = a;").unwrap().1).unwrap();
        e.execute(&statement(b"c = a(3, 2);").unwrap().1).unwrap();
        e.execute(&statement(b"d = a * a(a, 3);").unwrap().1)
            .unwrap();
        assert_eq!(e.variables[&as_name("a")], 7);
        assert_eq!(e.variables[&as_name("b")], 7);
        assert_eq!(e.variables[&as_name("c")], 3 / 2);
        assert_eq!(e.variables[&as_name("d")], 7 * (7 / 3));
    }

    #[test]
    fn vars_use_current_fn_definitions() {
        let mut e = Executor::new();
        e.execute(&statement(b"f(a) = a;").unwrap().1).unwrap();
        e.execute(&statement(b"x = f(1);").unwrap().1).unwrap();
        e.execute(&statement(b"f(a) = a + 1;").unwrap().1).unwrap();
        e.execute(&statement(b"y = f(1);").unwrap().1).unwrap();
        e.execute(&statement(b"f(a, b) = a + b;").unwrap().1)
            .unwrap();
        e.execute(&statement(b"z = f(1, 2);").unwrap().1).unwrap();
        assert_eq!(e.variables[&as_name("x")], 1);
        assert_eq!(e.variables[&as_name("y")], 2);
        assert_eq!(e.variables[&as_name("z")], 3);
    }

    fn as_name(s: &str) -> Name {
        Name(s.to_string())
    }
}
