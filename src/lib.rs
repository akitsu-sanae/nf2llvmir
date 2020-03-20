#![feature(box_syntax)]
#![feature(box_patterns)]

extern crate libc;
extern crate llvm_sys as llvm;

mod codegen;
mod err_util;
mod printer;
mod typecheck;

#[derive(Debug)]
pub enum Error {
    Typecheck(typecheck::Error),
    Codegen(codegen::Error),
    Others(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);

impl Ident {
    pub fn new(name: &str) -> Ident {
        Ident(name.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nf {
    funcs: Vec<Func>,
    body: Expr,
}

impl Nf {
    pub fn codegen<T: std::io::Write>(&self, name: &str, out: &mut T) -> Result<(), Error> {
        typecheck::check(self)?;
        codegen::gen(out, self, name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub name: Ident,
    pub params: Vec<(Ident, Type)>,
    pub ret_type: Type,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Const(Literal),
    Var(Ident),
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    ArrayAt(Box<Expr>, Box<Expr>),
    StructAt(Box<Expr>, Ident),
    PrintNum(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Bool(bool),
    Char(char),
    Int(i32),
    Array(Vec<Expr>, Box<Type>),
    Struct(Vec<(Ident, Expr)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Void,
    Bool,
    Char,
    Int,
    Func(Vec<Type>, Box<Type>),
    Array(Box<Type>, usize),
    Struct(Vec<(Ident, Type)>),
}
