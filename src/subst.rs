use crate::*;
use ident::Ident;

impl Func {
    pub fn subst_expr(mut self: Func, name: &Ident, e: &Expr) -> Func {
        self.body = self.body.subst_expr(name, e);
        self
    }
}

impl Expr {
    pub fn subst_expr(self: Expr, name: &Ident, e: &Expr) -> Expr {
        match self {
            Expr::Const(Literal::Bool(_))
            | Expr::Const(Literal::Char(_))
            | Expr::Const(Literal::Int(_)) => self,
            Expr::Const(Literal::Array(es, typ)) => Expr::Const(Literal::Array(
                es.into_iter().map(|e_| e_.subst_expr(name, e)).collect(),
                typ,
            )),
            Expr::Const(Literal::Tuple(elems)) => Expr::Const(Literal::Tuple(
                elems.into_iter().map(|e_| e_.subst_expr(name, e)).collect(),
            )),
            Expr::Const(Literal::ExternalFunc(_, _)) => self,
            Expr::Let(ref name_, _, _, _) if name_ == name => self,
            Expr::Let(name_, typ, box e1, box e2) => Expr::Let(
                name_,
                typ,
                box e1.subst_expr(name, e),
                box e2.subst_expr(name, e),
            ),
            Expr::Var(name_) if &name_ == name => e.clone(),
            Expr::Var(_) => self,
            Expr::Load(box e_) => Expr::Load(box e_.subst_expr(name, e)),
            Expr::Assign(box e1, box e2) => {
                Expr::Assign(box e1.subst_expr(name, e), box e2.subst_expr(name, e))
            }
            Expr::Call(box f, args) => Expr::Call(
                box f.subst_expr(name, e),
                args.into_iter().map(|e_| e_.subst_expr(name, e)).collect(),
            ),
            Expr::If(box cond, box e1, box e2) => Expr::If(
                box cond.subst_expr(name, e),
                box e1.subst_expr(name, e),
                box e2.subst_expr(name, e),
            ),
            Expr::BinOp(op, box e1, box e2) => {
                Expr::BinOp(op, box e1.subst_expr(name, e), box e2.subst_expr(name, e))
            }
            Expr::ArrayAt(box arr, box idx) => {
                Expr::ArrayAt(box arr.subst_expr(name, e), box idx.subst_expr(name, e))
            }
            Expr::TupleAt(box e_, label) => Expr::TupleAt(box e_.subst_expr(name, e), label),
            Expr::PrintNum(box e_) => Expr::PrintNum(box e_.subst_expr(name, e)),
        }
    }
}
