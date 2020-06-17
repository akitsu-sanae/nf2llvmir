use super::*;
use crate::*;

#[test]
fn primitive_test() {
    let nf = Nf {
        funcs: vec![],
        body: Some(Expr::Const(Literal::Int(42))),
    };
    assert_eq!(check(&nf), Ok(Some(Type::Int)));

    let nf = Nf {
        funcs: vec![],
        body: Some(Expr::Const(Literal::Bool(true))),
    };
    assert_eq!(check(&nf), Ok(Some(Type::Bool)));

    let nf = Nf {
        funcs: vec![],
        body: Some(Expr::Const(Literal::Char('c'))),
    };
    assert_eq!(check(&nf), Ok(Some(Type::Char)));
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
        body: Some(Expr::Var(Ident::new("a"))),
    };
    assert_eq!(
        check(&nf),
        Ok(Some(Type::Pointer(box Type::Func(vec![], box Type::Int))))
    );
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
        body: Some(Expr::Call(box Expr::Var(Ident::new("a")), vec![])),
    };
    assert_eq!(check(&nf), Ok(Some(Type::Int)));
}

#[test]
fn array_test() {
    // int[2] arr = {114, 514};
    // return (load arr[0]);
    let nf = Nf {
        funcs: vec![],
        body: Some(Expr::Let(
            Ident::new("arr"),
            Type::Array(box Type::Int, 2),
            box Expr::Const(Literal::Array(
                vec![
                    Expr::Const(Literal::Int(114)),
                    Expr::Const(Literal::Int(514)),
                ],
                Type::Int,
            )),
            box Expr::Load(box Expr::ArrayAt(
                box Expr::Var(Ident::new("arr")),
                box Expr::Const(Literal::Int(0)),
            )),
        )),
    };
    assert_eq!(check(&nf), Ok(Some(Type::Int)));
}
