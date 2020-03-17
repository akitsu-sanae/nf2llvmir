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
            ApplyNonFunc(_e, _ty) => unimplemented!(),
            UnmatchIfBranches(_e, _ty1, _ty2) => unimplemented!(),
            UnmatchIfCond(_e, _ty) => unimplemented!(),
            InvalidBinOp(_op, _e1, _e2) => unimplemented!(),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
