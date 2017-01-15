#![feature(plugin_registrar)]
#![feature(box_syntax, rustc_private)]

extern crate syntax;
extern crate syntax_pos;

// Load rustc as a plugin to get macros
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;

extern crate ispell;
extern crate markdown;

use rustc::lint::{LateContext, LintContext, LintPass, LateLintPass,
                  LateLintPassObject, LintArray};
use rustc::hir;
// use rustc::hir::attr;
use rustc::hir::map as hir_map;
use rustc_plugin::Registry;
use syntax::ast;
use syntax::ast::MetaItemKind;
use syntax::ast::LitKind;
use syntax::attr;
use syntax_pos::Span;

use std::collections::HashSet;

use ispell::{SpellLauncher, SpellChecker};
use markdown::{Block, ListItem, Span as MdSpan};

declare_lint!(SPELLING_ERROR, Warn, "Warn about spelling errors.");

pub struct SpellingError {
    /// Stack of IDs of struct definitions.
    struct_def_stack: Vec<ast::NodeId>,

    /// True if inside variant definition
    in_variant: bool,

    /// Stack of whether #[doc(hidden)] is set
    /// at each level which has lint attributes.
    doc_hidden_stack: Vec<bool>,

    /// Private traits or trait items that leaked through. Don't check their methods.
    private_traits: HashSet<ast::NodeId>,
}

impl SpellingError {
    pub fn new() -> SpellingError {
        SpellingError {
            struct_def_stack: vec![],
            in_variant: false,
            doc_hidden_stack: vec![false],
            private_traits: HashSet::new(),
        }
    }

    fn doc_hidden(&self) -> bool {
        *self.doc_hidden_stack.last().expect("empty doc_hidden_stack")
    }

    fn prepare_line(input: String) -> String {
        let mut result = input;

        // TODO: only replace if at line beginning
        result = result.replace("//! ", "");
        result = result.replace("//!", "");
        result = result.replace("/// ", "");
        result = result.replace("///", "");
        result = result.replace("// ", "");
        result = result.replace("//", "");

        result
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
                // println!("'{}' (pos: {}) is misspelled!", &e.misspelled, e.position);
                if !e.suggestions.is_empty() {
                    // println!("Maybe you meant '{}'?", &e.suggestions[0]);
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
            println!("FAILED RUNNING CHECK ON LINE:");
            println!("{}", test_text);
        }
    }

    fn check_spelling_errors(&self,
                                cx: &LateContext,
                                id: Option<ast::NodeId>,
                                attrs: &[ast::Attribute],
                                sp: Span,
                                desc: &'static str) {
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

        let mut text_block = "".to_owned();
        println!("=====START BLOCK=====");

        let mut checker = SpellLauncher::new()
            .aspell()
            .dictionary("en")
            .timeout(1000)
            .launch()
            .unwrap();

        let maybe_lines: Vec<Option<String>> = attrs.iter().map(|attr|
            match attr.value.node {
                MetaItemKind::NameValue(ref spanned) => {
                    match spanned.node {
                        LitKind::Str(text, _) => {
                            let mut test_text = text.as_str().to_string();
                            test_text = Self::prepare_line(test_text);

                            return Some(test_text);
                        },
                        _ => None,
                    }
                },
                _ => None,
            }
        ).collect();

        let lines: Vec<_> = maybe_lines.into_iter().filter_map(|l| l).collect();
        let text_block = lines.join("\n");

        println!("{}", text_block);
        let blocks = markdown::tokenize(&text_block);
        for block in &blocks {
            Self::travserse_markdown_block(&mut checker, cx, sp, block);
            // println!("{:?}", block);
        }

        println!("=====END BLOCK=====");

        // let has_doc = attrs.iter().any(|a| a.is_value_str() && a.name() == "doc");
        // if !has_doc {
        //     cx.span_lint(SPELLING_ERROR,
        //                  sp,
        //                  &format!("missing documentation for {}", desc));
        // }
    }
}

impl LintPass for SpellingError {
    fn get_lints(&self) -> LintArray {
        lint_array!(SPELLING_ERROR)
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

    fn check_struct_def(&mut self,
                        _: &LateContext,
                        _: &hir::VariantData,
                        _: ast::Name,
                        _: &hir::Generics,
                        item_id: ast::NodeId) {
        self.struct_def_stack.push(item_id);
    }

    fn check_struct_def_post(&mut self,
                             _: &LateContext,
                             _: &hir::VariantData,
                             _: ast::Name,
                             _: &hir::Generics,
                             item_id: ast::NodeId) {
        let popped = self.struct_def_stack.pop().expect("empty struct_def_stack");
        assert!(popped == item_id);
    }

    fn check_crate(&mut self, cx: &LateContext, krate: &hir::Crate) {
        self.check_spelling_errors(cx, None, &krate.attrs, krate.span, "crate");
    }

    fn check_item(&mut self, cx: &LateContext, it: &hir::Item) {
        let desc = match it.node {
            hir::ItemFn(..) => "a function",
            hir::ItemMod(..) => "a module",
            hir::ItemEnum(..) => "an enum",
            hir::ItemStruct(..) => "a struct",
            hir::ItemUnion(..) => "a union",
            hir::ItemTrait(.., ref trait_item_refs) => {
                // Issue #11592, traits are always considered exported, even when private.
                // if it.vis == hir::Visibility::Inherited {
                //     self.private_traits.insert(it.id);
                //     for trait_item_ref in trait_item_refs {
                //         self.private_traits.insert(trait_item_ref.id.node_id);
                //     }
                //     return;
                // }
                "a trait"
            }
            hir::ItemTy(..) => "a type alias",
            hir::ItemImpl(.., Some(ref trait_ref), _, ref impl_item_refs) => {
                // If the trait is private, add the impl items to private_traits so they don't get
                // reported for missing docs.
                let real_trait = trait_ref.path.def.def_id();
                if let Some(node_id) = cx.tcx.map.as_local_node_id(real_trait) {
                    match cx.tcx.map.find(node_id) {
                        Some(hir_map::NodeItem(item)) => {
                            if item.vis == hir::Visibility::Inherited {
                                for impl_item_ref in impl_item_refs {
                                    self.private_traits.insert(impl_item_ref.id.node_id);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                return;
            }
            hir::ItemConst(..) => "a constant",
            hir::ItemStatic(..) => "a static",
            _ => return,
        };
        // println!("{:?}", desc);
        // println!("{:?}", it.attrs);

        self.check_spelling_errors(cx, Some(it.id), &it.attrs, it.span, desc);
    }

    // fn check_trait_item(&mut self, cx: &LateContext, trait_item: &hir::TraitItem) {
    //     if self.private_traits.contains(&trait_item.id) {
    //         return;
    //     }
    //
    //     let desc = match trait_item.node {
    //         hir::TraitItemKind::Const(..) => "an associated constant",
    //         hir::TraitItemKind::Method(..) => "a trait method",
    //         hir::TraitItemKind::Type(..) => "an associated type",
    //     };
    //
    //     self.check_spelling_errors(cx,
    //                                   Some(trait_item.id),
    //                                   &trait_item.attrs,
    //                                   trait_item.span,
    //                                   desc);
    // }
    //
    // fn check_impl_item(&mut self, cx: &LateContext, impl_item: &hir::ImplItem) {
    //     // If the method is an impl for a trait, don't doc.
    //     if method_context(cx, impl_item.id, impl_item.span) == MethodLateContext::TraitImpl {
    //         return;
    //     }
    //
    //     let desc = match impl_item.node {
    //         hir::ImplItemKind::Const(..) => "an associated constant",
    //         hir::ImplItemKind::Method(..) => "a method",
    //         hir::ImplItemKind::Type(_) => "an associated type",
    //     };
    //     self.check_spelling_errors(cx,
    //                                   Some(impl_item.id),
    //                                   &impl_item.attrs,
    //                                   impl_item.span,
    //                                   desc);
    // }
    //
    // fn check_struct_field(&mut self, cx: &LateContext, sf: &hir::StructField) {
    //     if !sf.is_positional() {
    //         if sf.vis == hir::Public || self.in_variant {
    //             let cur_struct_def = *self.struct_def_stack
    //                 .last()
    //                 .expect("empty struct_def_stack");
    //             self.check_spelling_errors(cx,
    //                                           Some(cur_struct_def),
    //                                           &sf.attrs,
    //                                           sf.span,
    //                                           "a struct field")
    //         }
    //     }
    // }
    //
    // fn check_variant(&mut self, cx: &LateContext, v: &hir::Variant, _: &hir::Generics) {
    //     self.check_spelling_errors(cx,
    //                                   Some(v.node.data.id()),
    //                                   &v.node.attrs,
    //                                   v.span,
    //                                   "a variant");
    //     assert!(!self.in_variant);
    //     self.in_variant = true;
    // }
    //
    // fn check_variant_post(&mut self, _: &LateContext, _: &hir::Variant, _: &hir::Generics) {
    //     assert!(self.in_variant);
    //     self.in_variant = false;
    // }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box SpellingError::new() as LateLintPassObject);
}
