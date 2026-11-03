use alloc::vec::Vec;
use core::{iter::Peekable, ops::Range};
use logos::{Logos, SpannedIter};

use crate::{
    ast::{Ast, BinaryOp, Node},
    lexer::Token,
};

struct Parser<'a> {
    input: &'a str,
    tokens: Peekable<SpannedIter<'a, Token>>,
    nodes: Vec<Node<'a>>,
}

impl<'a> Parser<'a> {
    fn push(&mut self, node: Node<'a>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    fn expression(&mut self, minimum: u8, depth: usize) -> Result<usize, Range<usize>> {
        if depth >= 128 {
            return Err(self
                .tokens
                .peek()
                .map_or(self.input.len()..self.input.len(), |(_, span)| span.clone()));
        }

        let (token, span) = self
            .tokens
            .next()
            .ok_or(self.input.len()..self.input.len())?;

        let text = &self.input[span.clone()];

        let mut left = match token.map_err(|()| span.clone())? {
            Token::Number => self.push(Node::Number(text)),
            Token::Name => {
                if matches!(self.tokens.peek(), Some((Ok(Token::LeftParen), _))) {
                    self.tokens.next();
                    let mut args = Vec::new();

                    if matches!(self.tokens.peek(), Some((Ok(Token::RightParen), _))) {
                        self.tokens.next();
                    } else {
                        loop {
                            args.push(self.expression(0, depth + 1)?);

                            match self.tokens.next() {
                                Some((Ok(Token::Comma), _)) => {}
                                Some((Ok(Token::RightParen), _)) => break,
                                Some((_, span)) => return Err(span),
                                None => return Err(self.input.len()..self.input.len()),
                            }
                        }
                    }

                    self.push(Node::Call { name: text, args })
                } else {
                    self.push(Node::Symbol(text))
                }
            }

            Token::Plus => self.expression(5, depth + 1)?,

            Token::Minus => {
                let value = self.expression(5, depth + 1)?;
                self.push(Node::Neg(value))
            }

            Token::LeftParen => {
                let value = self.expression(0, depth + 1)?;

                match self.tokens.next() {
                    Some((Ok(Token::RightParen), _)) => value,
                    Some((_, span)) => return Err(span),
                    None => return Err(self.input.len()..self.input.len()),
                }
            }

            _ => return Err(span),
        };

        while let Some((token, span)) = self.tokens.peek() {
            let (op, left_power, right_power) = match token {
                Ok(Token::Plus) => (BinaryOp::Add, 1, 2),
                Ok(Token::Minus) => (BinaryOp::Sub, 1, 2),
                Ok(Token::Star) => (BinaryOp::Mul, 3, 4),
                Ok(Token::Slash) => (BinaryOp::Div, 3, 4),
                Ok(Token::Caret) => (BinaryOp::Pow, 7, 7),
                Err(()) => return Err(span.clone()),
                _ => break,
            };

            if left_power < minimum {
                break;
            }

            self.tokens.next();

            let right = self.expression(right_power, depth + 1)?;
            left = self.push(Node::Binary { op, left, right });
        }

        Ok(left)
    }
}

pub(crate) fn parse(input: &str) -> Result<Ast<'_>, Range<usize>> {
    let mut parser = Parser {
        input,
        tokens: Token::lexer(input).spanned().peekable(),
        nodes: Vec::new(),
    };

    let root = parser.expression(0, 0)?;

    if let Some((_, span)) = parser.tokens.next() {
        return Err(span);
    }

    Ok(Ast {
        nodes: parser.nodes,
        root,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::ast::{BinaryOp, Node};
    #[test]
    fn multiplication_binds_before_addition() {
        let ast = parse("2+3*4").unwrap();

        assert_eq!(ast.root, 4);
        assert_eq!(
            ast.nodes,
            [
                Node::Number("2"),
                Node::Number("3"),
                Node::Number("4"),
                Node::Binary {
                    op: BinaryOp::Mul,
                    left: 1,
                    right: 2,
                },
                Node::Binary {
                    op: BinaryOp::Add,
                    left: 0,
                    right: 3,
                },
            ]
        );
    }

    #[test]
    fn parses_arithmetic_grouping() {
        for (input, root, child_index, child) in [
            (
                "8/4/2",
                Node::Binary {
                    op: BinaryOp::Div,
                    left: 2,
                    right: 3,
                },
                2,
                Node::Binary {
                    op: BinaryOp::Div,
                    left: 0,
                    right: 1,
                },
            ),
            (
                "2^3^2",
                Node::Binary {
                    op: BinaryOp::Pow,
                    left: 0,
                    right: 3,
                },
                3,
                Node::Binary {
                    op: BinaryOp::Pow,
                    left: 1,
                    right: 2,
                },
            ),
            (
                "(2+3)*4",
                Node::Binary {
                    op: BinaryOp::Mul,
                    left: 2,
                    right: 3,
                },
                2,
                Node::Binary {
                    op: BinaryOp::Add,
                    left: 0,
                    right: 1,
                },
            ),
            (
                "-2^2",
                Node::Neg(2),
                2,
                Node::Binary {
                    op: BinaryOp::Pow,
                    left: 0,
                    right: 1,
                },
            ),
            (
                "2^-3",
                Node::Binary {
                    op: BinaryOp::Pow,
                    left: 0,
                    right: 2,
                },
                2,
                Node::Neg(1),
            ),
        ] {
            let ast = parse(input).unwrap();

            assert_eq!(ast.nodes[ast.root], root, "{input}");
            assert_eq!(ast.nodes[child_index], child, "{input}");
        }
    }

    #[test]
    fn rejects_incomplete_and_deep_expressions() {
        for (input, span) in [
            ("1+", 2..2),
            ("(1", 2..2),
            ("()", 1..2),
            ("1)", 1..2),
            ("1+π", 2..4),
            ("2**3", 2..3),
        ] {
            assert_eq!(parse(input), Err(span), "{input}");
        }

        let input = alloc::format!("{}1{}", "(".repeat(129), ")".repeat(129));

        assert!(parse(&input).is_err());
    }
    #[test]
    fn preserves_call_arguments() {
        let ast = parse("simplify(16/4)").unwrap();

        assert_eq!(ast.root, 3);
        assert_eq!(
            ast.nodes,
            [
                Node::Number("16"),
                Node::Number("4"),
                Node::Binary {
                    op: BinaryOp::Div,
                    left: 0,
                    right: 1,
                },
                Node::Call {
                    name: "simplify",
                    args: alloc::vec![2],
                },
            ]
        );
    }

    #[test]
    fn parses_argument_lists() {
        let ast = parse("f(x,g(2,3))").unwrap();

        assert_eq!(ast.nodes[0], Node::Symbol("x"));
        assert_eq!(
            ast.nodes[3],
            Node::Call {
                name: "g",
                args: alloc::vec![1, 2],
            }
        );
        assert_eq!(
            ast.nodes[ast.root],
            Node::Call {
                name: "f",
                args: alloc::vec![0, 3],
            }
        );

        let ast = parse("f()").unwrap();

        assert_eq!(
            ast.nodes[ast.root],
            Node::Call {
                name: "f",
                args: alloc::vec![],
            }
        );
    }

    #[test]
    fn rejects_invalid_calls() {
        for (input, span) in [
            ("f(", 2..2),
            ("f(1", 3..3),
            ("f(,1)", 2..3),
            ("f(1,)", 4..5),
            ("f(1,,2)", 4..5),
            ("f(1 2)", 4..5),
            ("f(π)", 2..4),
        ] {
            assert_eq!(parse(input), Err(span), "{input}");
        }

        let input = alloc::format!("{}1{}", "f(".repeat(129), ")".repeat(129));

        assert!(parse(&input).is_err());
    }
}
