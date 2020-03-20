use super::*;
use std::error;
use std::fmt;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            UnboundVariable(name) => write!(f, "unbound variable: {}", name),
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
            InvalidField(e, label) => write!(f, "expr {} does not have field named {}", e, label),
            FieldOfNonStruct(e) => write!(f, "{} is not struct expr", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
