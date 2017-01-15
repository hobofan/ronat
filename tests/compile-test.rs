extern crate compiletest_rs as compiletest;

use std::path::PathBuf;
use std::env::{set_var, var};

fn run_mode(dir: &'static str, mode: &'static str) {
    let mut config = compiletest::default_config();

    let cfg_mode = mode.parse().expect("Invalid mode");
    config.target_rustcflags = Some("-L target/debug/ -L target/debug/deps".to_owned());
    if let Ok(name) = var::<&str>("TESTNAME") {
        let s: String = name.to_owned();
        config.filter = Some(s)
    }

    config.mode = cfg_mode;
    config.verbose = true;
    // config.quiet = false;
    config.src_base = PathBuf::from(format!("tests/{}", dir));

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("run-pass", "run-pass");
    // run_mode("compile-fail", "compile-fail");
}
