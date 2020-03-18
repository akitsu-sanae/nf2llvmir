use super::gen;
use crate::{Expr, Func, Ident, Literal, Nf, Type};

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
fn array_test() {
    let nf = Nf {
        funcs: vec![Func {
            name: Ident::new("a"),
            params: vec![(Ident::new("idx"), Type::Int)],
            ret_type: Type::Int,
            body: Expr::ArrayAt(
                box Expr::Const(Literal::Array(
                    vec![
                        Expr::Const(Literal::Int(1)),
                        Expr::Const(Literal::Int(2)),
                        Expr::Const(Literal::Int(3)),
                    ],
                    box Type::Int,
                )),
                box Expr::Var(Ident::new("idx")),
            ),
        }],
        body: Expr::Call(
            box Expr::Var(Ident::new("a")),
            vec![Expr::Const(Literal::Int(1))],
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
