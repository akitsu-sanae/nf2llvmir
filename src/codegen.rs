mod base;
mod build;
pub mod error;
mod lit;
mod typ;
mod util;

#[cfg(test)]
mod test;

pub use self::base::*;
pub use self::build::*;
pub use self::lit::*;
pub use self::typ::*;
pub use self::util::*;

use error::Error;

use crate::{env::Env, *};

pub fn gen<T: std::io::Write>(out: &mut T, nf: &Nf, name: &str) -> Result<(), error::Error> {
    let base = Base::new(name);
    apply_nf(&base, nf)?;
    util::validate_module(base.module)?;
    write!(out, "{}", util::print_module(base.module)?)?;
    Ok(())
}

fn apply_nf(base: &Base, nf: &Nf) -> Result<(), Error> {
    let mut env = Env::new();

    for func in nf.funcs.iter() {
        let gen_func = add_function(base, func)?;
        env = env.add(func.name.clone(), gen_func);
    }

    for func in nf.funcs.iter() {
        let gen_func = env.lookup(&func.name).unwrap();
        add_function_body(base, gen_func, func, &env)?;
    }

    let main_func = Func {
        name: Ident::new("main"),
        params: vec![],
        ret_type: Type::Int,
        body: nf.body.clone(),
    };
    let gen_main_func = add_function(base, &main_func)?;
    add_function_body(base, gen_main_func, &main_func, &env)?;
    Ok(())
}

fn add_function(base: &Base, func: &Func) -> Result<LValue, Error> {
    let param_types: Result<_, _> = func
        .params
        .iter()
        .map(|&(_, ref ty)| apply_type(ty, base))
        .collect();
    let mut param_types = param_types?;
    let ret_ty = apply_type(&func.ret_type, base)?;
    let func_ty = typ::func(&mut param_types, ret_ty);
    Ok(util::add_function(
        base.module,
        func.name.0.as_str(),
        func_ty,
    ))
}

fn add_function_body(
    base: &Base,
    gen_func: LValue,
    func: &Func,
    env: &Env<LValue>,
) -> Result<(), Error> {
    util::add_entry_block(gen_func, base);
    let mut env = env.clone();
    let param_types: Result<_, _> = func
        .params
        .iter()
        .map(|&(_, ref ty)| apply_type(ty, base))
        .collect();
    let param_types: Vec<_> = param_types?;

    for (i, param) in func.params.iter().enumerate() {
        let typ = param_types[i];
        let var = build::declare(
            (param.0).0.as_str(),
            typ,
            util::get_func_param(gen_func, i),
            base.builder,
        );
        env = env.add(param.0.clone(), var);
    }

    let expr = apply_expr(&func.body, &env, base)?;
    build::ret(expr, base.builder);
    Ok(())
}

fn apply_expr(e: &Expr, env: &Env<LValue>, base: &Base) -> Result<LValue, Error> {
    match e {
        Expr::Const(ref lit) => apply_literal(lit, env, base),
        Expr::Let(ref name, ref typ, box ref e1, box ref e2) => {
            if typ == &Type::Void {
                apply_expr(e1, env, base)?;
                apply_expr(e2, env, base)
            } else {
                let typ = if let Type::Func(from, box to) = typ {
                    Type::Pointer(box Type::Func(from.clone(), box to.clone()))
                } else {
                    typ.clone()
                };
                let l_typ = apply_type(&typ, base)?;
                let l_e1 = apply_expr(e1, env, base)?;
                let var = match &typ {
                    Type::Array(_, _) => build::declare_array(&name.0, l_typ, l_e1, base),
                    Type::Tuple(_) => build::declare_tuple(&name.0, l_typ, l_e1, base),
                    _ => build::declare(&name.0, l_typ, l_e1, base.builder),
                };
                let env = env.add(name.clone(), var);
                apply_expr(e2, &env, base)
            }
        }
        Expr::Var(ref name) => env
            .lookup(name)
            .ok_or(Error::Internal(format!("unbound variable: {}", name))),
        Expr::Load(box ref e) => {
            let e = apply_expr(e, env, base)?;
            Ok(build::load(e, base.builder))
        }
        Expr::Assign(box ref e1, box ref e2) => {
            let lhs = apply_expr(e1, env, base)?;
            let rhs = apply_expr(e2, env, base)?;
            Ok(build::store(lhs, rhs, base.builder))
        }
        Expr::Call(box ref func, ref args) => {
            let func = apply_expr(func, env, base)?;
            let args: Result<_, _> = args.iter().map(|arg| apply_expr(arg, env, base)).collect();
            let mut args = args?;
            Ok(build::call(func, &mut args, base.builder))
        }
        Expr::If(box ref cond, box ref e1, box ref e2) => apply_if_expr(cond, e1, e2, env, base),
        Expr::BinOp(op, box ref e1, box ref e2) => apply_binop_expr(op, e1, e2, env, base),
        Expr::ArrayAt(box ref arr, box ref i) => apply_array_at(arr, i, env, base),
        Expr::TupleAt(box ref e, ref idx) => apply_tuple_at(e, *idx, env, base),
        Expr::PrintNum(box ref e) => apply_printnum_expr(e, env, base),
    }
}

fn apply_literal(lit: &Literal, env: &Env<LValue>, base: &Base) -> Result<LValue, Error> {
    match lit {
        Literal::Bool(b) => Ok(lit::bool(*b, base.context)),
        Literal::Int(n) => Ok(lit::int32(*n, base.context)),
        Literal::Char(c) => Ok(lit::char(*c, base.context)),
        Literal::Array(ref arr, ref elem_ty) => {
            let elem_ty = apply_type(elem_ty, base)?;
            let arr: Result<_, _> = arr.iter().map(|e| apply_expr(e, env, base)).collect();
            let arr = arr?;
            Ok(lit::array(arr, elem_ty, base))
        }
        Literal::Tuple(ref elems) => {
            let elems: Result<_, _> = elems.iter().map(|e| apply_expr(e, env, base)).collect();
            let elems = elems?;
            Ok(lit::tuple(elems, base))
        }
        Literal::ExternalFunc(ref name, ref typ) => {
            let typ = apply_type(typ, base)?;
            Ok(lit::external_func(name.clone(), typ, base.module))
        }
    }
}

fn apply_if_expr(
    cond: &Expr,
    e1: &Expr,
    e2: &Expr,
    env: &Env<LValue>,
    base: &Base,
) -> Result<LValue, Error> {
    let cond = apply_expr(cond, env, base)?;
    let insertion_block = util::insertion_block(base.builder);
    let then_block = append_block(insertion_block, base);
    let else_block = append_block(then_block, base);
    let merge_block = append_block(else_block, base);

    build::cond_branch(cond, then_block, else_block, base.builder);

    // code generation for then-block
    util::position_at_end(then_block, base.builder);
    let e1 = apply_expr(e1, env, base)?;
    build::branch(merge_block, base.builder);
    let then_block = util::insertion_block(base.builder);

    // code generation for else-block
    util::position_at_end(else_block, base.builder);
    let e2 = apply_expr(e2, env, base)?;
    build::branch(merge_block, base.builder);
    let else_block = util::insertion_block(base.builder);

    // code generation for merge-block
    util::position_at_end(merge_block, base.builder);
    Ok(build::phi(
        typ::type_of(e1),
        vec![(e1, then_block), (e2, else_block)],
        base.builder,
    ))
}

fn apply_binop_expr(
    op: &BinOp,
    e1: &Expr,
    e2: &Expr,
    env: &Env<LValue>,
    base: &Base,
) -> Result<LValue, Error> {
    let e1 = apply_expr(e1, env, base)?;
    let e2 = apply_expr(e2, env, base)?;
    match op {
        BinOp::Add => Ok(build::add(e1, e2, base.builder)),
        BinOp::Sub => Ok(build::sub(e1, e2, base.builder)),
        BinOp::Mult => Ok(build::mult(e1, e2, base.builder)),
        BinOp::Div => Ok(build::div(e1, e2, base.builder)),
        BinOp::Eq => Ok(build::eq(e1, e2, base.builder)),
        BinOp::Neq => Ok(build::neq(e1, e2, base.builder)),
        BinOp::Lt => Ok(build::lt(e1, e2, base.builder)),
        BinOp::Gt => Ok(build::gt(e1, e2, base.builder)),
        BinOp::Leq => Ok(build::leq(e1, e2, base.builder)),
        BinOp::Geq => Ok(build::geq(e1, e2, base.builder)),
    }
}

fn apply_array_at(arr: &Expr, idx: &Expr, env: &Env<LValue>, base: &Base) -> Result<LValue, Error> {
    let arr = apply_expr(arr, env, base)?;
    let idx = apply_expr(idx, env, base)?;
    Ok(build::gep(arr, idx, base))
}

fn apply_tuple_at(e: &Expr, idx: usize, env: &Env<LValue>, base: &Base) -> Result<LValue, Error> {
    let e = apply_expr(e, env, base)?;
    let idx = lit::int32(idx as i32, base.context);
    Ok(build::gep(e, idx, base))
}

fn apply_printnum_expr(e: &Expr, env: &Env<LValue>, base: &Base) -> Result<LValue, Error> {
    let e = apply_expr(e, env, base)?;
    Ok(build::builtin::print_num(e, base))
}

fn apply_type(ty: &Type, base: &Base) -> Result<LType, Error> {
    match ty {
        Type::Void => Ok(typ::void(base.context)),
        Type::Bool => Ok(typ::bool(base.context)),
        Type::Char => Ok(typ::char(base.context)),
        Type::Int => Ok(typ::int32(base.context)),
        Type::Func(ref params, box ret_ty) => {
            let params: Result<_, _> = params.iter().map(|ty| apply_type(ty, base)).collect();
            let mut params = params?;
            let ret_ty = apply_type(ret_ty, base)?;
            Ok(typ::func(&mut params, ret_ty))
        }
        Type::Array(box ref elem_ty, ref len) => Ok(typ::array(apply_type(elem_ty, base)?, *len)),
        Type::Pointer(box ref ty) => Ok(typ::ptr(apply_type(ty, base)?)),
        Type::Tuple(ref elems) => {
            let elems: Result<_, _> = elems.iter().map(|ty| apply_type(ty, base)).collect();
            let elems = elems?;
            Ok(typ::tuple(elems))
        }
    }
}
