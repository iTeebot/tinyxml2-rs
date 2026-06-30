#![allow(clippy::unnecessary_debug_formatting)]
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_cpp_helper_binary() -> PathBuf {
    let mut exe_path = std::env::current_exe().unwrap();
    // Target binary directory is target/debug/ or target/release/
    exe_path.pop(); // remove filename
    if exe_path.file_name().unwrap() == "deps" {
        exe_path.pop(); // remove deps
    }
    let binary_name = if cfg!(windows) {
        "tinyxml2-cpp-helper.exe"
    } else {
        "tinyxml2-cpp-helper"
    };
    exe_path.join(binary_name)
}

pub fn run_cpp_reference(xml_path: &Path, whitespace_mode: &str) -> String {
    let helper_path = find_cpp_helper_binary();
    assert!(
        helper_path.exists(),
        "C++ helper binary not found at {helper_path:?}"
    );

    let output = Command::new(&helper_path)
        .arg(xml_path)
        .arg(whitespace_mode)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run C++ helper at {helper_path:?}: {e}"));

    assert!(
        output.status.success(),
        "C++ helper failed with exit code {:?}. Stderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("C++ helper output is not valid UTF-8")
}
