use alloc::{vec, vec::Vec};
use num_bigint::BigInt;
use num_rational::BigRational;

use crate::ast::{Ast, BinaryOp, Node};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EvalError {
    Unsupported,
    OutsideDomain,
    InvalidAst,
}

enum Task<'a> {
    Visit(usize),
    Negate,
    Apply(&'a BinaryOp),
}

pub(crate) fn evaluate(ast: &Ast<'_>) -> Result<BigRational, EvalError> {
    let mut pending = vec![Task::Visit(ast.root)];
    let mut values: Vec<BigRational> = Vec::new();

    while let Some(task) = pending.pop() {
        match task {
            Task::Visit(index) => {
                match ast.nodes.get(index).ok_or(EvalError::InvalidAst)? {
                    Node::Number(text) => {
                        let integer = BigInt::parse_bytes(text.as_bytes(), 10)
                            .ok_or(EvalError::Unsupported)?;

                        values.push(BigRational::from_integer(integer));
                    }

                    Node::Neg(child) => {
                        pending.push(Task::Negate);
                        pending.push(Task::Visit(*child));
                    }

                    Node::Binary { op, left, right } => {
                        pending.push(Task::Apply(op));
                        pending.push(Task::Visit(*right));
                        pending.push(Task::Visit(*left));
                    }

                    Node::Symbol(_) | Node::Call { .. } => {
                        return Err(EvalError::Unsupported);
                    }
                }
            }

            Task::Negate => {
                let value = values.pop().ok_or(EvalError::InvalidAst)?;
                values.push(-value);
            }

            Task::Apply(op) => {
                let right = values.pop().ok_or(EvalError::InvalidAst)?;
                let left = values.pop().ok_or(EvalError::InvalidAst)?;

                let value = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,

                    BinaryOp::Div => {
                        if right.numer() == &BigInt::from(0) {
                            return Err(EvalError::OutsideDomain);
                        }

                        left / right
                    }

                    BinaryOp::Pow => return Err(EvalError::Unsupported),
                };

                values.push(value);
            }
        }
    }

    if values.len() != 1 {
        return Err(EvalError::InvalidAst);
    }

    values.pop().ok_or(EvalError::InvalidAst)
}

#[cfg(test)]
mod tests {
    use super::{evaluate, EvalError};
    use crate::parser::parse;
    use num_bigint::BigInt;
    use num_rational::BigRational;

    #[test]
    fn evaluates_exact_arithmetic() {
        for (input, numerator, denominator) in [
            ("2+3*4", 14, 1),
            ("(2+3)*4", 20, 1),
            ("1/3+1/6", 1, 2),
            ("-8/3", -8, 3),
            ("8/4/2", 1, 1),
        ] {
            let ast = parse(input).unwrap();
            let expected =
                BigRational::new(numerator.into(), denominator.into());

            assert_eq!(evaluate(&ast), Ok(expected), "{input}");
        }

        let ast = parse("18446744073709551616+1").unwrap();
        let expected = BigRational::from_integer(BigInt::from(
            18_446_744_073_709_551_617u128,
        ));

        assert_eq!(evaluate(&ast), Ok(expected));
    }

    #[test]
    fn evaluates_long_expressions() {
        let input = alloc::vec!["1"; 1000].join("+");
        let ast = parse(&input).unwrap();
        let expected = BigRational::from_integer(BigInt::from(1000));

        assert_eq!(evaluate(&ast), Ok(expected));
    }

    #[test]
    fn rejects_unsupported_or_outside_domain_inputs() {
        for (input, error) in [
            ("x+1", EvalError::Unsupported),
            ("simplify(16/4)", EvalError::Unsupported),
            ("1.5", EvalError::Unsupported),
            ("1e3", EvalError::Unsupported),
            ("2^3", EvalError::Unsupported),
            ("1/0", EvalError::OutsideDomain),
            ("0/0", EvalError::OutsideDomain),
        ] {
            let ast = parse(input).unwrap();

            assert_eq!(evaluate(&ast), Err(error), "{input}");
        }
    }
}