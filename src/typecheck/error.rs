use super::*;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnboundVariable(Ident),
    UnmatchLet(Expr, Type),
    UnmatchParamsAndArgs(Expr, Vec<Type>, Vec<Type>),
    ApplyNonFunc(Expr, Type),
    UnmatchIfBranches(Expr, Type, Type),
    UnmatchIfCond(Expr, Type),
    DereferenceNonpointer(Expr),
    AssignToNonpointer(Expr),
    UnmatchAssign(Expr, Type, Type),
    InvalidBinOp(BinOp, Expr, Expr),
    IndexingForNonArray(Expr, Type),
    IndexingWithNonInteger(Expr, Type),
    UnmatchArrayElem(Expr, Type),
    InvalidTupleAccess(Expr, usize),
    IndexingForNonTuple(Expr),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            UnboundVariable(name) => write!(f, "unbound variable: {}", name),
            UnmatchLet(e, typ) => write!(f, "{} is expected to have type {}", e, typ),
            UnmatchParamsAndArgs(e, params, args) => write!(
                f,
                "in {}, params are expected in {}, but given in {}",
                e,
                {
                    let params: Vec<String> =
                        params.iter().map(|param| param.to_string()).collect();
                    params.join(", ")
                },
                {
                    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
                    args.join(", ")
                }
            ),
            ApplyNonFunc(e, ty) => write!(f, "{} must have function type, but have {}", e, ty),
            UnmatchIfBranches(e, ty1, ty2) => write!(
                f,
                "in {}, branches have different type, {} vs {}",
                e, ty1, ty2
            ),
            UnmatchIfCond(e, ty) => write!(
                f,
                "cond in if-expr, {}, must have bool type, but have {}",
                e, ty
            ),
            DereferenceNonpointer(e) => write!(
                f,
                "dereferenced expression, `{}`, does not have pointer type",
                e
            ),
            AssignToNonpointer(e) => write!(f, "assign to non-pointer {}", e),
            UnmatchAssign(e, left_ty, right_ty) => write!(
                f,
                "{} was expected to be {}, but actually {}",
                e, left_ty, right_ty
            ),
            InvalidBinOp(op, e1, e2) => write!(
                f,
                "invalid operation application, {} for {} and {}",
                op, e1, e2
            ),
            IndexingForNonArray(e, ty) => write!(
                f,
                "indexed expr {} must have array type, but have {}",
                e, ty
            ),
            IndexingWithNonInteger(e, ty) => write!(
                f,
                "indexing expr {} must have integer type, but have {}",
                e, ty
            ),
            UnmatchArrayElem(e, ty) => write!(f, "elem {} in array must have {}", e, ty),
            InvalidTupleAccess(e, idx) => write!(f, "invalid access of expr {}, with {}", e, idx),
            IndexingForNonTuple(e) => write!(f, "{} is not struct expr", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
