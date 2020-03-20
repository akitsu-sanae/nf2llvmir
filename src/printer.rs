use crate::*;
use std::fmt;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Typecheck(err) => write!(f, "{}", err),
            Codegen(err) => write!(f, "{}", err),
            Others(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Expr::*;
        match self {
            Const(ref lit) => write!(f, "{}", lit),
            Var(ref name) => write!(f, "{}", name),
            Call(box ref func, ref args) => write!(f, "{}({})", func, {
                let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
                args.join(", ")
            }),
            If(box ref cond, box ref e1, box ref e2) => {
                write!(f, "if {} then {} else {}", cond, e1, e2)
            }
            BinOp(ref op, box ref e1, box ref e2) => write!(f, "({}) {} ({})", e1, op, e2),
            ArrayAt(box ref arr, box ref idx) => write!(f, "{}[{}]", arr, idx),
            StructAt(box ref e, ref label) => write!(f, "{}.{}", e, label),
            PrintNum(box ref e) => write!(f, "printnum {}", e),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Literal::*;
        match self {
            Bool(b) => write!(f, "{}", b),
            Char(c) => write!(f, "{}", c),
            Int(n) => write!(f, "{}", n),
            Array(ref arr, _) => write!(f, "[{}]", {
                let arr: Vec<_> = arr.iter().map(|e| e.to_string()).collect();
                arr.join(", ")
            }),
            Struct(ref fields) => write!(f, "{{ {}  }}", {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|(label, e)| format!("{}: {}", label, e))
                    .collect();
                fields.join(", ")
            }),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mult => "*",
                BinOp::Div => "/",
                BinOp::Eq => "==",
                BinOp::Neq => "/=",
                BinOp::Lt => "<",
                BinOp::Gt => ">",
                BinOp::Leq => "<=",
                BinOp::Geq => ">=",
            }
        )
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::Int => write!(f, "int"),
            Type::Func(ref params, box ref ret_ty) => write!(
                f,
                "({}) -> {}",
                {
                    let params: Vec<String> =
                        params.iter().map(|param| param.to_string()).collect();
                    params.join(", ")
                },
                ret_ty
            ),
            Type::Array(box ref elem_ty, ref len) => write!(f, "{}[{}]", elem_ty, len),
            Type::Struct(ref fields) => write!(f, "{{ {} }}", {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|(label, ty)| format!("{}: {}", label, ty))
                    .collect();
                fields.join(", ")
            }),
        }
    }
}
