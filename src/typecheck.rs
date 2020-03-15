use crate::{BinOp, Expr, Ident, Literal, Nf, Type};
use std::collections::HashMap;

type Error = String; // TODO

type TypeEnv = HashMap<Ident, Type>;

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
        None => Err(format!("unbound variable: {}", name)),
    }
}

fn check_expr(e: &Expr, env: &TypeEnv) -> Result<Type, Error> {
    match e {
        Expr::Const(lit) => Ok(check_literal(lit)),
        Expr::Var(name) => lookup(env, &name),
        Expr::Call(box ref e, ref args) => {
            if let Type::Func(params, box ret_type) = check_expr(e, env)? {
                let args: Result<_, _> = args.iter().map(|arg| check_expr(arg, env)).collect();
                let args: Vec<Type> = args?;
                if params == args {
                    Ok(ret_type)
                } else {
                    Err(format!(
                        "type of params {:?} and args {:?} must match", // TODO: not to use debug formatter
                        params, args
                    ))
                }
            } else {
                Err(format!("{:?} must have function type", e))
            }
        }
        Expr::If(box ref cond, box ref e1, box ref e2) => {
            if check_expr(cond, env)? == Type::Bool {
                let ty1 = check_expr(e1, env)?;
                let ty2 = check_expr(e2, env)?;
                if ty1 == ty2 {
                    Ok(ty1)
                } else {
                    Err(format!("{:?} and {:?} must match", ty1, ty2))
                }
            } else {
                Err(format!("{:?} must be boolean", cond))
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
                _ => Err(format!(
                    "cannot apply operator {:?} for {:?} and {:?}",
                    op, e1, e2
                )),
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

#[test]
fn primitive_test() {
    let nf = Nf {
        funcs: vec![],
        body: Expr::Const(Literal::Int(42)),
    };
    assert_eq!(check(&nf), Ok(Type::Int));

    let nf = Nf {
        funcs: vec![],
        body: Expr::Const(Literal::Bool(true)),
    };
    assert_eq!(check(&nf), Ok(Type::Bool));

    let nf = Nf {
        funcs: vec![],
        body: Expr::Const(Literal::Char('c')),
    };
    assert_eq!(check(&nf), Ok(Type::Char));
}

#[test]
fn func_test() {
    use crate::Func;
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("a"),
            params: vec![],
            ret_type: Type::Int,
            body: Expr::Const(Literal::Int(42)),
        }],
        body: Expr::Var(Ident::new("a")),
    };
    assert_eq!(check(&nf), Ok(Type::Func(vec![], box Type::Int)));
}

#[test]
fn apply_test() {
    use crate::Func;
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("a"),
            params: vec![],
            ret_type: Type::Int,
            body: Expr::Const(Literal::Int(42)),
        }],
        body: Expr::Call(box Expr::Var(Ident::new("a")), vec![]),
    };
    assert_eq!(check(&nf), Ok(Type::Int));
}
