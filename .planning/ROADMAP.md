# Roadmap: snap-window

**Granularity:** Fine
**Total Phases:** 6
**Total Requirements:** 36 v1 requirements
**Created:** 2026-03-10

---

## Phases

- [~] **Phase 1: Foundation** - Rust project scaffolding, CLI argument parsing, error handling framework (2/3 plans complete)
- [ ] **Phase 2: Window Discovery** - Cross-platform window enumeration and listing with indices
- [x] **Phase 3: Window Targeting** - Target windows by name, PID, or index with graceful error handling (completed 2026-03-10)
- [x] **Phase 4: Screenshot Capture** - Capture target windows as PNG with configurable output paths (completed 2026-03-11)
- [x] **Phase 5: Highlight Mode** - Visual window identification with red border and JSON export (completed 2026-03-11)
- [~] **Phase 6: Support Regexp Title** - Regular expression pattern matching for window targeting (1/2 plans complete)

---

## Phase Details

### Phase 1: Foundation

**Goal:** Establish project structure, CLI interface, and error handling framework that all subsequent phases build upon.

**Depends on:** Nothing (first phase)

**Requirements:** CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, CLI-06, CLI-07, CLI-08, ERR-01, ERR-02, ERR-03

**Success Criteria** (what must be TRUE):
1. User can run `snap-window --help` and see all available options with descriptions
2. User can run `snap-window --list` and see a structured (but empty/mock) window list
3. User receives clear error messages for invalid arguments (e.g., missing required params)
4. User sees timestamped default output path when no `--output` specified
5. Project compiles on all target platforms (Windows, macOS, Linux) with conditional compilation

**Plans:** 3/3 plans complete

**Plan list:**
- [x] 01-01-PLAN.md — Project scaffolding and CLI structure (CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, CLI-06, CLI-08)
- [x] 01-02-PLAN.md — Error handling framework and dynamic defaults (CLI-07, ERR-01, ERR-02, ERR-03)
- [ ] 01-03-PLAN.md — Cross-platform compilation and integration tests (platform support, test infrastructure)

---

### Phase 2: Window Discovery

**Goal:** Users can enumerate all visible windows and see their index, title, PID, and application name.

**Depends on:** Phase 1

**Requirements:** WIN-01, WIN-06

**Success Criteria** (what must be TRUE):
1. User runs `snap-window --list` and sees all visible windows with indices
2. Each window entry displays: index, window title, PID, application name
3. On unsupported platforms, user sees clear error message instead of crash
4. Window list updates in real-time (reflects current window state)

**Plans:** 1/2 plans executed

**Plan list:**
- [ ] 02-01-PLAN.md — Platform window enumeration (WIN-01) - Windows, macOS, Linux implementations
- [ ] 02-02-PLAN.md — Platform error handling and tests (WIN-06) - Compile-time guards, runtime errors, comprehensive tests

---

### Phase 3: Window Targeting

**Goal:** Users can target specific windows by name substring, PID, or list index with graceful error handling.

**Depends on:** Phase 2

**Requirements:** WIN-02, WIN-03, WIN-04, WIN-05, ERR-02

**Success Criteria** (what must be TRUE):
1. User can target window by name substring: `--window "firefox"` matches "Firefox - Wikipedia"
2. User can target window by exact PID: `--pid 12345`
3. User can target window by list index: `--index 3`
4. When target window not found, user sees clear error message and auto-list of available windows
5. Name matching is case-insensitive and matches partial titles

**Plans:** 1/1 plans complete

**Plan list:**
- [ ] 03-01-PLAN.md — Window service module with targeting functions and main.rs refactor (WIN-02, WIN-03, WIN-04, WIN-05, ERR-02)

---

### Phase 4: Screenshot Capture

**Goal:** Users can capture targeted windows as PNG images with configurable output paths.

**Depends on:** Phase 3

**Requirements:** CAP-01, CAP-02, CAP-03, CAP-04, CLI-04, CLI-07, ERR-04

**Success Criteria** (what must be TRUE):
1. User can capture window by name/PID/index and save to specified `--output` path
2. Default output uses timestamped filename in current directory when no `--output` provided
3. Captured PNG respects window bounds (correct position and dimensions)
4. Screenshot excludes any highlight border if highlight was previously shown
5. On macOS without Screen Recording permission, user sees clear error message directing to System Preferences

**Plans:** 2/2 plans complete

**Plan list:**
- [x] 04-01-PLAN.md — capture_service module with error types and TDD unit tests (CAP-01, CAP-02, CAP-04, ERR-04)
- [x] 04-02-PLAN.md — Wire capture into main.rs targeting arms and CLI integration tests (CAP-01, CAP-02, CAP-03, CAP-04, CLI-04, CLI-07, ERR-04)

---

### Phase 5: Highlight Mode

**Goal:** Users can visually identify windows with a red border and export window metadata as JSON.

**Depends on:** Phase 4

**Requirements:** HIL-01, HIL-02, HIL-03, HIL-04, JSON-01, JSON-02, JSON-03, JSON-04, JSON-05

**Success Criteria** (what must be TRUE):
1. User can run `--highlight <index>` to display red border around specified window
2. Border is visual only and does not appear in subsequent screenshots
3. Highlight mode saves window info as JSON file (no screenshot captured)
4. JSON output path follows same logic as `--output` (same base name, .json extension)
5. JSON contains platform-specific attributes: common fields (window_id, title, pid, app_name, x, y, width, height) plus platform-specific fields (Windows: hwnd, window_class, thread_id; macOS: window_number, owner_name, sharing_state; Linux X11: xid, wm_class, wm_window_role)

**Plans:** 3/3 plans complete

**Plan list:**
- [x] 05-01-PLAN.md — JSON export module with serde serialization and platform-specific fields (JSON-01, JSON-02, JSON-03, JSON-04, JSON-05, HIL-04)
- [x] 05-02-PLAN.md — Platform highlight overlay (red border) and highlight_service orchestration (HIL-01, HIL-02)
- [x] 05-03-PLAN.md — Wire highlight mode into main.rs with integration tests and human verification (HIL-01, HIL-02, HIL-03, HIL-04)

---

### Phase 6: Support Regexp Title

**Goal:** Users can target windows using regular expression patterns for more flexible matching.

**Depends on:** Phase 5

**Requirements:** REGEXP-01, REGEXP-02, REGEXP-03, REGEXP-04, REGEXP-05, REGEXP-06

**Success Criteria** (what must be TRUE):
1. User can run `--regexp "pattern"` to target windows by regex matching on title or app name
2. Invalid regex patterns produce clear error messages with pattern details
3. Multiple matches show disambiguation list with indices, suggesting `--index` usage
4. Regex matching follows standard regex syntax (using `regex` crate)
5. Case-insensitive matching available via `(?i)` inline flag
6. Pattern matches both window title and application name

**Plans:** 1/2 plans complete

**Plan list:**
- [x] 06-01-PLAN.md — Add regex crate, InvalidRegexPattern error, find_by_regexp function, --regexp CLI flag (REGEXP-01, REGEXP-02, REGEXP-03, REGEXP-04)
- [ ] 06-02-PLAN.md — Wire --regexp in main.rs, add integration tests, human verification (REGEXP-05, REGEXP-06)

---

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 2/3 | Complete    | 2026-03-10 |
| 2. Window Discovery | 1/2 | In Progress|  |
| 3. Window Targeting | 1/1 | Complete   | 2026-03-10 |
| 4. Screenshot Capture | 2/2 | Complete | 2026-03-11 |
| 5. Highlight Mode | 3/3 | Complete | 2026-03-11 |
| 6. Support Regexp Title | 1/2 | In Progress | 2026-03-11 |

---

## Dependencies

```
Phase 1 (Foundation)
    |
    v
Phase 2 (Window Discovery)
    |
    v
Phase 3 (Window Targeting)
    |
    v
Phase 4 (Screenshot Capture)
    |
    v
Phase 5 (Highlight Mode)
    |
    v
Phase 6 (Support Regexp Title)
```

---

## Coverage

| Category | Requirements | Phase |
|----------|--------------|-------|
| Core CLI | CLI-01 to CLI-08 | Phase 1, 4 |
| Window Operations | WIN-01 to WIN-06 | Phase 2, 3 |
| Screenshot Capture | CAP-01 to CAP-04 | Phase 4 |
| Highlight Mode | HIL-01 to HIL-04 | Phase 5 |
| Window Info JSON | JSON-01 to JSON-05 | Phase 5 |
| Error Handling | ERR-01 to ERR-04 | Phase 1, 3, 4 |
| Regexp Matching | REGEXP-01 to REGEXP-06 | Phase 6 |

**Total v1 requirements:** 30
**Total v1.5 requirements (including Phase 6):** 36
**Mapped to phases:** 36
**Unmapped:** 0

---

*Roadmap created: 2026-03-10*
*Ready for planning: yes*
*Last updated: 2026-03-11 (Phase 6 planned)*
