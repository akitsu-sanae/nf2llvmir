use super::*;
use crate::*;

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
    assert_eq!(
        check(&nf),
        Ok(Type::Pointer(box Type::Func(vec![], box Type::Int)))
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
        body: Expr::Call(box Expr::Var(Ident::new("a")), vec![]),
    };
    assert_eq!(check(&nf), Ok(Type::Int));
}

#[test]
fn array_test() {
    use crate::Func;
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("a"),
            params: vec![(Ident::new("idx"), Type::Int)],
            ret_type: Type::Int,
            body: Expr::Load(box Expr::ArrayAt(
                box Expr::Const(Literal::Array(
                    vec![
                        Expr::Const(Literal::Int(1)),
                        Expr::Const(Literal::Int(2)),
                        Expr::Const(Literal::Int(3)),
                    ],
                    Type::Int,
                )),
                box Expr::Load(box Expr::Var(Ident::new("idx"))),
            )),
        }],
        body: Expr::Call(
            box Expr::Var(Ident::new("a")),
            vec![Expr::Const(Literal::Int(1))],
        ),
    };
    assert_eq!(check(&nf), Ok(Type::Int));
}
