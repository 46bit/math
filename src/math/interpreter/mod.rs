use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownVariable(Name),
    UnknownFunction(Name),
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
        assert_eq!(program.inputs.len(), inputs.len());
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

    pub fn execute(&mut self, statement: &Statement) {
        match statement {
            &Statement::VarAssignment(ref name, ref expr) => {
                let expr_value = self.evaluate_expression(expr);
                self.variables.insert(name.clone(), expr_value);
            }
            &Statement::FnDefinition(ref name, ref params, ref expr) => {
                self.functions
                    .insert(name.clone(), (params.clone(), expr.clone()));
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> i64 {
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
    ) -> i64 {
        let value1 = self.evaluate_expression(operand1);
        let value2 = self.evaluate_expression(operand2);
        match operator {
            &Operator::Add => value1 + value2,
            &Operator::Subtract => value1 - value2,
            &Operator::Multiply => value1 * value2,
            &Operator::Divide => value1 / value2,
        }
    }

    fn evaluate_operand(&mut self, operand: &Operand) -> i64 {
        match operand {
            &Operand::I64(value) => value,
            &Operand::Group(ref expr) => self.evaluate_expression(expr),
            &Operand::VarSubstitution(ref name) => self.variables[name],
            &Operand::FnApplication(ref name, ref args) => self.evaluate_function(name, args),
        }
    }

    // 1. Verify number of args matches the expected number of params.
    // 2. Replace all args by their values.
    // 3. Make a backup copy of the current variables.
    // 4. Insert all evaluated args as variables.
    // 5. Evaluate the function's expression as normal.
    // 6. Restore the variables to the backup.
    // 7. Return the function's evaluated value.
    // FIXME: This allows functions to access global variables. Hmmm.
    fn evaluate_function(&mut self, name: &Name, args: &Vec<Expression>) -> i64 {
        let backup_of_global_variables = self.variables.clone();
        assert_eq!(args.len(), self.functions[name].0.len());
        for (i, arg_expr) in args.iter().enumerate() {
            let param_name = self.functions[name].0[i].clone();
            let arg_value = self.evaluate_expression(arg_expr);
            self.variables.insert(param_name, arg_value);
        }
        // FIXME: We're borrowing an Expression from `&self` and passing it to `&mut self`.
        // This forces a clone. I'd like to remove this but I'm not sure how, short of
        // copy-on-write or breaking functions out.
        let function_expr = self.functions[name].1.clone();
        let result = self.evaluate_expression(&function_expr);
        self.variables = backup_of_global_variables;
        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_assignment() {
        assert_eq!(
            execute(&Statements(vec![
                Statement::VarAssignment(as_name("i"), Expression::Operand(Operand::I64(1))),
            ])).map(pairs),
            Ok(vec![(as_name("i"), 1)])
        );
    }

    #[test]
    fn unused_fn_definition() {
        assert_eq!(
            execute(&Statements(vec![
                Statement::FnDefinition(
                    as_name("f"),
                    vec![as_name("a"), as_name("b")],
                    Expression::Operation(
                        Operator::Multiply,
                        box Expression::Operand(Operand::VarSubstitution(as_name("a"))),
                        box Expression::Operand(Operand::VarSubstitution(as_name("b"))),
                    ),
                ),
            ])).map(pairs),
            Ok(vec![])
        );
    }

    #[test]
    fn assignment_using_function() {
        assert_eq!(
            execute(&Statements(vec![
                Statement::FnDefinition(
                    as_name("f"),
                    vec![as_name("a"), as_name("b")],
                    Expression::Operation(
                        Operator::Multiply,
                        box Expression::Operand(Operand::VarSubstitution(as_name("a"))),
                        box Expression::Operand(Operand::VarSubstitution(as_name("b"))),
                    ),
                ),
                Statement::VarAssignment(
                    as_name("i"),
                    Expression::Operand(Operand::FnApplication(
                        as_name("f"),
                        vec![
                            Expression::Operand(Operand::I64(2)),
                            Expression::Operand(Operand::I64(3)),
                        ],
                    )),
                ),
            ])).map(pairs),
            Ok(vec![(as_name("i"), 6)])
        );
    }

    fn as_name(s: &str) -> Name {
        Name(s.to_string())
    }

    fn pairs(h: HashMap<Name, i64>) -> Vec<(Name, i64)> {
        h.into_iter().collect()
    }
}
