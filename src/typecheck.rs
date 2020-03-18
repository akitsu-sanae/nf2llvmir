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
    IndexingForNonArray(Expr, Type),
    IndexingWithNonInteger(Expr, Type),
    UnmatchArrayElem(Expr, Type),
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
        Expr::Const(lit) => check_literal(lit, env),
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
        Expr::ArrayAt(box ref arr, box ref idx) => {
            let arr_ty = check_expr(arr, env)?;
            let idx_ty = check_expr(idx, env)?;
            if idx_ty == Type::Int {
                if let Type::Array(box elem_ty, _) = arr_ty {
                    Ok(elem_ty)
                } else {
                    Err(Error::IndexingForNonArray(arr.clone(), arr_ty))
                }
            } else {
                Err(Error::IndexingWithNonInteger(idx.clone(), idx_ty))
            }
        }
        Expr::PrintNum(box ref e) => {
            check_expr(e, env)?;
            Ok(Type::Void)
        }
    }
}

fn check_literal(lit: &Literal, env: &TypeEnv) -> Result<Type, Error> {
    match lit {
        Literal::Bool(_) => Ok(Type::Bool),
        Literal::Char(_) => Ok(Type::Char),
        Literal::Int(_) => Ok(Type::Int),
        Literal::Array(elems, box ref ty) => {
            for e in elems.iter() {
                let given_ty = check_expr(e, env)?;
                if ty != &given_ty {
                    return Err(Error::UnmatchArrayElem(e.clone(), ty.clone()));
                }
            }
            Ok(Type::Array(box ty.clone(), elems.len()))
        }
    }
}
