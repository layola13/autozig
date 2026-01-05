//! Rust tests that invoke Zig test executables
//!
//! This demonstrates integration between Rust's test framework and Zig's test
//! blocks.

use std::{
    path::PathBuf,
    process::Command,
};

/// Get the path to a compiled Zig test executable
fn get_test_exe_path(name: &str) -> PathBuf {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    PathBuf::from(out_dir).join(format!("test_{}", name))
}

#[test]
fn test_math_zig_tests() {
    let test_exe = get_test_exe_path("math");

    println!("Running Zig tests from: {}", test_exe.display());

    let output = Command::new(&test_exe)
        .output()
        .expect("Failed to execute Zig math tests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Zig math tests output:");
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(
        output.status.success(),
        "Zig math tests failed with status: {:?}\nStdout: {}\nStderr: {}",
        output.status.code(),
        stdout,
        stderr
    );
}

#[test]
fn test_strings_zig_tests() {
    let test_exe = get_test_exe_path("strings");

    println!("Running Zig tests from: {}", test_exe.display());

    let output = Command::new(&test_exe)
        .output()
        .expect("Failed to execute Zig strings tests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Zig strings tests output:");
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(
        output.status.success(),
        "Zig strings tests failed with status: {:?}\nStdout: {}\nStderr: {}",
        output.status.code(),
        stdout,
        stderr
    );
}

#[test]
fn test_zig_zig_tests() {
    let test_exe = get_test_exe_path("zig");

    println!("Running Zig tests from: {}", test_exe.display());

    let output = Command::new(&test_exe)
        .output()
        .expect("Failed to execute Zig zig.zig tests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Zig zig.zig tests output:");
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(
        output.status.success(),
        "Zig zig.zig tests failed with status: {:?}\nStdout: {}\nStderr: {}",
        output.status.code(),
        stdout,
        stderr
    );
}

#[test]
fn test_all_zig_tests_exist() {
    // Verify that all expected test executables were compiled
    let test_names = ["math", "strings", "zig"];

    for name in &test_names {
        let test_exe = get_test_exe_path(name);
        assert!(test_exe.exists(), "Test executable not found: {}", test_exe.display());
        println!("✓ Found test executable: {}", test_exe.display());
    }
}
