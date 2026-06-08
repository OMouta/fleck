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

### TASK-004

Status: done

Evidence:
- Added core command infrastructure in `crates/fleck-core/src/command.rs`.
- Added `CommandRegistry`, `CommandDefinition`, command groups, aliases, shortcuts, parameter prompts, typed command parameters, and command context.
- Added `CommandEngine` with snapshot-backed undo/redo and redo truncation after new edits.
- Added workspace history synchronization with operation labels exposed through `Workspace.history`.
- Added `CommandRuntime`, `CancellationToken`, `ProgressSink`, and `CommandProgress` for long-running command plumbing.
- Added a concrete `workspace.rename` command to verify command invocation, typed parameters, validation, history labels, and undo/redo behavior.
- Added tests for registry listing, execution/history update, undo/redo restoration, redo truncation, cancellation, and progress reporting.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- Only infrastructure and one minimal workspace command are included. Real layer, selection, export, and pixel-editing commands remain with their linked implementation tasks.

### TASK-005

Status: done

Evidence:
- Added workspace geometry and viewport core in `crates/fleck-core/src/geometry.rs`.
- Added `Viewport` with workspace/screen coordinate conversion, pan-by-screen-delta, zoom-around-anchor, and visible workspace rect calculation.
- Added `LayerTransform` with local-to-workspace and workspace-to-local conversion, including scale and rotation.
- Added snapping settings and snapping helpers for pixels, layer bounds, export area bounds, centers, edges, guides, and common sizes.
- Added guide line extraction.
- Added pixel-grid visibility rules and integer pixel-boundary grid generation.
- Added overlay settings for checkerboard, guides, pixel grid, selections, transform handles, and export areas.
- Added tests for viewport conversion, zoom anchor stability, layer transform round-trip, snapping targets, common-size snapping, and pixel-grid visibility/alignment.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- This task provides geometry and viewport math only. Skia rendering, canvas event routing, and visual overlay drawing remain with TASK-006 and TASK-FE-004.

### TASK-006

Status: done

Evidence:
- Added `SkiaViewportRenderer` in `crates/fleck-render/src/lib.rs`.
- Added `RenderRequest`, `RenderedFrame`, `OverlaySummary`, and `RenderError` as the rendering boundary API.
- Added Skia raster surface creation and RGBA pixel readback.
- Added checkerboard transparency rendering.
- Added deterministic layer preview drawing with layer visibility, opacity, viewport pan, and zoom.
- Added overlay drawing hooks and reporting for export areas, guides, selections, transform handles, and pixel grid lines.
- Added tests for checkerboard/layer pixels, alpha compositing, pan/zoom behavior, overlay counts, and non-mutating workspace access.
- Added `skia-safe` and `thiserror` dependencies to `crates/fleck-render/Cargo.toml`.
- Verified `cargo test -p fleck-render`.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- The current layer render path is a Skia-backed preview of core layer bounds because the model does not yet carry decoded raster pixel buffers. Real image decoding and per-layer raster source binding remains for later asset/pixel-editing tasks.
- Canvas event routing and frontend host integration remain with TASK-FE-004.

### TASK-007

Status: done

Evidence:
- Added `crates/fleck-core/src/layer.rs` with core raster-layer operations for create, delete, duplicate, rename, reorder, grouping, merge down, flatten visible layers, visibility, locking, opacity, blend mode, clipping, masks, rasterize, and trim-to-visible-pixels.
- Registered undoable layer commands in `crates/fleck-core/src/command.rs`.
- Added typed command parameter helpers for booleans, numbers, optional strings, and object IDs.
- Added locked-layer protections for mutating layer operations while still allowing visibility and lock state changes.
- Added layer merge and flatten operations that produce deterministic composite bounds.
- Updated `crates/fleck-render/src/lib.rs` so hidden layers do not draw and layer blend modes map to Skia blend modes.
- Added core tests for layer operations, locked-layer rejection, command undo/redo, and command registry exposure.
- Added render tests for hidden layers and deterministic blend-mode output.
- Verified `cargo test -p fleck-core`.
- Verified `cargo test -p fleck-render`.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- Raster layer operations currently operate on layer metadata and preview bounds. Real decoded pixel buffers, pixel-level trim, and destructive pixel compositing remain deferred to image import and pixel-editing tasks.
- Frontend layer list, inspector controls, and context menus remain with TASK-FE-005.

### TASK-008

Status: done

Evidence:
- Added Rust-native image decoding through the `image` crate in `crates/fleck-core/src/image_import.rs`.
- Added decoded image metadata capture for dimensions, format, color type, and alpha.
- Added persistent `ImageAssetMetadata` and `ImageFormat` model fields for imported assets.
- Added embedded image import into `WorkspacePackage` with embedded asset blob storage.
- Added linked image import with source path tracking.
- Added placed image object creation with source asset, position, scale, rotation, opacity, crop bounds, rasterized layer link, and export inclusion.
- Added image object duplication, source replacement, reveal-linked-path helper, linked asset collection helper, and rasterization into an editable layer.
- Registered undoable command hooks for `image.import_linked`, `image.import_clipboard`, `image.import_drag_drop`, `image.place_asset`, `image.duplicate_object`, `image.replace_source`, and `image.rasterize_object`.
- Added tests for decode metadata/pixels, embedded package import, linked import, replacement preserving object settings, duplication, rasterization, command undo, and command registry exposure.
- Verified `cargo test -p fleck-core`.
- Verified `cargo fmt --all`.
- Verified `cargo test --workspace`.

Known gaps:
- Clipboard and drag/drop byte acquisition remain frontend/Tauri responsibilities in TASK-FE-006; TASK-008 provides command/API hooks for those flows.
- Rasterization currently creates an editable layer with correct object-derived bounds and linkage, but destructive pixel-buffer transfer remains deferred to later pixel-editing/export work.

### TASK-FE-005

Status: done

Evidence:
- Rebuilt the layers panel and inspector in `src/components/fleck/side-panel.tsx`: layer rows with visibility/lock toggles, selection, inline rename (double-click / F2 / inspector field), HTML5 drag-to-reorder with a drop indicator, per-row right-click context menus, and an inspector with an editable name, opacity slider (live preview, single undoable commit on release), full blend-mode dropdown, and duplicate/merge-down/delete actions.
- Added `src/components/ui/context-menu.tsx` (Radix `@radix-ui/react-context-menu`, added to `package.json`) mirroring the existing dropdown-menu styling.
- Added `src/lib/layer-commands.ts` to map UI actions to the exact core `layer.*` commands, generate the object IDs the core requires (`create`/`duplicate`/`group`/`flatten`), default the target to the current selection, and list blend modes.
- Routed every layer mutation through the command engine in `src/store/command-store.ts` (resolves layer params, auto-selects created layers) so all edits are undoable and appear in the history panel.
- Aligned `src/lib/command-registry.ts` palette entries to real core IDs (`layer.create/duplicate/rename/delete/merge_down/flatten/group`), replacing the prior placeholder IDs.
- Extended the mock backend in `src/lib/api.ts` to apply `layer.*` commands to the mock document and record history, seeded representative layers on workspace open, and removed the now-unused direct visibility/lock setters (also removed their hooks from `src/lib/queries.ts`).
- Widened `BlendMode` in `src/lib/fleck-data.ts` to the full core set and added the missing `history` value to `SideTab` in `src/store/ui-store.ts`.
- Accessibility: aria-labels/aria-pressed/aria-current on row controls, screen-reader hidden/locked status text, locked layers shown in the warning color with an inspector "Locked" badge, destructive/reorder actions disabled on locked layers, and keyboard-accessible reorder via the context menu (drag is mouse-only).
- Verified `npm run build` (tsc typecheck + vite build) passes.

Coverage impact:
- REQ-039: layer inspector controls implemented (covered for the layer surface; other object inspectors remain with their own TASK-FE-* items).
- REQ-040: list, visibility, locks, drag reorder, opacity, blend mode, add/delete/duplicate/merge/flatten/rename implemented. Grouping is wired to `layer.group` but rendered flat — hierarchical nesting deferred per DEC-FE-005-group-nesting, so REQ-040 stays partial.
- REQ-042: layer operations produce named, undoable history entries surfaced in the history panel.
- REQ-052: keyboard operability, accessible labels, and clear locked/hidden state for the layers panel; broader accessibility audit remains TASK-FE-018.

Known gaps:
- The dev mock does not snapshot-revert document state on undo (pre-existing mock limitation; the real Rust engine restores via snapshots). Forward layer operations and history labels are fully exercised.
- Hierarchical group rendering is deferred (DEC-FE-005-group-nesting).

### TASK-FE-006

Status: done

Evidence:
- Added an Images side-panel tab in `src/components/fleck/side-panel.tsx`: a placed-image list with per-state source icons, a row context menu (duplicate / replace source / rasterize / reveal), and an image-object inspector showing source state, source name/path, format + dimensions, read-only transform (position/scale/rotation/opacity/crop), a missing-source warning, and Replace/Reveal/Rasterize/Duplicate actions. Converted the panel tabs to icon+count so four tabs fit the rail (labels in tooltip/aria).
- Added import flows in `src/lib/image-import.ts`: open (native picker → `image.import_linked`), paste (clipboard → `image.import_clipboard`), drag-drop (`image.import_drag_drop`), replace (`image.replace_source`), and reveal source. Each acquires through a native hook then places via the undoable command engine and reveals the Images panel.
- Added `src/lib/image-commands.ts` resolver: generates the object/asset/layer IDs core `image.*` commands require, defaults the target to the selected object, and derives a name from the picked path.
- Wired image-command resolution + created-object selection into `src/store/command-store.ts`; added `selectedImageObjectId` to `src/store/ui-store.ts` and the `images` SideTab.
- Extended the mock backend in `src/lib/api.ts`: added asset/image-object document state, `get_image_objects` projecting the joined `ImageObject` DTO with resolved source state, `image.*` mutation application + history, native acquisition mocks (`pick_image_file`, clipboard/drop/replacement asset acquisition, reveal), and seeded placed images spanning linked/embedded/missing on open.
- Added the `ImageObject`/`ImageSourceState` DTO and the `image_object` command group to `src/lib/fleck-data.ts`; surfaced `image.duplicate_object`/`image.rasterize_object` in the palette (`command-registry.ts`) with the new Images group label/icon/order + context boost in `command-palette.tsx`.
- Wired imports into surfaces: File menu (Open image → flow, new Paste image item), canvas drag-and-drop with a drop overlay (`canvas.tsx`), and a global ⌘/Ctrl+V paste guard in `App.tsx`.
- Verified `npm run build` (tsc typecheck + vite build) passes.

Coverage impact:
- REQ-039: image-object inspector implemented showing all listed properties + source state. Transform editing is read-only (no core command) — deferred per DEC-FE-006-image-transform-edit, so REQ-039 stays partial for image objects.
- REQ-046: import/replace/reveal/clipboard/drag-drop route through the `api` bridge (Tauri-forwarding with a browser mock fallback), keeping native acquisition in the platform layer.
- REQ-050: image paste, drag-image-in, and replace flows implemented; linked / embedded / missing / replaced asset states are visually distinguishable in the list and inspector.

Known gaps:
- Image-object transform/opacity/crop edit and image-object delete have no core command yet (DEC-FE-006-image-transform-edit).
- Import flows are reachable from the File menu, canvas drop, Images panel header, and ⌘V; they are not palette commands (import needs native acquisition). Palette exposes duplicate/rasterize.
- Mock undo does not snapshot-revert (shared pre-existing limitation).

### TASK-011

Status: done

Evidence:
- Added `crates/fleck-core/src/selection.rs` with selection mask creation and operations for rectangular, elliptical, lasso, polygon, magic wand, color range, expand, contract, feather, invert, move, delete, copy metadata, layer-from-selection, and export-area-from-selection.
- Extended `Selection` with an optional `SelectionMask` carrying per-pixel alpha values, preserving compatibility with existing serialized selections that do not have explicit masks.
- Added validation for selection bounds, source-layer references, and mask alpha length in `crates/fleck-core/src/model.rs`.
- Registered selection commands in `crates/fleck-core/src/command.rs`, including undoable selection-changing operations and non-undoable copy/direct-export preparation hooks.
- Kept existing render/export selection paths compatible by updating render test fixtures for the optional mask field.
- Added tests for mask alpha behavior, feather/invert/move, conversion to layer/export area, command registration, and command undo.
- Verified `cargo fmt --all`.
- Verified `cargo test -p fleck-core`.
- Verified `cargo test --workspace`.

Coverage impact:
- REQ-008: selection types, expansion/contraction, feathering, inversion, movement, deletion of active selection state, copy metadata, layer creation, export area creation, and direct-export command hooks are implemented. True selected-pixel deletion/copy/move awaits pixel-buffer editing per DEC-011-selection-pixel-buffer, so REQ-008 stays partial.
- REQ-016: selection movement hooks exist. TASK-012 added editable raster pixels and selection-masked writes; selection extraction/duplication/paste remains partial.
- REQ-028 and REQ-029: selection bounds can create export areas and feed direct selection export hooks; full batch/export UI behavior remains with later export/frontend tasks.

Known gaps:
- Magic wand and color range currently build masks from provided bounds/tolerance metadata because layer pixel sampling is not available yet.
- Copy/direct export commands validate and expose selection mask/bounds metadata, but encoded clipboard/export payload generation remains in rendering/platform integration.

### TASK-012

Status: done

Evidence:
- Added optional `RasterPixels` storage to `Layer` in `crates/fleck-core/src/model.rs`, with validation for byte length and backward-compatible serialization.
- Updated layer creation/flattening in `crates/fleck-core/src/layer.rs` to initialize transparent RGBA raster buffers.
- Added `crates/fleck-core/src/pixel.rs` with core pixel editing backends for move, crop, resize layer, resize canvas origin, rotate, flip, brush, pencil, eraser, fill bucket, gradient, color picker, clone, healing, blur, sharpen, and smudge.
- Pixel writes are gated by `SelectionMask`/selection bounds where a selection is supplied, so tools modify only the intended selection region.
- Registered undoable `pixel.*` tool commands in `crates/fleck-core/src/command.rs`; color picker is registered as non-undoable.
- Updated `crates/fleck-render/src/lib.rs` so layers with raster buffers render/export actual pixels, while older metadata-only layers still use the deterministic preview fallback.
- Added tests for raster brush/eraser behavior, selection-limited fill, crop/resize/rotate/flip shape updates, command registration, and command undo.
- Verified `cargo fmt --all`.
- Verified `cargo test -p fleck-core`.
- Verified `cargo test --workspace`.

Coverage impact:
- REQ-015: core backends exist for the listed raster tools except text/shape/line/arrow/rounded-rectangle, which are planned separately under TASK-013 and TASK-FE-011.
- REQ-016: selected-region pixel writes, erasing, movement hooks, layer movement, and layer transforms are implemented. Selection extraction/duplication/paste-into-selection remain partial refinement.
- REQ-027: alpha editing works through RGBA raster pixels and eraser/fill/gradient/brush alpha behavior.
- REQ-048: operations are synchronous but deterministic and undoable; benchmarked responsiveness and background scheduling remain with TASK-023.

Known gaps:
- Healing, blur, sharpen, and smudge use simple local algorithms suitable for an MVP backend, not production-grade image processing.
- `pixel.resize_canvas` updates workspace canvas origin because the current infinite-canvas model has no finite canvas dimensions.
- Brush stroke batching/performance budgets are not yet defined.
