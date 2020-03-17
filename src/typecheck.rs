use crate::{BinOp, Expr, Ident, Literal, Nf, Type};
use std::collections::HashMap;

mod err_util;

#[cfg(test)]
mod test;

type TypeEnv = HashMap<Ident, Type>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnboundVariable(Ident),
    UnmatchParamsAndArgs(Expr, Vec<Type>, Vec<Type>),
    ApplyNonFunc(Expr, Type),
    UnmatchIfBranches(Expr, Type, Type),
    UnmatchIfCond(Expr, Type),
    InvalidBinOp(BinOp, Expr, Expr),
}

pub fn check(nf: &Nf) -> Result<Type, Error> {
    let mut env = TypeEnv::new();
    for func in nf.funcs.iter() {
        let params: Vec<Type> = func.params.iter().map(|param| param.1.clone()).collect();
        env.insert(
            func.name.clone(),
            Type::Func(params, box func.ret_type.clone()),
        );
    }

    for func in nf.funcs.iter() {
        let mut env = env.clone();
        for (ref name, ref ty) in func.params.iter() {
            env.insert(name.clone(), ty.clone());
        }
        check_expr(&func.body, &env)?;
    }
    check_expr(&nf.body, &env)
}

fn lookup(env: &TypeEnv, name: &Ident) -> Result<Type, Error> {
    match env.get(name) {
        Some(typ) => Ok(typ.clone()),
        None => Err(Error::UnboundVariable(name.clone())),
    }
}

fn check_expr(e: &Expr, env: &TypeEnv) -> Result<Type, Error> {
    match e {
        Expr::Const(lit) => Ok(check_literal(lit)),
        Expr::Var(name) => lookup(env, &name),
        Expr::Call(box ref e, ref args) => {
            let e_ty = check_expr(e, env)?;
            if let Type::Func(params, box ret_type) = e_ty {
                let args: Result<_, _> = args.iter().map(|arg| check_expr(arg, env)).collect();
                let args: Vec<Type> = args?;
                if params == args {
                    Ok(ret_type)
                } else {
                    Err(Error::UnmatchParamsAndArgs(e.clone(), params, args))
                }
            } else {
                Err(Error::ApplyNonFunc(e.clone(), e_ty))
            }
        }
        Expr::If(box ref cond, box ref e1, box ref e2) => {
            let cond_ty = check_expr(cond, env)?;
            if cond_ty == Type::Bool {
                let ty1 = check_expr(e1, env)?;
                let ty2 = check_expr(e2, env)?;
                if ty1 == ty2 {
                    Ok(ty1)
                } else {
                    Err(Error::UnmatchIfBranches(
                        Expr::If(box cond.clone(), box e1.clone(), box e2.clone()),
                        ty1,
                        ty2,
                    ))
                }
            } else {
                Err(Error::UnmatchIfCond(cond.clone(), cond_ty))
            }
        }
        Expr::BinOp(ref op, box ref e1, box ref e2) => {
            let ty1 = check_expr(e1, env)?;
            let ty2 = check_expr(e2, env)?;
            match (op, ty1, ty2) {
                (BinOp::Add, Type::Int, Type::Int)
                | (BinOp::Sub, Type::Int, Type::Int)
                | (BinOp::Mult, Type::Int, Type::Int)
                | (BinOp::Div, Type::Int, Type::Int) => Ok(Type::Int),

                (BinOp::Eq, Type::Bool, Type::Bool)
                | (BinOp::Neq, Type::Bool, Type::Bool)
                | (BinOp::Eq, Type::Int, Type::Int)
                | (BinOp::Neq, Type::Int, Type::Int)
                | (BinOp::Lt, Type::Int, Type::Int)
                | (BinOp::Gt, Type::Int, Type::Int)
                | (BinOp::Leq, Type::Int, Type::Int)
                | (BinOp::Geq, Type::Int, Type::Int) => Ok(Type::Bool),
                _ => Err(Error::InvalidBinOp(op.clone(), e1.clone(), e2.clone())),
            }
        }
        Expr::PrintNum(box ref e) => {
            check_expr(e, env)?;
            Ok(Type::Void)
        }
    }
}

fn check_literal(lit: &Literal) -> Type {
    match lit {
        Literal::Bool(_) => Type::Bool,
        Literal::Char(_) => Type::Char,
        Literal::Int(_) => Type::Int,
    }
}
