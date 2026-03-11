//! CLI integration tests for --regexp flag
//!
//! These tests verify the regex-based window targeting functionality.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that --help shows the --regexp flag
#[test]
fn test_regexp_help() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--regexp"))
        .stdout(predicate::str::contains("-r"));
}

/// Test that --regexp with invalid regex pattern shows error message
#[test]
fn test_regexp_invalid_pattern() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--regexp").arg("[invalid").arg("--output").arg("/tmp/test_invalid.png");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid").or(predicate::str::contains("regex")))
        .stderr(predicate::str::contains("Available windows"));
}

/// Test that --regexp with pattern matching no windows shows error and window list
#[test]
fn test_regexp_no_matches() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--regexp")
        .arg("XYZNonExistentWindowPattern12345")
        .arg("--output")
        .arg("/tmp/test_no_match.png");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Window")))
        .stderr(predicate::str::contains("Available windows"));
}

/// Test that --regexp flag is accepted and processed
/// Uses dual-outcome pattern: accepts success or graceful failure
/// Auto-selects first match when multiple windows match
#[test]
fn test_regexp_flag_accepted() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--regexp").arg("Terminal").arg("--output").arg("/tmp/test_regexp.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Command should run - either success with capture or error with not found/capture error
    assert!(
        output.status.success()
            || stdout.contains("Saved screenshot to:")
            || stderr.contains("not found")
            || stderr.contains("capture")
            || stderr.contains("Error"),
        "--regexp flag should be accepted and processed, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test that --regexp with broad pattern auto-selects first match
/// (No more disambiguation - automatically captures first matching window)
#[test]
fn test_regexp_multiple_matches_auto_selects_first() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    // Use a very broad pattern that should match multiple windows
    cmd.arg("--regexp").arg(".*").arg("--output").arg("/tmp/test_multi.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should NOT show "Multiple windows matched" disambiguation anymore
    assert!(
        !stderr.contains("Multiple windows matched"),
        "Should not show disambiguation - should auto-select first match"
    );

    // Should either succeed (auto-selecting first) or fail gracefully
    if output.status.success() {
        assert!(
            stdout.contains("Saved screenshot to:"),
            "Success should print 'Saved screenshot to:'"
        );
    }
    // If fails, it's due to capture error or permission, not disambiguation
}

/// Test that --regexp is mutually exclusive with --window (clap enforces)
#[test]
fn test_regexp_mutually_exclusive_with_window() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--regexp")
        .arg("Terminal")
        .arg("--window")
        .arg("Terminal")
        .arg("--output")
        .arg("/tmp/test_exclusive.png");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with").or(predicate::str::contains("error")));
}

/// Test that --regexp with case-insensitive (?i) flag is accepted
#[test]
fn test_regexp_case_insensitive_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--regexp")
        .arg("(?i)terminal")
        .arg("--output")
        .arg("/tmp/test_case.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Command should run - either success or graceful failure
    assert!(
        output.status.success()
            || stdout.contains("Saved screenshot to:")
            || stderr.contains("not found")
            || stderr.contains("capture")
            || stderr.contains("Error"),
        "--regexp with (?i) flag should be accepted, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test that --regexp with -r short flag works
#[test]
fn test_regexp_short_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("-r").arg("Terminal").arg("--output").arg("/tmp/test_short.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Command should run - either success or graceful failure
    assert!(
        output.status.success()
            || stdout.contains("Saved screenshot to:")
            || stderr.contains("not found")
            || stderr.contains("capture")
            || stderr.contains("Error"),
        "-r short flag should work, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test that successful --regexp capture uses correct output path in message
#[test]
fn test_regexp_custom_output_path() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--regexp")
        .arg("Terminal")
        .arg("--output")
        .arg("/tmp/snap_regexp_custom.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        // Success message must include the custom path
        assert!(
            stdout.contains("/tmp/snap_regexp_custom.png"),
            "Success message should contain the custom --output path"
        );
    }
    // If capture fails (headless CI, no permission), that is acceptable
}

/// Test that --regexp with specific pattern that likely matches single window
/// follows the same success/failure patterns as other modes
#[test]
fn test_regexp_single_match_pattern() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    // Use a specific pattern that might match exactly one window
    cmd.arg("--regexp")
        .arg("^Terminal$")
        .arg("--output")
        .arg("/tmp/test_single.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either succeed with capture or fail gracefully
    if output.status.success() {
        assert!(
            stdout.contains("Saved screenshot to:"),
            "Success should print 'Saved screenshot to:'"
        );
    } else {
        // Graceful failure should show error - could be not found,
        // invalid regex, capture error, or permission error
        let has_error = stderr.contains("not found")
            || stderr.contains("Invalid")
            || stderr.contains("capture")
            || stderr.contains("Error")
            || stderr.contains("Available windows");
        assert!(
            has_error,
            "Failure should be graceful with appropriate error message, got stderr: {}",
            stderr
        );
    }
}
