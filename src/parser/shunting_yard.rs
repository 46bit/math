use super::super::*;

#[derive(Debug, Clone)]
pub struct ShuntingYard {
    operator_stack: Vec<Operator>,
    expr_stack: Vec<Expression>,
}

impl ShuntingYard {
    pub fn new(first_operand: Operand) -> ShuntingYard {
        ShuntingYard {
            operator_stack: vec![],
            expr_stack: vec![Expression::Operand(first_operand)],
        }
    }

    pub fn push(&mut self, operator: Operator, operand: Operand) {
        loop {
            let end_operator = self.operator_stack.last().cloned();
            match end_operator {
                Some(end_operator) if end_operator >= operator => self.make_a_tree(),
                _ => break,
            }
        }
        self.operator_stack.push(operator);
        self.expr_stack.push(Expression::Operand(operand));
    }

    fn make_a_tree(&mut self) {
        let operator = self.operator_stack.pop().unwrap();
        let (arg_b, arg_a) = (
            self.expr_stack.pop().unwrap(),
            self.expr_stack.pop().unwrap(),
        );
        let expr = Expression::Operation(operator, box arg_a, box arg_b);
        self.expr_stack.push(expr);
    }

    pub fn into_expression(mut self) -> Expression {
        while !self.operator_stack.is_empty() {
            self.make_a_tree();
        }
        assert_eq!(self.expr_stack.len(), 1);
        self.expr_stack.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use quickcheck::{QuickCheck, StdGen};

    #[test]
    fn lone_operand_test() {
        let first_operand = Operand::I64(-3);

        let shunting_yard = ShuntingYard::new(first_operand.clone());
        assert_eq!(
            shunting_yard.expr_stack,
            vec![Expression::Operand(first_operand.clone())]
        );
        assert_eq!(shunting_yard.operator_stack, vec![]);

        let expr = shunting_yard.into_expression();
        assert_eq!(expr, Expression::Operand(first_operand));
    }

    #[test]
    fn single_operation_test() {
        let operator = Operator::Multiply;
        let operands = (Operand::I64(1), Operand::I64(2));

        let mut shunting_yard = ShuntingYard::new(operands.0.clone());
        assert_eq!(
            shunting_yard.expr_stack,
            vec![Expression::Operand(operands.0.clone())]
        );
        assert_eq!(shunting_yard.operator_stack, vec![]);

        shunting_yard.push(operator, operands.1.clone());
        assert_eq!(
            shunting_yard.expr_stack,
            vec![
                Expression::Operand(operands.0.clone()),
                Expression::Operand(operands.1.clone()),
            ]
        );
        assert_eq!(shunting_yard.operator_stack, vec![operator]);

        assert_eq!(
            shunting_yard.into_expression(),
            Expression::Operation(
                operator,
                box Expression::Operand(operands.0),
                box Expression::Operand(operands.1)
            )
        );
    }

    #[test]
    fn operator_priority_test() {
        let tokens = (
            Operand::I64(-3),
            Operator::Divide,
            Operand::I64(7),
            Operator::Add,
            Operand::I64(11),
        );

        let mut shunting_yard = ShuntingYard::new(tokens.0.clone());
        assert_eq!(
            shunting_yard.expr_stack,
            vec![Expression::Operand(tokens.0.clone())]
        );
        assert_eq!(shunting_yard.operator_stack, vec![]);

        shunting_yard.push(tokens.1, tokens.2.clone());
        assert_eq!(
            shunting_yard.expr_stack,
            vec![
                Expression::Operand(tokens.0.clone()),
                Expression::Operand(tokens.2.clone()),
            ]
        );
        assert_eq!(shunting_yard.operator_stack, vec![tokens.1]);

        shunting_yard.push(tokens.3, tokens.4.clone());
        assert_eq!(
            shunting_yard.expr_stack,
            vec![
                Expression::Operation(
                    tokens.1,
                    box Expression::Operand(tokens.0.clone()),
                    box Expression::Operand(tokens.2.clone()),
                ),
                Expression::Operand(tokens.4.clone()),
            ]
        );
        assert_eq!(shunting_yard.operator_stack, vec![tokens.3]);

        assert_eq!(
            shunting_yard.into_expression(),
            Expression::Operation(
                tokens.3,
                box Expression::Operation(
                    tokens.1,
                    box Expression::Operand(tokens.0),
                    box Expression::Operand(tokens.2)
                ),
                box Expression::Operand(tokens.4)
            )
        );
    }

    fn shunts_correctly_prop(input: Expression) -> bool {
        let tokens = input.tokens();
        let mut shunting_yard = match tokens[0].clone() {
            Token::Operand(first_operand) => ShuntingYard::new(first_operand),
            _ => unreachable!(),
        };
        for chunk in tokens[1..].chunks(2) {
            let (operator, operand) = match chunk {
                &[Token::Operator(operator), Token::Operand(ref operand)] => (operator, operand),
                _ => unreachable!(),
            };
            shunting_yard.push(operator, operand.clone());
        }
        let output = shunting_yard.into_expression();
        format!("{}", input) == format!("{}", output)
    }

    #[test]
    fn shunts_correctly() {
        // QuickCheck's default size creates infeasibly vast statements, and beyond some
        // point they stop exploring novel code paths. This does a much better job of
        // exploring potential edgecases.
        for size in 1..11 {
            let mut qc = QuickCheck::new().gen(StdGen::new(thread_rng(), size));
            qc.quickcheck(shunts_correctly_prop as fn(Expression) -> bool);
        }
    }
}
