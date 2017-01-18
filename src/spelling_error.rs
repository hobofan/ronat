use markdown;
use rustc::hir;
use rustc::lint::{LateContext, LintContext, LintPass, LateLintPass, LintArray};
use syntax_pos::Span;
use syntax::ast;
use syntax::attr;

use ispell::{SpellLauncher, SpellChecker};
use markdown::{Block, Span as MdSpan};

use helpers::to_text_block;

declare_lint!(SPELLING_ERROR, Warn, "Warn about spelling errors.");

pub struct SpellingError {
    // /// Stack of IDs of struct definitions.
    // struct_def_stack: Vec<ast::NodeId>,

    /// Stack of whether #[doc(hidden)] is set
    /// at each level which has lint attributes.
    doc_hidden_stack: Vec<bool>,
}

impl LintPass for SpellingError {
    fn get_lints(&self) -> LintArray {
        lint_array!(SPELLING_ERROR)
    }
}

impl SpellingError {
    pub fn new() -> SpellingError {
        SpellingError {
            // struct_def_stack: vec![],
            doc_hidden_stack: vec![false],
        }
    }

    fn doc_hidden(&self) -> bool {
        *self.doc_hidden_stack.last().expect("empty doc_hidden_stack")
    }

    fn travserse_markdown_block(checker: &mut SpellChecker, cx: &LateContext, sp: Span, block: &Block) {
        match *block {
            Block::Header(ref spans, _) => {
                for span in spans {
                    Self::travserse_markdown_span(checker, cx, sp, span);
                }
            },
            Block::Paragraph(ref spans) => {
                for span in spans {
                    Self::travserse_markdown_span(checker, cx, sp, span);
                }
            }
            Block::Blockquote(_) => {}, // TODO
            Block::CodeBlock(_) => (),
            Block::UnorderedList(_) => {}, // TODO
            Block::Raw(_) => {}, // TODO
            Block::Hr => {},
        }
    }

    fn travserse_markdown_span(checker: &mut SpellChecker, cx: &LateContext, sp: Span, span: &MdSpan) {
        match *span {
            MdSpan::Break => (),
            MdSpan::Text(ref text) => {
                Self::check_text(checker, cx, sp, text.clone());
            },
            MdSpan::Code(_) => (),
            MdSpan::Link(_, _, _) => {}, // TODO
            MdSpan::Image(_, _, _) => {}, // TODO: check alt text?
            MdSpan::Emphasis(ref spans) => {
                for span in spans {
                    Self::travserse_markdown_span(checker, cx, sp, span);
                }
            },
            MdSpan::Strong(ref spans) => {
                for span in spans {
                    Self::travserse_markdown_span(checker, cx, sp, span);
                }
            },
        }
    }

    fn check_text(checker: &mut SpellChecker, cx: &LateContext, sp: Span, test_text: String) {
        if let Ok(errors) = checker.check(&test_text) {
            for e in errors {
                if !e.suggestions.is_empty() {
                    cx.span_lint(SPELLING_ERROR,
                                 sp,
                                 &format!("'{}' is misspelled. Maybe you meant '{}'",
                                     &e.misspelled,
                                     &e.suggestions[0]
                                 ));
                }
            }
            // println!("{:?}", text.as_str());
        } else {
            // DEBUG
            // println!("FAILED RUNNING CHECK ON LINE:");
            // println!("{}", test_text);
        }
    }

    fn check_spelling_errors(&self,
                                cx: &LateContext,
                                id: Option<ast::NodeId>,
                                attrs: &[ast::Attribute],
                                sp: Span) {
        // If we're building a test harness, then warning about
        // documentation is probably not really relevant right now.
        if cx.sess().opts.test {
            return;
        }

        // // `#[doc(hidden)]` disables missing_docs check.
        // if self.doc_hidden() {
        //     return;
        // }

        // Only check publicly-visible items, using the result from the privacy pass.
        // It's an option so the crate root can also use this function (it doesn't
        // have a NodeId).
        if let Some(id) = id {
            if !cx.access_levels.is_exported(id) {
                return;
            }
        }

        // println!("=====START BLOCK====="); // DEBUG

        let mut checker = SpellLauncher::new()
            .aspell()
            .dictionary("en")
            .timeout(1000)
            .launch()
            .unwrap();

        let text_block = to_text_block(attrs);

        let blocks = markdown::tokenize(&text_block);
        for block in &blocks {
            Self::travserse_markdown_block(&mut checker, cx, sp, block);
        }

        // println!("=====END BLOCK====="); // DEBUG
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for SpellingError {
    fn enter_lint_attrs(&mut self, _: &LateContext, attrs: &[ast::Attribute]) {
        let doc_hidden = self.doc_hidden() ||
                         attrs.iter().any(|attr| {
            attr.check_name("doc") &&
            match attr.meta_item_list() {
                None => false,
                Some(l) => attr::list_contains_name(&l[..], "hidden"),
            }
        });
        self.doc_hidden_stack.push(doc_hidden);
    }

    fn exit_lint_attrs(&mut self, _: &LateContext, _attrs: &[ast::Attribute]) {
        self.doc_hidden_stack.pop().expect("empty doc_hidden_stack");
    }

    // fn check_struct_def(&mut self,
    //                     _: &LateContext,
    //                     _: &hir::VariantData,
    //                     _: ast::Name,
    //                     _: &hir::Generics,
    //                     item_id: ast::NodeId) {
    //     self.struct_def_stack.push(item_id);
    // }
    //
    // fn check_struct_def_post(&mut self,
    //                          _: &LateContext,
    //                          _: &hir::VariantData,
    //                          _: ast::Name,
    //                          _: &hir::Generics,
    //                          item_id: ast::NodeId) {
    //     let popped = self.struct_def_stack.pop().expect("empty struct_def_stack");
    //     assert!(popped == item_id);
    // }

    fn check_crate(&mut self, cx: &LateContext, krate: &hir::Crate) {
        self.check_spelling_errors(cx, None, &krate.attrs, krate.span);
    }

    fn check_item(&mut self, cx: &LateContext, it: &hir::Item) {
        self.check_spelling_errors(cx, Some(it.id), &it.attrs, it.span);
    }
}
