use super::gen;
use crate::{BinOp, Expr, Func, Ident, Literal, Nf, Type};

fn codegen_check(nf: &Nf, name: &str, expected_output: &str, expected_status: i32) {
    use std::{fs, process::Command, str};
    let filename = format!("./test-output/{}", name);
    {
        let mut f = fs::File::create(&filename).unwrap();
        assert_eq!(gen(&mut f, &nf, name).unwrap(), ());
    }
    {
        let result = Command::new("lli")
            .arg(&filename)
            .output()
            .expect("failed to execute lli");
        let output = str::from_utf8(&result.stdout).expect("unrecognized output");
        assert_eq!(output, expected_output);
        assert_eq!(result.status.code(), Some(expected_status));
    }
}

#[test]
fn primitive_test() {
    let nf = Nf {
        funcs: vec![],
        body: Expr::Const(Literal::Int(42)),
    };
    codegen_check(&nf, "primitive", "", 42);
}

#[test]
fn func_test() {
    // int add(int a, int b) { return (load a) + (load b); }
    // let (): Void = printnum (add (114, 514));
    // return 0;
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("add"),
            params: vec![(Ident::new("a"), Type::Int), (Ident::new("b"), Type::Int)],
            ret_type: Type::Int,
            body: Expr::BinOp(
                BinOp::Add,
                box Expr::Load(box Expr::Var(Ident::new("a"))),
                box Expr::Load(box Expr::Var(Ident::new("b"))),
            ),
        }],
        body: Expr::Let(
            Ident::new("dummy"),
            Type::Void,
            box Expr::PrintNum(box Expr::Call(
                box Expr::Var(Ident::new("add")),
                vec![
                    Expr::Const(Literal::Int(114)),
                    Expr::Const(Literal::Int(514)),
                ],
            )),
            box Expr::Const(Literal::Int(0)),
        ),
    };
    codegen_check(&nf, "func", "628\n", 0);
}

#[test]
fn if_test() {
    let nf = Nf {
        funcs: vec![],
        body: Expr::If(
            box Expr::Const(Literal::Bool(true)),
            box Expr::Const(Literal::Int(42)),
            box Expr::Const(Literal::Int(32)),
        ),
    };
    codegen_check(&nf, "if", "", 42);
}

#[test]
fn let_test() {
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
            Ident::new("a"),
            Type::Int,
            box Expr::Const(Literal::Int(42)),
            box Expr::Const(Literal::Int(2)),
        ),
    };
    codegen_check(&nf, "let", "", 2);
}

#[test]
fn assign_test() {
    // int a = 42;
    // int* dummy = (a <- 4);
    // return (load a);
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
            Ident::new("a"),
            Type::Int,
            box Expr::Const(Literal::Int(42)),
            box Expr::Let(
                Ident::new("dummy"),
                Type::Pointer(box Type::Int),
                box Expr::Assign(
                    box Expr::Var(Ident::new("a")),
                    box Expr::Const(Literal::Int(4)),
                ),
                box Expr::Load(box Expr::Var(Ident::new("a"))),
            ),
        ),
    };
    codegen_check(&nf, "assign", "", 4);
}

#[test]
fn const_array_test() {
    // int[2] arr = {114, 514};
    // return (load arr[0]);
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
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
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    codegen_check(&nf, "const_array", "", 114);
}

#[test]
fn array_test() {
    // int a = 114;
    // int[2] arr = {a, 514};
    // return arr[0];
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
            Ident::new("a"),
            Type::Int,
            box Expr::Const(Literal::Int(114)),
            box Expr::Let(
                Ident::new("arr"),
                Type::Array(box Type::Int, 2),
                box Expr::Const(Literal::Array(
                    vec![
                        Expr::Load(box Expr::Var(Ident::new("a"))),
                        Expr::Const(Literal::Int(514)),
                    ],
                    Type::Int,
                )),
                box Expr::Load(box Expr::ArrayAt(
                    box Expr::Var(Ident::new("arr")),
                    box Expr::Const(Literal::Int(0)),
                )),
            ),
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    codegen_check(&nf, "array", "", 114);
}

#[test]
fn const_tuple_test() {
    // let a = (114, 514);
    // printnum a.1;
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
            Ident::new("a"),
            Type::Tuple(vec![Type::Int, Type::Int]),
            box Expr::Const(Literal::Tuple(vec![
                Expr::Const(Literal::Int(114)),
                Expr::Const(Literal::Int(514)),
            ])),
            box Expr::Let(
                Ident::new("dummy"),
                Type::Void,
                box Expr::PrintNum(box Expr::Load(box Expr::TupleAt(
                    box Expr::Var(Ident::new("a")),
                    1,
                ))),
                box Expr::Const(Literal::Int(0)),
            ),
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    codegen_check(&nf, "const_tuple", "514\n", 0);
}

#[test]
fn tuple_test() {
    // let a = 1;
    // let tuple = (a, 2);
    // tuple.1
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
            Ident::new("a"),
            Type::Int,
            box Expr::Const(Literal::Int(1)),
            box Expr::Let(
                Ident::new("tuple"),
                Type::Tuple(vec![Type::Int, Type::Int]),
                box Expr::Const(Literal::Tuple(vec![
                    Expr::Load(box Expr::Var(Ident::new("a"))),
                    Expr::Const(Literal::Int(2)),
                ])),
                box Expr::Load(box Expr::TupleAt(box Expr::Var(Ident::new("tuple")), 1)),
            ),
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    codegen_check(&nf, "tuple", "", 2);
}

#[test]
fn tuple_arg_test() {
    // func foo x:{int, int}: int {
    //     return 42;
    // }
    // foo({30, 12})
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("foo"),
            params: vec![(Ident::new("x"), Type::Tuple(vec![Type::Int, Type::Int]))],
            ret_type: Type::Int,
            body: Expr::Const(Literal::Int(42)),
        }],
        body: Expr::Call(
            box Expr::Var(Ident::new("foo")),
            vec![Expr::Const(Literal::Tuple(vec![
                Expr::Const(Literal::Int(30)),
                Expr::Const(Literal::Int(12)),
            ]))],
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    codegen_check(&nf, "tuple-arg", "", 42);
}

#[test]
fn external_func_test() {
    // rand()
    let nf = Nf {
        funcs: vec![],
        body: Expr::Call(
            box Expr::Const(Literal::ExternalFunc(
                "abs".to_string(),
                Type::Func(vec![Type::Int], box Type::Int),
            )),
            vec![Expr::Const(Literal::Int(-3))],
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    codegen_check(&nf, "rand", "", 3); // first value when seed is 1 (default)
}
