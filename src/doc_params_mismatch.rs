use rustc::hir;
use rustc::hir::Item_;
use rustc::lint::{LateContext, LintContext, LintPass, LateLintPass, LintArray};
use syntax_pos::Span;
use syntax::ast;

use docstrings::parse_md_docblock;
use docstrings::DocSection;

use helpers::to_text_block;

declare_lint!(DOC_PARAMS_MISMATCH, Warn, "Warn about non-existing documented function parameters.");

pub struct DocParamsMismatch;

impl LintPass for DocParamsMismatch {
    fn get_lints(&self) -> LintArray {
        lint_array!(DOC_PARAMS_MISMATCH)
    }
}

impl DocParamsMismatch {
    fn lint(&self,
            cx: &LateContext,
            id: Option<ast::NodeId>,
            attrs: &[ast::Attribute],
            item: &Item_,
            sp: Span) {
        // If we're building a test harness, then warning about
        // documentation is probably not really relevant right now.
        if cx.sess().opts.test {
            return;
        }

        // Only check publicly-visible items, using the result from the privacy pass.
        // It's an option so the crate root can also use this function (it doesn't
        // have a NodeId).
        if let Some(id) = id {
            if !cx.access_levels.is_exported(id) {
                return;
            }
        }

        // println!("=====START BLOCK====="); // DEBUG

        if let &Item_::ItemFn(ref decl,_,_,_,_,_) = item {
            let input_names: Vec<_> = decl.inputs.iter().filter_map(|input|input.pat.simple_name()).map(|name| name.as_str().to_string()).collect();

            let text_block = to_text_block(attrs);
            if let Ok(parsed) = parse_md_docblock(&text_block) {

                let mut parameters = None;
                for section in parsed.sections {
                    match section {
                        DocSection::Parameters(params) => {
                            parameters = Some(params);
                        },
                        _ => (),
                    }
                }

                if let Some(params) = parameters {
                    for (identifier, _) in params {
                        if !input_names.contains(&identifier) {
                            cx.span_lint(DOC_PARAMS_MISMATCH,
                                         sp,
                                         &format!("The documented paramter '{}' does not exist for this function.",
                                             &identifier
                                         ));
                        }
                    }
                }
            }
        }

        // println!("=====END BLOCK====="); // DEBUG
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for DocParamsMismatch {
    fn check_item(&mut self, cx: &LateContext, it: &hir::Item) {
        // TODO: filter for ItemFn
        self.lint(cx, Some(it.id), &it.attrs, &it.node, it.span);
    }
}
