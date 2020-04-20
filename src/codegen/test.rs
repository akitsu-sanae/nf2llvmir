use super::gen;
use crate::{BinOp, Expr, Func, Ident, Literal, Nf, Type};

fn codegen_check(nf: &Nf, expected: &str) {
    use std::io::BufWriter;
    let mut buf = vec![];
    {
        let mut out = BufWriter::new(&mut buf);
        assert_eq!(gen(&mut out, &nf, "output").unwrap(), ());
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
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @add(i32, i32) {
entry:
  %a = alloca i32
  store i32 %0, i32* %a
  %b = alloca i32
  store i32 %1, i32* %b
  %2 = load i32, i32* %a
  %3 = load i32, i32* %b
  %4 = add i32 %2, %3
  ret i32 %4
}

define i32 @main() {
entry:
  %0 = call i32 @add(i32 114, i32 514)
  %1 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.builtin.format.num, i32 0, i32 0), i32 %0)
  ret i32 0
}"#;
    codegen_check(&nf, expected.trim());
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
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  br i1 true, label %0, label %1

0:                                                ; preds = %entry
  br label %2

1:                                                ; preds = %entry
  br label %2

2:                                                ; preds = %1, %0
  %3 = phi i32 [ 42, %0 ], [ 32, %1 ]
  ret i32 %3
}
"#;
    codegen_check(&nf, expected.trim());
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
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %a = alloca i32
  store i32 42, i32* %a
  ret i32 2
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn assign_test() {
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
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %a = alloca i32
  store i32 42, i32* %a
  store i32 4, i32* %a
  %dummy = alloca i32*
  store i32* %a, i32** %dummy
  %0 = load i32, i32* %a
  ret i32 %0
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn const_array_test() {
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
            box Expr::PrintNum(box Expr::Load(box Expr::ArrayAt(
                box Expr::Var(Ident::new("arr")),
                box Expr::Const(Literal::Int(0)),
            ))),
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Void));
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"
@0 = constant [2 x i32] [i32 114, i32 514]

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %arr = alloca [2 x i32]
  %0 = bitcast [2 x i32]* %arr to i8*
  call void (i8*, i8*, ...) @memcpy(i8* %0, i8* bitcast ([2 x i32]* @0 to i8*), i64 mul nuw (i64 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i64), i64 2))
  %1 = getelementptr [2 x i32], [2 x i32]* %arr, i32 0, i32 0
  %2 = load i32, i32* %1
  %3 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.builtin.format.num, i32 0, i32 0), i32 %2)
  ret i32 %3
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn array_test() {
    // let a = 114;
    // let arr = {a, 514};
    // printnum arr[0];
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
                box Expr::PrintNum(box Expr::Load(box Expr::ArrayAt(
                    box Expr::Var(Ident::new("arr")),
                    box Expr::Const(Literal::Int(0)),
                ))),
            ),
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Void));
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %a = alloca i32
  store i32 114, i32* %a
  %0 = load i32, i32* %a
  %1 = alloca [2 x i32]
  %2 = getelementptr [2 x i32], [2 x i32]* %1, i32 0, i32 0
  store i32 %0, i32* %2
  %3 = getelementptr [2 x i32], [2 x i32]* %1, i32 0, i32 1
  store i32 514, i32* %3
  %arr = alloca [2 x i32]
  %4 = bitcast [2 x i32]* %arr to i8*
  %5 = bitcast [2 x i32]* %1 to i8*
  call void (i8*, i8*, ...) @memcpy(i8* %4, i8* %5, i64 mul nuw (i64 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i64), i64 2))
  %6 = getelementptr [2 x i32], [2 x i32]* %arr, i32 0, i32 0
  %7 = load i32, i32* %6
  %8 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.builtin.format.num, i32 0, i32 0), i32 %7)
  ret i32 %8
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn const_tuple_test() {
    // let a = (114, 514); a.0
    let nf = Nf {
        funcs: vec![],
        body: Expr::Let(
            Ident::new("a"),
            Type::Tuple(vec![Type::Int, Type::Int]),
            box Expr::Const(Literal::Tuple(vec![
                Expr::Const(Literal::Int(114)),
                Expr::Const(Literal::Int(514)),
            ])),
            box Expr::Load(box Expr::TupleAt(box Expr::Var(Ident::new("a")), 1)),
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"
@0 = constant { i32, i32 } { i32 114, i32 514 }

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %a = alloca { i32, i32 }
  %0 = bitcast { i32, i32 }* %a to i8*
  call void (i8*, i8*, ...) @memcpy(i8* %0, i8* bitcast ({ i32, i32 }* @0 to i8*), i64 mul nuw (i64 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i64), i64 2))
  %1 = getelementptr { i32, i32 }, { i32, i32 }* %a, i32 0, i32 1
  %2 = load i32, i32* %1
  ret i32 %2
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn tuple_test() {
    // let a = 1;
    // let tuple = (a, 2);
    // tuple.0
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
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %a = alloca i32
  store i32 1, i32* %a
  %0 = load i32, i32* %a
  %1 = alloca { i32, i32 }
  %2 = getelementptr { i32, i32 }, { i32, i32 }* %1, i32 0, i32 0
  store i32 %0, i32* %2
  %3 = getelementptr { i32, i32 }, { i32, i32 }* %1, i32 0, i32 1
  store i32 2, i32* %3
  %tuple = alloca { i32, i32 }
  %4 = bitcast { i32, i32 }* %tuple to i8*
  %5 = bitcast { i32, i32 }* %1 to i8*
  call void (i8*, i8*, ...) @memcpy(i8* %4, i8* %5, i64 mul nuw (i64 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i64), i64 2))
  %6 = getelementptr { i32, i32 }, { i32, i32 }* %tuple, i32 0, i32 1
  %7 = load i32, i32* %6
  ret i32 %7
}
"#;
    codegen_check(&nf, expected.trim());
}

#[test]
fn external_func_test() {
    // rand()
    let nf = Nf {
        funcs: vec![],
        body: Expr::Call(
            box Expr::Const(Literal::ExternalFunc(
                "rand".to_string(),
                Type::Func(vec![], box Type::Int),
            )),
            vec![],
        ),
    };
    assert_eq!(crate::typecheck::check(&nf), Ok(Type::Int));
    let expected = r#"
; ModuleID = 'output'
source_filename = "output"

@.builtin.format.num = global [3 x i8] c"%d\0A"

declare i32 @printf(i8*, ...)

declare void @memcpy(i8*, i8*, ...)

define i32 @main() {
entry:
  %0 = call i32 @rand()
  ret i32 %0
}

declare i32 @rand()
"#;
    codegen_check(&nf, expected.trim());
}
