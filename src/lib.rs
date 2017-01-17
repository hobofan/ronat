#![feature(plugin_registrar)]
#![feature(box_syntax, rustc_private)]

extern crate syntax;
extern crate syntax_pos;

// Load rustc as a plugin to get macros
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;

extern crate docstrings;
extern crate ispell;
extern crate markdown;

mod helpers;

mod doc_params_mismatch;
mod spelling_error;

use doc_params_mismatch::DocParamsMismatch;
use spelling_error::SpellingError;

use rustc::lint::LateLintPassObject;
use rustc_plugin::Registry;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box DocParamsMismatch as LateLintPassObject);
    reg.register_late_lint_pass(box SpellingError::new() as LateLintPassObject);
}
