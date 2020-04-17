pub mod error;

#[cfg(test)]
mod test;

use crate::{env::Env, *};
use error::Error;

pub fn check(nf: &Nf) -> Result<Type, Error> {
    let mut env = Env::new();
    for func in nf.funcs.iter() {
        let params: Vec<Type> = func.params.iter().map(|param| param.1.clone()).collect();
        env = env.add(
            func.name.clone(),
            Type::Func(params, box func.ret_type.clone()),
        )
    }

    for func in nf.funcs.iter() {
        let mut env = env.clone();
        for (ref name, ref ty) in func.params.iter() {
            env = env.add(name.clone(), ty.clone());
        }
        check_expr(&func.body, &env)?;
    }
    check_expr(&nf.body, &env)
}

fn check_expr(e: &Expr, env: &Env<Type>) -> Result<Type, Error> {
    match e {
        Expr::Const(lit) => check_literal(lit, env),
        Expr::Let(ref name, ref typ, box ref e1, box ref e2) => {
            let typ1 = check_expr(e1, env)?;
            if typ != &typ1 {
                return Err(Error::UnmatchLet(e1.clone(), typ1));
            }
            let mut env = env.clone();
            env = env.add(name.clone(), typ1);
            check_expr(e2, &env)
        }
        Expr::Var(ref name) => {
            let ty = env
                .lookup(name)
                .ok_or(Error::UnboundVariable(name.clone()))?;
            Ok(match ty {
                Type::Array(_, _) | Type::Tuple(_) => ty,
                _ => Type::Pointer(box ty),
            })
        }
        Expr::Load(box ref e) => {
            if let Type::Pointer(box ty) = check_expr(e, env)? {
                Ok(ty)
            } else {
                Err(Error::DereferenceNonpointer(e.clone()))
            }
        }
        Expr::Call(box ref e, ref args) => {
            let e_ty = check_expr(e, env)?;
            if let Type::Pointer(box Type::Func(params, box ret_type)) = e_ty {
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
                    Ok(Type::Pointer(box elem_ty))
                } else {
                    Err(Error::IndexingForNonArray(arr.clone(), arr_ty))
                }
            } else {
                Err(Error::IndexingWithNonInteger(idx.clone(), idx_ty))
            }
        }
        Expr::TupleAt(box ref e, ref idx) => {
            if let Type::Tuple(elems) = check_expr(e, env)? {
                if let Some(ty) = elems.into_iter().nth(*idx) {
                    Ok(Type::Pointer(box ty))
                } else {
                    Err(Error::InvalidTupleAccess(e.clone(), *idx))
                }
            } else {
                Err(Error::IndexingForNonTuple(e.clone()))
            }
        }
        Expr::PrintNum(box ref e) => {
            check_expr(e, env)?;
            Ok(Type::Void)
        }
    }
}

fn check_literal(lit: &Literal, env: &Env<Type>) -> Result<Type, Error> {
    match lit {
        Literal::Bool(_) => Ok(Type::Bool),
        Literal::Char(_) => Ok(Type::Char),
        Literal::Int(_) => Ok(Type::Int),
        Literal::Array(elems, ref ty) => {
            for e in elems.iter() {
                let given_ty = check_expr(e, env)?;
                if ty != &given_ty {
                    return Err(Error::UnmatchArrayElem(e.clone(), ty.clone()));
                }
            }
            Ok(Type::Array(box ty.clone(), elems.len()))
        }
        Literal::Tuple(ref elems) => {
            let elems: Result<Vec<Type>, _> = elems.iter().map(|e| check_expr(e, env)).collect();
            Ok(Type::Tuple(elems?))
        }
        Literal::ExternalFunc(_, typ) => Ok(Type::Pointer(box typ.clone())),
    }
}
