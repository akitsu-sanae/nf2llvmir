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
