# Fleck Linear Build Tasks

Source: `.plan/spec.md`

This is the primary task list. It is intentionally linear: agents should work from top to bottom. When a milestone needs frontend work, the frontend task is split into an adjacent `TASK-FE-*` item so a UI agent can take it without forcing "all core first, all frontend later".

Task statuses: `open`, `in_progress`, `done`, `blocked`, `deferred`.

## Phase 0: Project Foundation

### TASK-001: Scaffold App, Workspace, And Build System

Status: done

Requirements: REQ-001, REQ-003, REQ-036, REQ-046

Agent type: core/platform

Deliverables:
- Tauri app scaffold.
- Rust workspace with crates for core engine, desktop bridge, rendering integration, and CLI.
- React frontend scaffold wired into Tauri.
- Basic CI/build scripts.
- Architecture note documenting ownership: Rust owns document state, React owns UI, Skia owns rendering.

Acceptance criteria:
- App builds and launches a placeholder editor window.
- Rust core crate can be tested independently.
- CLI crate can compile even if commands are placeholders.
- React cannot own authoritative document state.

### TASK-FE-001: Build Placeholder Editor Shell

Status: done

Requirements: REQ-037, REQ-052

Agent type: frontend

Depends on: TASK-001

Deliverables:
- Initial editor shell with central canvas area, tool strip placeholder, inspector placeholder, layers placeholder, exports placeholder, history placeholder, status bar, and menu area.
- Tailwind, shadcn/ui, Radix UI, TanStack Query, and Zustand setup.

Acceptance criteria:
- Layout clearly prioritizes the canvas.
- Zustand stores only local UI state.
- Backend calls are routed through a shared API wrapper, even if still mocked.

## Phase 1: Workspace Files And Basic Document State

### TASK-002: Implement Versioned Workspace Document Model

Status: done

Requirements: REQ-002, REQ-004, REQ-005, REQ-006, REQ-007, REQ-009, REQ-010, REQ-011, REQ-013

Agent type: core

Deliverables:
- Versioned workspace model with metadata, canvas settings, layers, image objects, selections, guides, export areas, outputs, recipes, assets, history, and document settings.
- Stable IDs for workspace objects.
- Validation rules for references and required fields.

Acceptance criteria:
- Empty workspace can be created, serialized, deserialized, and validated.
- Layers, image objects, export areas, outputs, and recipes round-trip without data loss.
- Invalid object references are reported with explicit diagnostics.

### TASK-003: Implement `.fleck` Save And Load

Status: done

Requirements: REQ-004, REQ-012, REQ-013, REQ-014

Agent type: core/platform

Depends on: TASK-002

Deliverables:
- `.fleck` save/load pipeline.
- Embedded asset and linked asset representation.
- Migration framework for older versions.
- Warning path for newer unsupported versions.

Acceptance criteria:
- Workspace fixtures save and reopen with content intact.
- Embedded assets remain available after moving the file.
- Missing linked assets are reported with enough data to relink.
- Older fixture migrates safely; newer fixture warns without destructive load.

### TASK-FE-002: Build Open, Save, And Workspace File UI

Status: done

Requirements: REQ-012, REQ-014, REQ-046, REQ-050

Agent type: frontend

Depends on: TASK-003

Deliverables:
- Open workspace, save workspace, save as, open image placeholder flow.
- Dialogs for missing linked assets and unsupported newer versions.
- Recent-file UI placeholder.

Acceptance criteria:
- File operations use Tauri/native dialog commands.
- Missing assets and unsupported versions are visible to the user.
- UI does not directly parse or mutate workspace files.

## Phase 2: Command Engine, History, And Editor State

### TASK-004: Implement Command Registry And Undo/Redo

Status: done

Requirements: REQ-004, REQ-006, REQ-008, REQ-011, REQ-016, REQ-035, REQ-042, REQ-044, REQ-048

Agent type: core

Depends on: TASK-002

Deliverables:
- Rust command registry with typed parameters.
- Undoable operation model.
- Redo support.
- History entries with operation names.
- Progress and cancellation contract for long commands.

Acceptance criteria:
- Commands can be invoked by ID.
- Undo/redo restores document state for supported commands.
- History exposes operation names to frontend and CLI.
- Long-running command contract supports progress and cancellation.

### TASK-FE-003: Build Command Palette And History Panel

Status: open

Requirements: REQ-042, REQ-044, REQ-045, REQ-052

Agent type: frontend

Depends on: TASK-004

Deliverables:
- Fuzzy command palette UI.
- Recent commands, groups, descriptions, aliases, shortcuts, parameter prompts, repeat last, history, and context-aware ranking.
- History panel with undo/redo and jump-to-state where supported.

Acceptance criteria:
- Commands execute through the shared command API.
- Undo/redo buttons and shortcuts stay in sync with core history.
- Unsupported history jump behavior is disabled or hidden.

## Phase 3: Canvas Viewport And Rendering

### TASK-005: Implement Workspace Geometry And Viewport Core

Status: open

Requirements: REQ-005, REQ-023, REQ-024, REQ-025, REQ-026, REQ-027

Agent type: core/rendering

Depends on: TASK-002

Deliverables:
- Workspace/screen/layer/export-area coordinate transforms.
- Pan/zoom state.
- Snapping and guide geometry.
- Pixel-grid visibility and alignment rules.
- Transparency/checkerboard settings model.

Acceptance criteria:
- Coordinate conversions are unit tested.
- Pixel grid aligns to true pixel boundaries.
- Snapping can target pixels, layers, export areas, centers, edges, and common sizes.

### TASK-006: Implement Skia Rendering MVP

Status: open

Requirements: REQ-023, REQ-024, REQ-026, REQ-027, REQ-048

Agent type: core/rendering

Depends on: TASK-005

Deliverables:
- Skia-backed viewport renderer.
- Layer compositing preview for basic raster layers.
- Checkerboard transparency.
- Selection/export-area/guide/pixel-grid overlay hooks.

Acceptance criteria:
- Renderer displays layered raster content with alpha correctly.
- Pan and zoom work on representative documents.
- Rendering code does not own document truth.

### TASK-FE-004: Build Canvas Host And Navigation UI

Status: open

Requirements: REQ-024, REQ-026, REQ-027, REQ-037, REQ-045

Agent type: frontend

Depends on: TASK-005, TASK-006

Deliverables:
- Canvas host component or native rendering bridge mount.
- Pointer, wheel, trackpad, keyboard, pan, zoom, spacebar-pan, and viewport focus handling.
- Zoom controls: fit, selection, export area, 100%, pixel-perfect.
- Overlay toggles for checkerboard, guides, pixel grid, selections, transform handles, and export areas.

Acceptance criteria:
- UI events route to core/rendering commands.
- Canvas remains the default editor focus.
- Navigation feels usable before advanced editing tools exist.

## Phase 4: Layers And Imported Images

### TASK-007: Implement Raster Layers

Status: open

Requirements: REQ-006, REQ-016, REQ-027, REQ-049

Agent type: core

Depends on: TASK-004, TASK-006

Deliverables:
- Layer create/delete/duplicate/rename/reorder/group/merge/flatten/hide/lock/opacity/blend/clipping/mask/rasterize operations.
- Layer bounds and trim-to-visible-pixels.
- Required blend modes.

Acceptance criteria:
- Layer operations are undoable.
- Hidden and locked layers behave correctly.
- Blend modes produce deterministic render/export output.

### TASK-FE-005: Build Layers Panel And Layer Inspector

Status: open

Requirements: REQ-039, REQ-040, REQ-042, REQ-052

Agent type: frontend

Depends on: TASK-007

Deliverables:
- Layer list with visibility, lock, drag reorder, grouping, opacity, blend mode, add, delete, duplicate, merge, flatten, rename.
- Layer inspector controls.
- Layer row context menus.

Acceptance criteria:
- Layer mutations use undoable core commands.
- Locked/hidden states are visually and accessibly clear.
- Inspector and history update after layer operations.

### TASK-008: Implement Image Import And Image Objects

Status: open

Requirements: REQ-007, REQ-012, REQ-047, REQ-050

Agent type: core/platform

Depends on: TASK-003, TASK-007

Deliverables:
- Image decode/import pipeline.
- Placed image objects with source tracking, position, scale, rotation, opacity, crop, replacement, duplication, rasterization, and export inclusion.
- Drag/drop and clipboard import command hooks.

Acceptance criteria:
- Imported images retain source metadata where applicable.
- Image objects can be rasterized into editable layers.
- Replacement preserves object settings where practical.

### TASK-FE-006: Build Image Import And Object Inspector UI

Status: open

Requirements: REQ-039, REQ-046, REQ-050

Agent type: frontend

Depends on: TASK-008

Deliverables:
- Open image, paste image, drag image into workspace, replace image, reveal source, rasterize object UI.
- Image object inspector for position, scale, rotation, opacity, crop, and source state.

Acceptance criteria:
- Import flows use native/Tauri hooks.
- Linked, embedded, missing, and replaced asset states are distinguishable.
- Image object mutations use core commands.

## Phase 5: Export Areas And Basic Export

### TASK-009: Implement Export Areas And Outputs

Status: open

Requirements: REQ-002, REQ-009, REQ-010, REQ-028, REQ-029, REQ-032, REQ-033

Agent type: core

Depends on: TASK-004, TASK-006, TASK-007

Deliverables:
- Export area create/rename/resize/move/duplicate/tag/group/delete operations.
- Output add/remove/duplicate/update operations.
- Layer include/exclude rules.
- Export preview metadata and warning generation.

Acceptance criteria:
- Export areas are metadata, not pixels.
- Export area operations are undoable.
- Preview and warning data is available to UI and CLI.

### TASK-FE-007: Build Export Area Tool, Exports Panel, And Export Inspector

Status: open

Requirements: REQ-037, REQ-039, REQ-041, REQ-043

Agent type: frontend

Depends on: TASK-009

Deliverables:
- Export area tool.
- Exports panel listing areas and outputs.
- Export inspector with name, dimensions, position, padding, background, outputs, format, quality, scale, destination, and preview metadata.
- Context menus for export areas and outputs.

Acceptance criteria:
- Area selection syncs across canvas, inspector, and exports panel.
- Export warnings come from core preview metadata.
- Export area commands are available from panel, canvas context menu, and command palette.

### TASK-010: Implement Basic Export Pipeline

Status: open

Requirements: REQ-010, REQ-027, REQ-028, REQ-029, REQ-030, REQ-047, REQ-049

Agent type: core

Depends on: TASK-009

Deliverables:
- Default visible-non-transparent-bounds export.
- Export selected area.
- Export one export area.
- PNG/JPEG/WebP support first.
- Background, padding, trim, transparency, scale, quality, metadata, and layer participation handling.

Acceptance criteria:
- If no export areas exist, export behaves like a normal raster editor.
- Exported pixels match preview semantics.
- Transparent and solid-background exports both work.

### TASK-FE-008: Build Basic Export UI

Status: open

Requirements: REQ-029, REQ-032, REQ-033

Agent type: frontend

Depends on: TASK-010

Deliverables:
- Export selected area, export all placeholder, preview export result, copy export result, reveal exported file.
- Export preview panel/dialog showing crop, background, padding, transparency, dimensions, format, estimated size, warnings, filename, and destination.

Acceptance criteria:
- Export actions call backend export jobs.
- Warnings are visible before export.
- Reveal/copy actions use native integration where available.

## Phase 6: Selection And Pixel Editing

### TASK-011: Implement Selection Engine

Status: open

Requirements: REQ-008, REQ-016, REQ-028, REQ-029

Agent type: core

Depends on: TASK-004, TASK-007, TASK-010

Deliverables:
- Selection mask model.
- Rectangular, elliptical, lasso, polygon, magic wand, and color-range selection.
- Expand, contract, feather, invert, move, delete, copy, layer-from-selection, export-area-from-selection, and direct-export operations.

Acceptance criteria:
- Selection masks preserve alpha behavior.
- Selection-changing operations are undoable where applicable.
- Selection export and export-area creation use selected bounds/mask correctly.

### TASK-FE-009: Build Selection Tools And Selection UI

Status: open

Requirements: REQ-008, REQ-016, REQ-038, REQ-045

Agent type: frontend

Depends on: TASK-011

Deliverables:
- Selection tool UI for rectangular, elliptical, lasso, polygon, magic wand, and color range.
- Selection controls for expand, contract, feather, invert, move, delete, copy, layer-from-selection, export-area-from-selection, and direct export.
- Keyboard nudging and larger nudging.

Acceptance criteria:
- Selection commands are available through tools, menus, shortcuts, and command palette.
- Selection state is represented in canvas, inspector, and status bar.
- Clipboard routes use native/core clipboard contracts.

### TASK-012: Implement Core Pixel Editing Tools

Status: open

Requirements: REQ-015, REQ-016, REQ-027, REQ-048

Agent type: core

Depends on: TASK-011

Deliverables:
- Move, crop, resize image/canvas, rotate, flip, brush, pencil, eraser, fill bucket, gradient, color picker, clone, healing, blur, sharpen, and smudge backends.
- Pointer-event command integration.
- Low-latency brush stroke path.

Acceptance criteria:
- Tools modify only the intended layer/selection.
- Tool actions are undoable and named in history.
- Brush-like tools remain responsive on representative canvas sizes.

### TASK-FE-010: Build Tool Strip And Pixel Tool Options

Status: open

Requirements: REQ-015, REQ-016, REQ-038, REQ-045, REQ-052

Agent type: frontend

Depends on: TASK-012

Deliverables:
- Tool strip with move, select, lasso, magic wand, brush, eraser, fill, crop, text placeholder, shape placeholder, color picker, export area, hand/pan, and zoom.
- Tool option controls for implemented pixel tools.
- Keyboard-accessible tool selection with tooltips and active/disabled states.

Acceptance criteria:
- Selecting tools updates local UI state and core tool context.
- Disabled states reflect command availability.
- Tool actions are reachable through shortcuts and command palette.

## Phase 7: Text, Shapes, Filters, And Cleanup

### TASK-013: Implement Text And Shape Objects

Status: open

Requirements: REQ-017, REQ-018, REQ-023

Agent type: core/rendering

Depends on: TASK-006, TASK-012

Deliverables:
- Editable text object model and rasterization.
- Editable shape object model and rasterization.
- Skia preview rendering for text and shapes.

Acceptance criteria:
- Text and shapes remain editable until rasterized.
- Rasterized output matches preview closely enough for asset work.
- Text and shape actions are undoable.

### TASK-FE-011: Build Text And Shape UI

Status: open

Requirements: REQ-017, REQ-018, REQ-039

Agent type: frontend

Depends on: TASK-013

Deliverables:
- Text controls for font family, size, weight, color, alignment, line height, letter spacing, box resizing, outlines, shadows, and rasterization.
- Shape controls for fill, stroke, stroke width, opacity, corner radius, arrowhead style, alignment, and rasterization.

Acceptance criteria:
- Controls update selected objects through undoable commands.
- Rasterize actions are explicit.
- Invalid combinations are disabled or validated.

### TASK-014: Implement Filters, Adjustments, Metadata Cleanup

Status: open

Requirements: REQ-019, REQ-020, REQ-022, REQ-047, REQ-049

Agent type: core

Depends on: TASK-012

Deliverables:
- Brightness, contrast, exposure, saturation, hue, temperature, tint, grayscale, invert, blur, Gaussian blur, sharpen, pixelate, noise, threshold, posterize, outline, shadow, glow, stroke, round corners, and trim transparent pixels.
- Apply-to-layer, selection, export preview, and workspace output paths.
- EXIF/color profile/transparency/optimization/compression/palette/file-size cleanup utilities.

Acceptance criteria:
- Required filters have tested command paths.
- Filters respect selection masks and alpha.
- Cleanup settings affect output files as configured.

### TASK-FE-012: Build Filters, Adjustments, And Cleanup UI

Status: open

Requirements: REQ-019, REQ-020, REQ-022, REQ-039

Agent type: frontend

Depends on: TASK-014

Deliverables:
- Filter and adjustment controls.
- Cleanup controls for metadata, profiles, transparency, trim, compression, palette, and file-size estimate.
- Preview/apply/revert UI where applicable.

Acceptance criteria:
- Controls call core commands.
- Preview/apply/revert states are clear.
- Cleanup options are available in edit and export contexts where required.

## Phase 8: Background Removal

### TASK-015: Implement Local Background Removal

Status: open

Requirements: REQ-021, REQ-048

Agent type: core

Depends on: TASK-011, TASK-014

Deliverables:
- Optional local model/runtime integration.
- Preprocessing and postprocessing pipeline.
- Editable alpha-mask output.
- Progress, preview, apply, revert, cancellation, and refinement command hooks.

Acceptance criteria:
- No account or cloud upload is required.
- Long operation does not block canvas interaction where possible.
- If model/runtime packaging is deferred, the limitation is recorded in `.plan/decisions.md`.

### TASK-FE-013: Build Background Removal UI

Status: open

Requirements: REQ-021, REQ-048

Agent type: frontend

Depends on: TASK-015

Deliverables:
- Background removal action UI for layer, image object, selection, and pasted image.
- Progress, cancellation, preview, apply, revert, and manual cleanup/refinement controls.

Acceptance criteria:
- UI never implies cloud upload is required.
- Progress and cancellation are visible for long operations.
- Result can be applied as editable raster data.

## Phase 9: Full Export System, Presets, And CLI

### TASK-016: Implement Full Export Formats And Clipboard Outputs

Status: open

Requirements: REQ-030, REQ-050

Agent type: core/platform

Depends on: TASK-010, TASK-014

Deliverables:
- PNG, JPEG, WebP, AVIF, static GIF, BMP, TIFF, ICO, ICNS, applicable SVG rasterized export, and applicable PDF export.
- Clipboard outputs: PNG, JPEG, WebP, Base64, data URI, Markdown image, CSS background image.

Acceptance criteria:
- Format-specific options are validated.
- Unsupported format/platform cases fail with explicit diagnostics.
- Clipboard outputs work in common developer workflows.

### TASK-017: Implement Presets And Recipes

Status: open

Requirements: REQ-011, REQ-031, REQ-044, REQ-055, REQ-056

Agent type: core

Depends on: TASK-009, TASK-016

Deliverables:
- Built-in presets for favicon, `.ico`, PWA, Apple touch, macOS ICNS, Windows/iOS/Android/Electron/Tauri/browser extension icons, GitHub social preview, Open Graph, Twitter/X card, README banner, docs header, and useful app store sizes.
- Recipe storage and execution.
- Preset contribution format.

Acceptance criteria:
- Favicon and app icon recipes generate complete multi-output sets.
- Recipes are saved in and loaded from `.fleck` workspaces.
- Presets can be added without changing unrelated engine code.

### TASK-FE-014: Build Presets And Recipes UI

Status: open

Requirements: REQ-011, REQ-031, REQ-044

Agent type: frontend

Depends on: TASK-017

Deliverables:
- Preset browser/selector.
- Recipe create, save, apply, and command-palette execution UI.
- Preset and recipe controls in export panel/inspector where relevant.

Acceptance criteria:
- Built-in presets can be applied to appropriate targets.
- Workspace recipes reappear after reload.
- Recipe commands are discoverable in command palette.

### TASK-018: Implement Batch Export Jobs

Status: open

Requirements: REQ-029, REQ-034, REQ-048

Agent type: core

Depends on: TASK-017

Deliverables:
- Export all, selected, tag, preset, folder, and changed-area jobs.
- Destination remembering, overwrite confirmation contract, conflict handling, export report, failed-output retry, progress, and cancellation.

Acceptance criteria:
- Batch exports do not block the UI.
- Failed outputs are reported individually.
- Retry can run failed outputs without rerunning successes unnecessarily.

### TASK-FE-015: Build Batch Export UI

Status: open

Requirements: REQ-029, REQ-034, REQ-048

Agent type: frontend

Depends on: TASK-018

Deliverables:
- Batch export configuration for all/selected/tag/preset/folder.
- Export progress, cancellation, overwrite confirmation, conflict handling, report, and retry UI.

Acceptance criteria:
- Conflicts and overwrites require explicit user decisions.
- Failed outputs can be retried from the report.
- Long-running jobs show progress.

### TASK-019: Implement CLI

Status: open

Requirements: REQ-035, REQ-011, REQ-029, REQ-034

Agent type: core/CLI

Depends on: TASK-018

Deliverables:
- `fleck` CLI binary.
- Commands for export all, export one area, export by tag, export to folder, validate workspace exports, list export areas, list outputs, and run recipes.
- CI-friendly exit codes and machine-readable output option.

Acceptance criteria:
- CLI uses same Rust core as desktop.
- CLI exports a workspace fixture without launching GUI.
- Validation failures produce clear human-readable and machine-readable output.

## Phase 10: Native Integration, Settings, Accessibility, Plugins

### TASK-020: Implement Native Platform Integration

Status: open

Requirements: REQ-046, REQ-050, REQ-053

Agent type: core/platform

Depends on: TASK-003, TASK-008, TASK-018

Deliverables:
- Native menus, file dialogs, filesystem access, drag/drop, clipboard, notifications, open-with, recent files, file associations, system theme, and high-DPI support.
- Secure Tauri bridge permissions.

Acceptance criteria:
- Native operations respect macOS, Windows, and Linux conventions.
- Drag/drop and clipboard paths integrate with import/export commands.
- No filesystem action requiring approval is silently performed.

### TASK-021: Implement Settings Backend

Status: open

Requirements: REQ-051, REQ-052, REQ-053

Agent type: core/platform

Depends on: TASK-020

Deliverables:
- Settings storage model.
- Defaults for theme, accent, export format/location/padding/background/scaling, transparency grid, shortcuts, command palette, recent files, autosave, file format, background removal, performance, and telemetry.
- Shortcut customization data model.

Acceptance criteria:
- Telemetry is absent or opt-in.
- Settings survive app restart.
- Shortcut settings are consumable by UI and command routing.

### TASK-FE-016: Build Settings And Shortcut UI

Status: open

Requirements: REQ-045, REQ-051, REQ-052, REQ-053

Agent type: frontend

Depends on: TASK-021

Deliverables:
- Settings screens for all required settings.
- Shortcut editor with conflict detection.
- Platform-specific shortcut labels.
- Reduced motion and accessibility settings.

Acceptance criteria:
- Setting changes persist and update relevant UI where practical.
- Shortcut conflicts are detected and surfaced.
- Telemetry, if present, is opt-in only.

### TASK-022: Implement Plugin Runtime

Status: open

Requirements: REQ-056, REQ-044, REQ-055

Agent type: core/platform

Depends on: TASK-004, TASK-017, TASK-021

Deliverables:
- Command-oriented plugin API.
- Plugin support for commands, recipes, formats, transforms, generators, presets, and validators.
- Permission model with local execution, no silent network access, and explicit filesystem approval.

Acceptance criteria:
- Plugin commands appear in the command registry.
- Plugins cannot perform network or filesystem actions outside declared permissions.
- API versioning is documented.

### TASK-FE-017: Build Plugin Management UI

Status: open

Requirements: REQ-056, REQ-044, REQ-051

Agent type: frontend

Depends on: TASK-022

Deliverables:
- Plugin list/settings UI.
- Permission display and approval prompts.
- Plugin command visibility in command palette.

Acceptance criteria:
- Plugin permissions are understandable before risky actions.
- Plugin commands are grouped and attributable.
- UI does not imply silent network/filesystem access.

### TASK-FE-018: Accessibility And Frontend Polish Pass

Status: open

Requirements: REQ-037, REQ-038, REQ-043, REQ-045, REQ-052

Agent type: frontend

Depends on: TASK-FE-001 through TASK-FE-017

Deliverables:
- Keyboard navigation audit.
- Accessible labels for practical controls.
- Contrast and scalable text audit.
- Reduced-motion behavior.
- Focus management for dialogs, menus, panels, and command palette.

Acceptance criteria:
- Surrounding controls are navigable without a mouse where practical.
- Dialogs and menus expose accessible labels.
- Reduced motion setting affects nonessential animations.

## Phase 11: Performance, Tests, Packaging, Documentation

### TASK-023: Implement Performance Infrastructure

Status: open

Requirements: REQ-023, REQ-047, REQ-048

Agent type: core/rendering

Depends on: TASK-018

Deliverables:
- Background job scheduler.
- Cancellation tokens.
- Preview cache and invalidation strategy.
- Benchmarks for pan/zoom, brush strokes, layer toggling, export preview, export all, large resize, and workspace load.

Acceptance criteria:
- Long operations are cancellable where possible.
- Common interactions remain responsive during background work where possible.
- Benchmarks define target budgets before optimization claims are made.

### TASK-024: Build Test Fixtures, Golden Outputs, And Frontend Tests

Status: open

Requirements: REQ-004, REQ-006, REQ-008, REQ-010, REQ-015, REQ-030, REQ-036, REQ-037, REQ-044, REQ-049, REQ-052

Agent type: mixed

Depends on: TASK-023, TASK-FE-018

Deliverables:
- Workspace fixtures.
- Export golden images.
- Serialization compatibility fixtures.
- Pixel-operation test data.
- Component/integration tests for command palette, inspector, layers panel, exports panel, settings, shortcuts, menus, dialogs, and critical layouts.

Acceptance criteria:
- Core behavior can be verified without manual UI testing.
- Export changes can be compared against golden outputs.
- UI tests do not assume React owns document state.
- Accessibility regressions are caught for menus/dialogs where practical.

### TASK-025: Implement Cross-Platform Packaging

Status: open

Requirements: REQ-001, REQ-053, REQ-054

Agent type: platform

Depends on: TASK-024

Deliverables:
- macOS, Windows, and Linux packaging.
- Direct download/GitHub release artifacts.
- Homebrew cask, winget, AppImage, Flatpak, and installers where practical.
- Package-manager-friendly app naming.

Acceptance criteria:
- At least one package artifact builds per supported OS in CI.
- Installed app launches and registers expected file associations where supported.
- Unsupported distribution channels are recorded as deferred in `.plan/decisions.md`.

### TASK-026: Write Contributor And Architecture Documentation

Status: open

Requirements: REQ-001, REQ-013, REQ-055, REQ-056

Agent type: docs/core

Depends on: TASK-022, TASK-025

Deliverables:
- Architecture guide.
- Contribution guide for presets, recipes, image formats, packaging, translations, UI, bugs, plugin commands, docs, and background removal.
- File format documentation where practical.
- Plugin safety and API documentation.

Acceptance criteria:
- A new contributor can locate extension points without reading the whole codebase.
- Local-first, no-account, and no-cloud expectations are explicit.
- Plugin safety rules are documented.
