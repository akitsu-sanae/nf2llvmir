mod base;
mod build;
mod lit;
mod typ;
mod util;

pub use self::base::*;
pub use self::build::*;
pub use self::lit::*;
pub use self::typ::*;
pub use self::util::*;

use crate::{Error, Expr, Func, Ident, Literal, Nf, Type};
use std::collections::HashMap;
use std::io::Write;

pub fn gen<T: Write>(out: &mut T, nf: &Nf, name: &str) -> Result<(), Error> {
    let mut base = Base::new(name);
    apply_nf(&mut base, nf)?;
    util::validate_module(base.module)?;
    write!(out, "{}", util::print_module(base.module)?).unwrap(); // TODO
    Ok(())
}

type Env = HashMap<Ident, LValue>;

fn apply_nf(base: &mut Base, nf: &Nf) -> Result<(), Error> {
    let mut env = Env::new();

    for func in nf.funcs.iter() {
        let gen_func = add_function(base, func)?;
        env.insert(func.name.clone(), gen_func);
    }

    for func in nf.funcs.iter() {
        let gen_func = env.get(&func.name).unwrap();
        add_function_body(base, *gen_func, func, &env)?;
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

fn add_function(base: &mut Base, func: &Func) -> Result<LValue, Error> {
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
    base: &mut Base,
    gen_func: LValue,
    func: &Func,
    env: &Env,
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
        env.insert(param.0.clone(), var);
    }

    let expr = apply_expr(&func.body, &env, base)?;
    build::ret(expr, base.builder);
    Ok(())
}

fn apply_expr(e: &Expr, env: &Env, base: &Base) -> Result<LValue, Error> {
    match e {
        Expr::Const(ref lit) => apply_literal(lit, env, base),
        Expr::Var(ref name) => env
            .get(name)
            .cloned()
            .ok_or(format!("unbound variable: {}", name)),
        Expr::Call(box ref func, ref args) => {
            let func = apply_expr(func, env, base)?;
            let args: Result<_, _> = args.iter().map(|arg| apply_expr(arg, env, base)).collect();
            let mut args = args?;
            Ok(build::call(func, &mut args, base.builder))
        }
        _ => unimplemented!(),
    }
}

fn apply_literal(lit: &Literal, _env: &Env, base: &Base) -> Result<LValue, Error> {
    match lit {
        Literal::Bool(b) => Ok(lit::bool(*b, base.context)),
        Literal::Int(n) => Ok(lit::int32(*n, base.context)),
        _ => unimplemented!(),
    }
}

fn apply_type(ty: &Type, base: &Base) -> Result<LType, Error> {
    match ty {
        Type::Void => Ok(typ::void(base.context)),
        Type::Bool => Ok(typ::bool(base.context)),
        Type::Char => Ok(typ::char(base.context)),
        Type::Int => Ok(typ::int32(base.context)),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
fn codegen_check(nf: &Nf, expected: &str) {
    use std::io::BufWriter;
    let mut buf = vec![];
    {
        let mut out = BufWriter::new(&mut buf);
        assert_eq!(gen(&mut out, &nf, "output"), Ok(()));
    }
    let mut gen_code = std::str::from_utf8(&buf).unwrap().to_string();
    gen_code.pop(); // remove eof
    assert_eq!(gen_code.as_str().trim(), expected);
}

#[test]
fn primitive_test() {
    let nf = Nf {
        funcs: vec![],
        body: Expr::Const(Literal::Int(42)),
    };
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  ret i32 42
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn func_test() {
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("a"),
            params: vec![],
            ret_type: Type::Int,
            body: Expr::Const(Literal::Int(42)),
        }],
        body: Expr::Call(box Expr::Var(Ident::new("a")), vec![]),
    };
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @a() {
entry:
  ret i32 42
}

define i32 @main() {
entry:
  %0 = call i32 @a()
  ret i32 %0
}
"#;
    codegen_check(&nf, expected.trim());
}
