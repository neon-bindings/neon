extern crate compiletest_rs as compiletest;
extern crate neon;

use std::path::PathBuf;
use std::env::var;
use neon::meta::BUILD_PROFILE;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    let cfg_mode = mode.parse().expect("Invalid mode");

    config.target_rustcflags = Some(format!("-L target/{}/ -L target/{}/deps/", BUILD_PROFILE, BUILD_PROFILE));
    if let Ok(name) = var("TESTNAME") {
        let s : String = name.to_owned();
        config.filter = Some(s)
    }
    config.mode = cfg_mode;
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    compiletest::run_tests(&config);
}

#[test]
fn run_all() {
    run_mode("compile-fail");
}
