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

## Document Model

- A workspace is an infinite board containing areas and placed image objects.
- Areas are the design/export containers. Each area owns its own layer stack and export/output settings.
- Layers are never workspace-level document state. Layer commands require an area target when creating or flattening layers, and layer queries are scoped to the selected area.
- Placed image objects remain non-raster workspace objects until rasterized.
- Rasterizing a linked image intersects the object with areas and writes real raster pixels into each intersecting area's layer stack. If no area intersects, the core creates an area from the image bounds first.

## Rendering And Export

- The renderer reads core-owned workspace state and draws area checkerboards, area outlines, area layers, placed image previews, guides, selections, and grid overlays.
- Export jobs export areas from their own layers. Workspace-level layer filtering is not part of the model.
- React may hold draft interaction state, such as an in-progress area drag, but committed areas, layers, pixels, and exports live in Rust-owned state.
