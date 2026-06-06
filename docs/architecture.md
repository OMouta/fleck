# Fleck Architecture

Fleck is split by ownership boundary so the desktop app, renderer, and CLI can share one source of truth.

## Ownership

- Rust core owns document state, command execution, undo/redo, file format, image operations, export logic, and long-running jobs.
- Skia owns viewport rendering and draws core-owned state. Rendering code must not become the document model.
- React owns interface composition, panels, dialogs, toolbars, command palette UI, and immediate interaction state.
- TanStack Query will coordinate async access to Rust-owned state.
- Zustand will store local UI state such as selected tool, active panel, hover state, and palette visibility.
- Tauri owns the native desktop shell, windows, menus, dialogs, filesystem access, drag/drop, clipboard, packaging, and the secure bridge.

## Crates

- `crates/fleck-core`: authoritative application engine.
- `crates/fleck-render`: rendering integration boundary.
- `crates/fleck-cli`: command-line entry point using `fleck-core`.
- `src-tauri`: desktop shell and Tauri command bridge.

## Frontend

The React app lives in `src/` and is built by Vite. It should call backend behavior through shared API wrappers and Tauri commands instead of owning or mutating document data directly.

## Current Scope

This scaffold intentionally does not implement the document model, renderer, or export pipeline. Those are covered by later tasks in `.plan/tasks.md`.
