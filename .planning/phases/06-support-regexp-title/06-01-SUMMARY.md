---
phase: 06-support-regexp-title
plan: 01
subsystem: window-targeting
tags: [regex, cli, error-handling, testing]
dependency_graph:
  requires: []
  provides: [06-02, 06-03]
  affects: [src/cli.rs, src/window_service.rs, src/error.rs, Cargo.toml]
tech_stack:
  added: [regex crate 1.12]
  patterns: [TDD, pure functions, error constructors]
key_files:
  created: []
  modified:
    - Cargo.toml
    - src/error.rs
    - src/window_service.rs
    - src/cli.rs
decisions:
  - "Use regex crate 1.12 for pattern matching"
  - "Return Vec from find_by_regexp for disambiguation (multiple matches possible)"
  - "InvalidRegexPattern error includes both pattern and details for debugging"
  - "Case-insensitive matching via (?i) flag in pattern (user-controlled)"
  - "Short flag -r for --regexp (consistent with other mode flags)"
metrics:
  duration: 8
  completed_date: "2026-03-11"
  tests_added: 11
  lines_added: ~200
---

# Phase 06 Plan 01: Core Regex Support Summary

**One-liner:** Added regex crate dependency, InvalidRegexPattern error variant, find_by_regexp service function, and --regexp CLI flag for pattern-based window matching.

---

## What Was Built

### 1. Regex Dependency (Cargo.toml)
- Added `regex = "1.12"` to [dependencies]
- Latest stable version with excellent performance

### 2. InvalidRegexPattern Error (src/error.rs)
- New error variant with structured fields:
  ```rust
  InvalidRegexPattern { pattern: String, details: String }
  ```
- User-friendly error message format: `"Invalid regex pattern '{pattern}': {details}"`
- Constructor method: `AppError::invalid_regex_pattern(pattern, details)`
- 3 unit tests for display, constructor, and Error trait

### 3. find_by_regexp Function (src/window_service.rs)
- Signature: `find_by_regexp(windows, pattern) -> Result<Vec<&WindowInfo>, AppError>`
- Matches on both window title and app_name
- Returns Vec for disambiguation when multiple windows match
- Error handling:
  - Invalid regex pattern → InvalidRegexPattern error
  - No matches → WindowNotFound error with pattern
- 8 comprehensive unit tests covering:
  - Title matching, app_name matching
  - Case-insensitive with (?i) flag
  - Invalid regex error handling
  - Multiple matches returned
  - Regex metacharacters (.*, +, [], ^, $)
  - Empty pattern matching

### 4. --regexp CLI Flag (src/cli.rs)
- Added to Mode struct with #[group] attribute (mutually exclusive)
- Short flag: `-r`
- Long flag: `--regexp`
- Value name: `PATTERN`
- Positioned after `--window` for logical grouping

---

## Test Results

```
running 51 tests
test result: ok. 51 passed; 0 failed; 0 ignored
```

New tests added:
- error::tests: 3 tests for InvalidRegexPattern
- window_service::tests: 8 tests for find_by_regexp

---

## Deviations from Plan

None - plan executed exactly as written.

---

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Return `Vec<&WindowInfo>` | Multiple windows can match a regex pattern; caller decides how to handle disambiguation |
| User-controlled case sensitivity | `(?i)` flag in regex pattern gives users full control vs forced case-insensitive |
| Include pattern in error message | Helps users debug regex issues |
| Short flag `-r` | Consistent with other mode flags (`-w`, `-p`, `-i`) |

---

## Commits

| Hash | Message |
|------|---------|
| f5141a3 | feat(06-01): add regex crate and InvalidRegexPattern error variant |
| 4e29993 | feat(06-01): implement find_by_regexp in window_service.rs |
| 6575ebf | feat(06-01): add --regexp flag to CLI Mode group |

---

## Self-Check: PASSED

- [x] regex crate in Cargo.toml
- [x] InvalidRegexPattern error variant with constructor
- [x] find_by_regexp function implemented with tests
- [x] --regexp flag in CLI Mode struct
- [x] All 51 library tests pass
- [x] Code follows existing patterns

---

## Next Steps

Plan 06-02 will wire the --regexp flag into main.rs, completing the end-to-end regex targeting feature.
