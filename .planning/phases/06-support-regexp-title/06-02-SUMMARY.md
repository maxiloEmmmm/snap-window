---
phase: 06-support-regexp-title
plan: 02
subsystem: window-targeting
tags: [regex, cli, integration-testing]
dependency_graph:
  requires: [06-01]
  provides: []
  affects: [src/main.rs, tests/cli_regex_tests.rs]
tech_stack:
  added: []
  patterns: [match arm pattern matching, dual-outcome tests, disambiguation UI]
key_files:
  created:
    - tests/cli_regex_tests.rs
  modified:
    - src/main.rs
decisions:
  - "Handle empty matches separately from error cases for consistent UX"
  - "Use dual-outcome test pattern for environment-dependent tests"
  - "Disambiguation list shows index, title, pid, and app_name for each match"
metrics:
  duration: 7
  completed_date: "2026-03-11"
  tests_added: 10
  lines_added: ~250
---

# Phase 06 Plan 02: Wire --regexp into CLI Summary

**One-liner:** Wired the --regexp flag into main.rs with single match capture, multiple match disambiguation, and comprehensive integration tests.

---

## What Was Built

### 1. --regexp Match Arm (src/main.rs)

Added new match arm in the main.rs cli.mode match expression:

```rust
Mode { regexp: Some(pattern), .. } => {
    let windows = platform::list_windows()
        .context("Failed to enumerate windows")?;

    match window_service::find_by_regexp(&windows, &pattern) {
        Ok(matches) if matches.len() == 1 => {
            // Single match - capture screenshot
            capture_service::capture_window(matches[0], &output_path)?;
            println!("Saved screenshot to: {}", output_path.display());
        }
        Ok(matches) if matches.len() > 1 => {
            // Multiple matches - show disambiguation
            eprintln!("Multiple windows matched pattern '{}'.", pattern);
            for w in &matches {
                eprintln!("  [{}] {} (PID: {}, {})",
                    w.index, w.title, w.pid, w.app_name);
            }
            eprintln!("\nUse --index to target a specific window.");
            return Err(error::AppError::window_not_found(pattern).into());
        }
        Ok(_) => {
            // Empty matches - show window list
            window_service::print_available_windows(&windows);
            return Err(error::AppError::window_not_found(pattern).into());
        }
        Err(e) => {
            // Invalid regex - show window list
            window_service::print_available_windows(&windows);
            return Err(e.into());
        }
    }
}
```

**Four distinct outcomes:**
1. **Single match**: Captures screenshot, prints success message
2. **Multiple matches**: Shows disambiguation list with indices, suggests --index
3. **No matches**: Shows available windows, returns WindowNotFound error
4. **Invalid regex**: Shows available windows, propagates InvalidRegexPattern error

### 2. Integration Tests (tests/cli_regex_tests.rs)

10 comprehensive CLI integration tests:

| Test | Purpose |
|------|---------|
| `test_regexp_help` | Verify --regexp and -r appear in help output |
| `test_regexp_invalid_pattern` | Invalid regex like "[invalid" shows error |
| `test_regexp_no_matches` | Pattern matching nothing shows window list |
| `test_regexp_flag_accepted` | Flag is processed (dual-outcome) |
| `test_regexp_multiple_matches` | Broad pattern shows disambiguation list |
| `test_regexp_mutually_exclusive_with_window` | Clap enforces --regexp vs --window exclusivity |
| `test_regexp_case_insensitive_flag` | (?i) flag for case-insensitive matching |
| `test_regexp_short_flag` | -r short flag works |
| `test_regexp_custom_output_path` | Custom --output path in success message |
| `test_regexp_single_match_pattern` | Single match capture works |

**Test patterns used:**
- Dual-outcome tests for environment-dependent scenarios
- Predicate-based assertions for output content
- Proper cleanup and temp file handling

---

## Test Results

```
running 71 tests (library + integration)
test result: ok. 71 passed; 0 failed; 0 ignored

New tests added:
- cli_regex_tests: 10 integration tests
```

---

## Deviations from Plan

### Auto-fixed Issue: Empty match handling

**Found during:** Task 1

**Issue:** Initial implementation didn't handle the `Ok([])` case explicitly, causing a non-exhaustive pattern compilation error.

**Fix:** Added explicit `Ok(_)` arm to handle empty matches:
```rust
Ok(_) => {
    window_service::print_available_windows(&windows);
    return Err(error::AppError::window_not_found(pattern).into());
}
```

**Commit:** Included in Task 1 commit

---

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Separate empty match arm | Consistent UX with other modes - always show window list on "not found" |
| Disambiguation to stderr | User-facing info (not program output) belongs on stderr |
| Include all window fields | Index, title, PID, and app_name help users identify correct window |
| Suggest --index | Clear next step for disambiguation |
| Dual-outcome tests | Tests work in both headless CI and desktop environments |

---

## Commits

| Hash | Message |
|------|---------|
| 25dd8e3 | feat(06-02): wire --regexp handling in main.rs |
| c572720 | test(06-02): add integration tests for --regexp flag |

---

## Self-Check: PASSED

- [x] --regexp match arm added to main.rs
- [x] Single match captures and prints success
- [x] Multiple matches show disambiguation with indices
- [x] No matches show window list
- [x] Invalid regex shows error with window list
- [x] All 10 integration tests pass
- [x] Full test suite passes (71 tests)
- [x] Release build compiles successfully

---

## Verification Steps

The --regexp feature is ready for human verification:

1. **List windows**: `./target/release/snap-window --list`
2. **Test single match**: `./target/release/snap-window --regexp "Terminal" --output /tmp/test.png`
3. **Test multiple matches**: `./target/release/snap-window --regexp ".*" --output /tmp/test.png`
4. **Test invalid regex**: `./target/release/snap-window --regexp "[invalid" --output /tmp/test.png`
5. **Test case-insensitive**: `./target/release/snap-window --regexp "(?i)terminal" --output /tmp/test.png`

---

## Phase 06 Complete

Both plans in Phase 06 are now complete:
- **06-01**: Core regex support (regex crate, error variant, find_by_regexp function, --regexp flag)
- **06-02**: CLI integration (main.rs wiring, disambiguation, integration tests)

The --regexp feature is fully functional for end users.
