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
    let helper_path = exe_path.join(binary_name);

    if !helper_path.exists() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let workspace_root = Path::new(&manifest_dir)
            .ancestors()
            .nth(2)
            .unwrap_or_else(|| Path::new("."));

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--bin")
            .arg("tinyxml2-cpp-helper")
            .current_dir(workspace_root);

        // If running in release mode, build release helper
        if !exe_path.to_string_lossy().contains("debug") {
            cmd.arg("--release");
        }

        let status = cmd.status();
        if let Ok(s) = status {
            if !s.success() {
                eprintln!("Warning: Failed to compile tinyxml2-cpp-helper via cargo build.");
            }
        } else if let Err(e) = status {
            eprintln!("Warning: Could not invoke cargo build: {e}");
        }
    }

    helper_path
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
