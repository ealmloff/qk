use crate::component::Component;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Type};
use syn::{ExprPath, Pat, PathArguments, PathSegment};

pub struct ComponentVisitorMut<'a> {
    pub memo_idx: usize,
    pub state_idx: usize,
    pub component: &'a Component,
}

impl VisitMut for ComponentVisitorMut<'_> {
    fn visit_stmt_mut(&mut self, i: &mut syn::Stmt) {
        let mut memo = None;
        let mut state = None;

        match i {
            syn::Stmt::Local(i) => {
                if let Pat::Type(pat_ty) = &i.pat {
                    if let Type::Path(path) = &*pat_ty.ty {
                        let segments = &path.path.segments;
                        if let Some(PathSegment {
                            ident,
                            arguments: PathArguments::AngleBracketed(ty),
                        }) = segments.first()
                        {
                            if segments.len() == 1 && ident == "Rx" {
                                if let Some(syn::GenericArgument::Type(ty)) = ty.args.first() {
                                    if let Pat::Ident(name) = &*pat_ty.pat {
                                        let maybe_state = &self.component.states[self.state_idx];
                                        self.state_idx += 1;
                                        assert_eq!(maybe_state.name, name.ident);
                                        assert_eq!(&maybe_state.ty, ty);

                                        state = Some(maybe_state);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            syn::Stmt::Semi(Expr::Call(expr), _) | syn::Stmt::Expr(Expr::Call(expr)) => {
                if let Expr::Path(ExprPath { path, .. }) = &*expr.func {
                    if let Some(fn_name) = path.get_ident() {
                        if fn_name == "rx" {
                            if let Some(Expr::Closure(closure)) = expr.args.first().cloned() {
                                let maybe_memo = &self.component.memos[self.memo_idx];
                                self.memo_idx += 1;
                                assert_eq!(maybe_memo.closure.as_ref().unwrap(), &*closure.body);

                                memo = Some(maybe_memo);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if let Some(memo) = memo {
            *i = memo.construct(self.component);
        } else if let Some(state) = state {
            *i = state.construct();
        }

        visit_mut::visit_stmt_mut(self, i);
    }
}
