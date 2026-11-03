#![no_std]

extern crate alloc;

mod ast;
mod eval;
mod lexer;
mod parser;

use alloc::string::{String, ToString};
use core::{fmt, ops::Range};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Syntax(Range<usize>),
    Unsupported,
    OutsideDomain,
    Internal,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(span) => write!(f, "Syntax error at bytes {}..{}", span.start, span.end),
            Self::Unsupported => f.write_str("Unsupported expression"),
            Self::OutsideDomain => f.write_str("Special numeric values are not supported yet"),
            Self::Internal => f.write_str("Internal evaluation error"),
        }
    }
}

pub fn execute(input: &str) -> Result<String, Error> {
    if input.trim().is_empty() {
        return Ok(String::new());
    }

    let ast = parser::parse(input).map_err(Error::Syntax)?;

    let value = eval::evaluate(&ast).map_err(|error| match error {
        eval::EvalError::Unsupported => Error::Unsupported,
        eval::EvalError::OutsideDomain => Error::OutsideDomain,
        eval::EvalError::InvalidAst => Error::Internal,
    })?;

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Error, execute};

    #[test]
    fn executes_and_formats_arithmetic() {
        for (input, expected) in [("2+3*4", "14"), ("1/3+1/6", "1/2"), ("16/4", "4")] {
            assert_eq!(execute(input).as_deref(), Ok(expected), "{input}");
        }
    }

    #[test]
    fn reports_errors() {
        for (input, expected) in [
            ("-", Error::Syntax(1..1)),
            ("  1+@", Error::Syntax(4..5)),
            ("1+", Error::Syntax(2..2)),
            ("1+@", Error::Syntax(2..3)),
            ("1e+", Error::Syntax(1..2)),
            ("factor(12)", Error::Unsupported),
            ("1/0", Error::OutsideDomain),
        ] {
            assert_eq!(execute(input), Err(expected), "{input}");
        }
    }
}
