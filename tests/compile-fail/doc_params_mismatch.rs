#![feature(plugin)]
#![plugin(ronat)]

pub fn main() {
    somefunction(5);
}

/// This is an header.
///
/// # Parameters
///
/// - `foo`: Incorrect variable name
#[deny(doc_params_mismatch)]
#[allow(unused_variables)]
pub fn somefunction(bar: isize) {
}
