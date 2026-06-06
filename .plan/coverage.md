# Coverage Audit

Source: `.plan/spec.md`

Primary task list: `.plan/tasks.md`

This is a pre-implementation coverage map. No requirement is marked implemented yet.

## Requirement Coverage

- REQ-001: planned by TASK-001, TASK-025, TASK-026.
- REQ-002: planned by TASK-002, TASK-005, TASK-009, TASK-010, TASK-FE-004, TASK-FE-007.
- REQ-003: planned by TASK-001.
- REQ-004: planned by TASK-002, TASK-003, TASK-004, TASK-024.
- REQ-005: planned by TASK-002, TASK-005.
- REQ-006: planned by TASK-002, TASK-004, TASK-007, TASK-FE-005, TASK-024.
- REQ-007: planned by TASK-002, TASK-008.
- REQ-008: planned by TASK-004, TASK-011, TASK-FE-009, TASK-024.
- REQ-009: planned by TASK-002, TASK-009, TASK-FE-007.
- REQ-010: planned by TASK-002, TASK-009, TASK-010, TASK-024.
- REQ-011: planned by TASK-002, TASK-004, TASK-017, TASK-FE-014, TASK-019.
- REQ-012: planned by TASK-003, TASK-008, TASK-FE-002.
- REQ-013: planned by TASK-002, TASK-003, TASK-026.
- REQ-014: planned by TASK-003, TASK-FE-002.
- REQ-015: planned by TASK-012, TASK-FE-010, TASK-024.
- REQ-016: planned by TASK-004, TASK-007, TASK-011, TASK-FE-009, TASK-012, TASK-FE-010.
- REQ-017: planned by TASK-013, TASK-FE-011.
- REQ-018: planned by TASK-013, TASK-FE-011.
- REQ-019: planned by TASK-014, TASK-FE-012.
- REQ-020: planned by TASK-014, TASK-FE-012.
- REQ-021: planned by TASK-015, TASK-FE-013.
- REQ-022: planned by TASK-014, TASK-FE-012.
- REQ-023: planned by TASK-005, TASK-006, TASK-013, TASK-023.
- REQ-024: planned by TASK-005, TASK-006, TASK-FE-004.
- REQ-025: planned by TASK-005.
- REQ-026: planned by TASK-005, TASK-006, TASK-FE-004.
- REQ-027: planned by TASK-005, TASK-006, TASK-FE-004, TASK-007, TASK-010, TASK-012.
- REQ-028: planned by TASK-009, TASK-010, TASK-011.
- REQ-029: planned by TASK-009, TASK-FE-007, TASK-010, TASK-FE-008, TASK-011, TASK-018, TASK-FE-015, TASK-019.
- REQ-030: planned by TASK-010, TASK-016, TASK-024.
- REQ-031: planned by TASK-017, TASK-FE-014.
- REQ-032: planned by TASK-009, TASK-FE-007, TASK-FE-008.
- REQ-033: planned by TASK-009, TASK-FE-007, TASK-FE-008.
- REQ-034: planned by TASK-018, TASK-FE-015, TASK-019.
- REQ-035: planned by TASK-004, TASK-019.
- REQ-036: planned by TASK-001, TASK-FE-001, TASK-024.
- REQ-037: planned by TASK-FE-001, TASK-FE-004, TASK-FE-007, TASK-FE-018, TASK-024.
- REQ-038: planned by TASK-FE-009, TASK-FE-010, TASK-FE-018.
- REQ-039: planned by TASK-FE-005, TASK-FE-006, TASK-FE-007, TASK-FE-011, TASK-FE-012.
- REQ-040: planned by TASK-FE-005.
- REQ-041: planned by TASK-FE-007.
- REQ-042: planned by TASK-004, TASK-FE-003, TASK-FE-005.
- REQ-043: planned by TASK-FE-007, TASK-FE-018.
- REQ-044: planned by TASK-004, TASK-FE-003, TASK-017, TASK-FE-014, TASK-022, TASK-FE-017, TASK-024.
- REQ-045: planned by TASK-FE-003, TASK-FE-004, TASK-FE-009, TASK-FE-010, TASK-FE-016, TASK-FE-018.
- REQ-046: planned by TASK-001, TASK-FE-002, TASK-FE-006, TASK-020.
- REQ-047: planned by TASK-008, TASK-010, TASK-014, TASK-023.
- REQ-048: planned by TASK-004, TASK-006, TASK-012, TASK-015, TASK-FE-013, TASK-018, TASK-FE-015, TASK-023.
- REQ-049: planned by TASK-007, TASK-010, TASK-014, TASK-024.
- REQ-050: planned by TASK-FE-002, TASK-008, TASK-FE-006, TASK-016, TASK-020.
- REQ-051: planned by TASK-021, TASK-FE-016, TASK-FE-017.
- REQ-052: planned by TASK-FE-001, TASK-FE-003, TASK-FE-005, TASK-FE-010, TASK-FE-016, TASK-FE-018, TASK-021, TASK-024.
- REQ-053: planned by TASK-020, TASK-021, TASK-FE-016, TASK-025.
- REQ-054: planned by TASK-025.
- REQ-055: planned by TASK-017, TASK-022, TASK-026.
- REQ-056: planned by TASK-017, TASK-022, TASK-FE-017, TASK-026.

## Audit Result

- Missing coverage: none at planning level.
- Partial coverage: all requirements are only planned; implementation evidence is not yet available.
- Orphan tasks: none.
- Deferred work: none yet.
- Scope creep: none identified; all tasks trace to `.plan/spec.md`.

## Evidence Rules For Future Agents

Do not mark a requirement covered without adding evidence here. Evidence should include changed files, tests run, manual checks, and known gaps. If a task implements only part of a requirement, keep the requirement partial and name the missing behavior.

## Implementation Evidence

### TASK-001

Status: done

Evidence:
- Added Vite React app scaffold in `src/`, `index.html`, `vite.config.ts`, `tsconfig.json`, and `package.json`.
- Added Tauri v2 desktop scaffold in `src-tauri/`.
- Added Rust workspace with `crates/fleck-core`, `crates/fleck-render`, `crates/fleck-cli`, and `src-tauri`.
- Added architecture note in `docs/architecture.md`.
- Added CI workflow in `.github/workflows/ci.yml`.
- Verified `npm install`.
- Verified `npm run build`.
- Verified `npm run tauri -- --version` reports `tauri-cli 2.11.2`.
- Verified Vite dev server responded with HTTP 200 at `http://127.0.0.1:1420`.
- Verified `cargo test --workspace`.
- Verified `npm run desktop:build`.
- Verified built app process starts from `target/release/fleck-desktop.exe`, then stopped it.

Known gaps:
- In-app browser verification could not run because the Browser backend was unavailable for `iab`.

### TASK-002

Status: done

Evidence:
- Added versioned workspace model types in `crates/fleck-core/src/model.rs`.
- Added stable `ObjectId` type and `CURRENT_WORKSPACE_FORMAT_VERSION`.
- Added document fields for metadata, canvas settings, layers, image objects, selections, guides, export areas, outputs, recipes, assets, object groups, history, and document settings.
- Added validation for duplicate IDs, missing references, invalid format versions, invalid opacity, non-positive export bounds, empty output filenames, and invalid output sizing.
- Added JSON serialization/deserialization coverage through serde.
- Added tests for empty workspace validation, full JSON round-trip, duplicate IDs, missing asset references, and invalid output settings.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- This task defines the model and validation only. Save/load persistence, migrations, and durable `.fleck` container behavior remain for TASK-003.

### TASK-003

Status: done

Evidence:
- Added `.fleck` package persistence in `crates/fleck-core/src/persistence.rs`.
- Added `WorkspacePackage` with `file_format_version`, `workspace`, and `embedded_assets`.
- Added JSON save/load helpers for readers, writers, strings, and paths.
- Added package validation for embedded asset blob consistency.
- Added linked asset missing-file reports with asset ID, display name, original path, and resolved path for relinking UI.
- Added migration path for legacy v0 package shape into the current package format.
- Added warning path for newer file and workspace format versions when the known shape can still be read.
- Added tests for current save/load round-trip, path save/load, embedded asset storage requirements, legacy migration, newer-version warnings, and missing linked asset metadata.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- The `.fleck` package is currently an inspectable JSON envelope. A compressed/archive container can be introduced later if the project records that decision.
