use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownVariable(Name),
    UnknownFunction(Name),
}

pub fn execute(statements: Vec<Statement>) -> Result<HashMap<Name, i64>, Error> {
    let mut executor = Executor::new();
    executor.run(statements);
    return Ok(executor.variables);
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

    pub fn run(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            self.execute(statement)
        }
    }

    pub fn execute(&mut self, statement: Statement) {
        match statement {
            Statement::VarAssignment(name, expr) => {
                let expr_value = self.evaluate_expression(expr);
                self.variables.insert(name, expr_value);
            }
            Statement::FnDefinition(name, params, expr) => {
                self.functions.insert(name, (params, expr));
            }
        }
    }

    // @TODO: Evaluation.
    // 1. Replace all operands by their values.
    // @TODO: 2. Devise queue of evaluations from highest to lowest priority operations.
    // 3. Perform series of evaluations.
    // 4. Return the expr's evaluated value.
    fn evaluate_expression(&mut self, expr: Expression) -> i64 {
        let mut accumulator = self.evaluate_operand(expr.0);
        for (operator, operand) in expr.1 {
            let operand_value = self.evaluate_operand(operand);
            accumulator = self.evaluate_operation(operator, accumulator, operand_value);
        }
        return accumulator;
    }

    fn evaluate_operation(&self, operator: Operator, operand1: i64, operand2: i64) -> i64 {
        match operator {
            Operator::Add => operand1 + operand2,
            Operator::Subtract => operand1 - operand2,
            Operator::Multiply => operand1 * operand2,
            Operator::Divide => operand1 / operand2,
        }
    }

    fn evaluate_operand(&mut self, operand: Operand) -> i64 {
        match operand {
            Operand::I64(value) => value,
            Operand::VarSubstitution(name) => self.variables[&name],
            Operand::FnApplication(name, args) => self.evaluate_function(name, args),
        }
    }

    // 1. Verify number of args matches the expected number of params.
    // 2. Replace all args by their values.
    // 3. Make a backup copy of the current variables.
    // 4. Insert all evaluated args as variables.
    // 5. Evaluate the function's expression as normal.
    // 6. Restore the variables to the backup.
    // 7. Return the function's evaluated value.
    // @TODO: This allows functions to access global variables. Hmmm.
    fn evaluate_function(&mut self, name: Name, args: Vec<Expression>) -> i64 {
        let (function_params, function_expr) = self.functions[&name].clone();
        assert_eq!(args.len(), function_params.len());
        let backup_of_global_variables = self.variables.clone();

        for (name, arg) in function_params.into_iter().zip(args) {
            let arg_value = self.evaluate_expression(arg);
            self.variables.insert(name.clone(), arg_value);
        }
        let result = self.evaluate_expression(function_expr);

        self.variables = backup_of_global_variables;
        return result;
    }
}
