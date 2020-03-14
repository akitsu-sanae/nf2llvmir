#![feature(box_syntax)]
#![feature(box_patterns)]

extern crate libc;
extern crate llvm_sys as llvm;

mod typecheck;

#[derive(Debug, Clone)]
pub struct Ident(String);

impl Ident {
    pub fn new(name: &str) -> Ident {
        Ident(name.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Nf {
    lets: Vec<Let>,
    body: Expr,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: Ident,
    pub params: Vec<(Ident, Type)>,
    pub ret_type: Type,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Const(Literal),
    Var(Ident),
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    PrintNum(Box<Expr>),
    // TODO: access to elements of array or struct
}

#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Char(char),
    Int(i32),
    Func(Ident),
    // TODO: add array and struct
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
}

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Bool,
    Char,
    Int,
    Func(Vec<Type>, Box<Type>),
    // TODO: add type of array and struct
}
