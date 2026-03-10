//! CLI integration tests
//!
//! These tests verify the CLI argument parsing and behavior using assert_cmd.
//! They invoke the actual binary to ensure end-to-end functionality.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that --help shows all expected flags
#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--window"))
        .stdout(predicate::str::contains("--pid"))
        .stdout(predicate::str::contains("--index"))
        .stdout(predicate::str::contains("--list"))
        .stdout(predicate::str::contains("--highlight"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--version"));
}

/// Test that --version returns version string
#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("snap-window"))
        .stdout(predicate::str::contains("0.1.0"));
}

/// Test that --list runs successfully and shows window entries
#[test]
fn test_list() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("PID:"))
        .stdout(predicate::str::contains("App:"));
}

/// Test that running without args fails with error about required mode
#[test]
fn test_missing_args() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    // No arguments provided
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

/// Test that --window flag is accepted
#[test]
fn test_window_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--window").arg("Terminal");
    // Should either succeed (if window found) or fail gracefully with window not found
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Command should run - either success with found window or error with not found
    assert!(
        output.status.success()
            || stdout.contains("Found window")
            || stderr.contains("not found")
            || stdout.contains("Available windows"),
        "--window flag should be accepted and processed"
    );
}

/// Test that --pid flag is accepted
#[test]
fn test_pid_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--pid").arg("12345");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stdout.contains("Found window")
            || stderr.contains("not found")
            || stdout.contains("Available windows"),
        "--pid flag should be accepted and processed"
    );
}

/// Test that --index flag is accepted
#[test]
fn test_index_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--index").arg("0");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stdout.contains("Selected window")
            || stderr.contains("Invalid")
            || stdout.contains("Available windows"),
        "--index flag should be accepted and processed"
    );
}

/// Test that --output flag is accepted
#[test]
fn test_output_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--list").arg("--output").arg("/tmp/test_output.png");
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("/tmp/test_output.png"));
}

/// Test that --highlight flag is accepted
#[test]
fn test_highlight_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--highlight").arg("0");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stdout.contains("highlight")
            || stderr.contains("Invalid")
            || stdout.contains("Available windows"),
        "--highlight flag should be accepted and processed"
    );
}

/// Test that invalid index produces error and auto-lists available windows on stderr
#[test]
fn test_invalid_index() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--index").arg("999");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid").or(predicate::str::contains("index")))
        .stderr(predicate::str::contains("Available windows"));
}

/// Test that invalid pid produces not found error and auto-lists available windows on stderr
#[test]
fn test_invalid_pid() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--pid").arg("99999");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("PID")))
        .stderr(predicate::str::contains("Available windows"));
}

/// Test that invalid window name produces not found error and auto-lists available windows on stderr
#[test]
fn test_invalid_window() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--window").arg("NonExistentWindowXYZ");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"))
        .stderr(predicate::str::contains("Available windows"));
}

/// Test that case-insensitive window search works (WIN-02 smoke test)
/// A lowercase search for a non-existent window should fail gracefully
#[test]
fn test_window_flag_case_insensitive_not_found() {
    // Even lowercase search should produce "not found" (not a crash/panic)
    // and show available windows on stderr
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--window").arg("nonexistentwindowxyz_testonly");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"))
        .stderr(predicate::str::contains("Available windows"));
}
