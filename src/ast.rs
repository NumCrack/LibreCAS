use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Ast<'a> {
    pub nodes: Vec<Node<'a>>,
    pub root: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Node<'a> {
    Number(&'a str),
    Symbol(&'a str),
    Neg(usize),

    Binary {
        op: BinaryOp,
        left: usize,
        right: usize,
    },

    Call {
        name: &'a str,
        args: Vec<usize>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}