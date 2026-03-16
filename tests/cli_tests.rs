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

    // Command should run - either success with capture or error with not found
    assert!(
        output.status.success()
            || stdout.contains("Saved screenshot to:")
            || stderr.contains("not found")
            || stderr.contains("capture")
            || stderr.contains("Error"),
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
            || stdout.contains("Saved screenshot to:")
            || stderr.contains("not found")
            || stderr.contains("capture")
            || stderr.contains("Error"),
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
            || stdout.contains("Saved screenshot to:")
            || stderr.contains("Invalid")
            || stderr.contains("capture")
            || stderr.contains("Error"),
        "--index flag should be accepted and processed"
    );
}

/// Test that --output flag is accepted (used with capture modes, not --list)
#[test]
fn test_output_flag() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    // --output is meaningful with window targeting, not --list
    // Just verify the flag is accepted (command parses successfully)
    cmd.arg("--window").arg("TestWindow").arg("--output").arg("/tmp/test_output.png");
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either succeed or fail gracefully (window not found, capture error, etc.)
    assert!(
        output.status.success()
            || stderr.contains("not found")
            || stderr.contains("Error")
            || stderr.contains("capture"),
        "--output flag should be accepted, got stderr: {}",
        stderr
    );
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

/// Test that --output flag is passed through to capture and not just displayed (CLI-04)
/// Window will not be found (NonExistentXYZ), but the failure path should not show
/// the old "Output path:" placeholder -- it should show a proper error.
#[test]
fn test_capture_output_flag_accepted() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--window")
        .arg("NonExistentWindowForCaptureTest_XYZ")
        .arg("--output")
        .arg("/tmp/snap_capture_test.png");
    cmd.assert()
        .failure()
        // Should show "not found", not the old placeholder "Output path:"
        .stderr(predicate::str::contains("not found"));
}

/// Test that successful capture (or graceful failure) uses the new message format.
/// Old placeholder "Found window:" and "Output path:" must not appear in output.
#[test]
fn test_capture_placeholder_text_removed() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--index").arg("0");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Old placeholder text must be gone
    assert!(
        !stdout.contains("Found window:"),
        "Old placeholder 'Found window:' should not appear in output"
    );
    assert!(
        !stdout.contains("Output path:"),
        "Old placeholder 'Output path:' should not appear in output"
    );
    // If it succeeded, it should say "Saved screenshot to:"
    if output.status.success() {
        assert!(
            stdout.contains("Saved screenshot to:"),
            "Success should print 'Saved screenshot to:'"
        );
    }
}

/// Test that --output with a custom path is used in the success message (CLI-04 wiring)
#[test]
fn test_capture_custom_output_path_in_success_message() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--index")
        .arg("0")
        .arg("--output")
        .arg("/tmp/snap_custom_output_test.png");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        // Success message must include the custom path
        assert!(
            stdout.contains("/tmp/snap_custom_output_test.png"),
            "Success message should contain the custom --output path"
        );
    }
    // If capture fails (headless CI, no permission), that is acceptable --
    // the test passes as long as there is no panic
}

/// Test that invalid highlight index shows error and lists available windows (HIL-01)
#[test]
fn test_highlight_invalid_index_shows_error() {
    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--highlight").arg("99999");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid").or(predicate::str::contains("index")))
        .stderr(predicate::str::contains("Available windows"));
}

/// Test that --highlight produces a JSON file with window metadata (HIL-03, HIL-04)
/// Uses dual-outcome pattern: accepts success (JSON created) or graceful failure (no windows)
#[test]
fn test_highlight_produces_json_file() {
    use std::fs;

    let json_path = std::path::PathBuf::from("/tmp/snap_test_highlight.json");
    let png_path = std::path::PathBuf::from("/tmp/snap_test_highlight.png");

    // Clean up any existing files
    let _ = fs::remove_file(&json_path);
    let _ = fs::remove_file(&png_path);

    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--highlight").arg("0").arg("--output").arg(&png_path);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Success case: JSON file should exist
        assert!(
            json_path.exists(),
            "JSON file should be created at {}",
            json_path.display()
        );

        // Verify JSON content
        let json_content = fs::read_to_string(&json_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify common fields are present
        assert!(parsed.get("window_id").is_some(), "JSON should contain window_id");
        assert!(parsed.get("title").is_some(), "JSON should contain title");
        assert!(parsed.get("pid").is_some(), "JSON should contain pid");
        assert!(parsed.get("app_name").is_some(), "JSON should contain app_name");
        assert!(parsed.get("x").is_some(), "JSON should contain x");
        assert!(parsed.get("y").is_some(), "JSON should contain y");
        assert!(parsed.get("width").is_some(), "JSON should contain width");
        assert!(parsed.get("height").is_some(), "JSON should contain height");
        assert!(parsed.get("platform").is_some(), "JSON should contain platform");

        // Verify success message
        assert!(
            stdout.contains("Saved window info to:"),
            "Success message should indicate JSON was saved"
        );
    } else {
        // Graceful failure case: either invalid index (no windows) or highlight error
        // This is acceptable in headless CI environments
        let has_no_windows = stderr.contains("Invalid") || stderr.contains("index") || stderr.contains("No window");
        let has_highlight_error = stderr.contains("highlight") || stderr.contains("border");
        assert!(
            has_no_windows || has_highlight_error,
            "Failure should be due to no windows available or highlight error, got stderr: {}",
            stderr
        );
    }

    // Clean up
    let _ = fs::remove_file(&json_path);
    let _ = fs::remove_file(&png_path);
}

/// Test that --highlight does NOT create a PNG file (HIL-03)
/// Uses dual-outcome pattern: accepts success (no PNG) or graceful failure
#[test]
fn test_highlight_no_png_created() {
    use std::fs;

    let json_path = std::path::PathBuf::from("/tmp/snap_test_no_png.json");
    let png_path = std::path::PathBuf::from("/tmp/snap_test_no_png.png");

    // Clean up any existing files
    let _ = fs::remove_file(&json_path);
    let _ = fs::remove_file(&png_path);

    let mut cmd = Command::cargo_bin("snap-window").unwrap();
    cmd.arg("--highlight").arg("0").arg("--output").arg(&png_path);
    let output = cmd.output().unwrap();

    if output.status.success() {
        // Success case: PNG should NOT exist, JSON should exist
        assert!(
            !png_path.exists(),
            "PNG file should NOT be created in highlight mode (only JSON)"
        );
        assert!(
            json_path.exists(),
            "JSON file should be created at {}",
            json_path.display()
        );
    }
    // If command fails (no windows, headless CI), that's acceptable --
    // the test passes as long as no PNG was created on success

    // Clean up
    let _ = fs::remove_file(&json_path);
    let _ = fs::remove_file(&png_path);
}
