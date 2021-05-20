use super::utils::{LoopNestVisitor, Nesting};
use super::WHILE_LET_ON_ITERATOR;
use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::higher;
use clippy_utils::source::snippet_with_applicability;
use clippy_utils::ty::implements_trait;
use clippy_utils::usage::mutated_variables;
use clippy_utils::{
    get_enclosing_block, is_refutable, is_trait_method, last_path_segment, path_to_local, path_to_local_id,
};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir::intravisit::{walk_block, walk_expr, NestedVisitorMap, Visitor};
use rustc_hir::{Expr, ExprKind, HirId, Node, PatKind};
use rustc_lint::LateContext;
use rustc_middle::hir::map::Map;
use rustc_span::symbol::sym;

pub(super) fn check(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
    if let Some(higher::WhileLet {
        let_pat,
        let_expr,
        if_then,
        ..
    }) = higher::WhileLet::hir(expr)
    {
        if let (
            PatKind::TupleStruct(qpath, pat_args, _),
            ExprKind::MethodCall(method_path, _, &[ref iter_self], _),
        ) = (&let_pat.kind, &let_expr.kind)
        {
            // Don't lint when the iterator is recreated on every iteration
            if_chain! {
                if let ExprKind::MethodCall(..) | ExprKind::Call(..) = iter_self.kind;
                if let Some(iter_def_id) = cx.tcx.get_diagnostic_item(sym::Iterator);
                if implements_trait(cx, cx.typeck_results().expr_ty(iter_self), iter_def_id, &[]);
                then {
                    return;
                }
            }

            let lhs_constructor = last_path_segment(qpath);
            if method_path.ident.name == sym::next
                && is_trait_method(cx, let_expr, sym::Iterator)
                && lhs_constructor.ident.name == sym::Some
                && (pat_args.is_empty()
                    || !is_refutable(cx, &pat_args[0])
                        && !is_used_inside(cx, iter_self, &if_then)
                        && !is_iterator_used_after_while_let(cx, iter_self)
                        && !is_loop_nested(cx, expr, iter_self))
            {
                let mut applicability = Applicability::MachineApplicable;
                let iterator = snippet_with_applicability(cx, iter_self.span, "_", &mut applicability);
                let loop_var = if pat_args.is_empty() {
                    "_".to_string()
                } else {
                    snippet_with_applicability(cx, pat_args[0].span, "_", &mut applicability).into_owned()
                };
                span_lint_and_sugg(
                    cx,
                    WHILE_LET_ON_ITERATOR,
                    expr.span.with_hi(let_expr.span.hi()),
                    "this loop could be written as a `for` loop",
                    "try",
                    format!("for {} in {}", loop_var, iterator),
                    applicability,
                );
            }
        }
    }
}

fn is_used_inside<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>, container: &'tcx Expr<'_>) -> bool {
    let def_id = match path_to_local(expr) {
        Some(id) => id,
        None => return false,
    };
    if let Some(used_mutably) = mutated_variables(container, cx) {
        if used_mutably.contains(&def_id) {
            return true;
        }
    }
    false
}

fn is_iterator_used_after_while_let<'tcx>(cx: &LateContext<'tcx>, iter_self: &'tcx Expr<'_>) -> bool {
    let iter_def_id = match path_to_local(iter_self) {
        Some(id) => id,
        None => return false,
    };
    let mut visitor = VarUsedAfterLoopVisitor {
        iter_def_id,
        iter_self_id: iter_self.hir_id,
        past_while_let: false,
        var_used_after_while_let: false,
    };
    if let Some(enclosing_block) = get_enclosing_block(cx, iter_def_id) {
        walk_block(&mut visitor, enclosing_block);
    }
    visitor.var_used_after_while_let
}

fn is_loop_nested(cx: &LateContext<'_>, loop_expr: &Expr<'_>, iter_self: &Expr<'_>) -> bool {
    let mut id = loop_expr.hir_id;
    let iter_id = if let Some(id) = path_to_local(iter_self) {
        id
    } else {
        return true;
    };
    loop {
        let parent = cx.tcx.hir().get_parent_node(id);
        if parent == id {
            return false;
        }
        match cx.tcx.hir().find(parent) {
            Some(Node::Expr(expr)) => {
                if let ExprKind::Loop(..) = expr.kind {
                    return true;
                };
            },
            Some(Node::Block(block)) => {
                let mut block_visitor = LoopNestVisitor {
                    hir_id: id,
                    iterator: iter_id,
                    nesting: Nesting::Unknown,
                };
                walk_block(&mut block_visitor, block);
                if block_visitor.nesting == Nesting::RuledOut {
                    return false;
                }
            },
            Some(Node::Stmt(_)) => (),
            _ => {
                return false;
            },
        }
        id = parent;
    }
}

struct VarUsedAfterLoopVisitor {
    iter_def_id: HirId,
    iter_self_id: HirId,
    past_while_let: bool,
    var_used_after_while_let: bool,
}

impl<'tcx> Visitor<'tcx> for VarUsedAfterLoopVisitor {
    type Map = Map<'tcx>;

    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if self.past_while_let {
            if path_to_local_id(expr, self.iter_def_id) {
                self.var_used_after_while_let = true;
            }
        } else if self.iter_self_id == expr.hir_id {
            self.past_while_let = true;
        }
        walk_expr(self, expr);
    }
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::None
    }
}
