# Decisions And Ambiguities

Source: `.plan/spec.md`

## Decisions

- DEC-001: Tasks are linear, with frontend tasks split into adjacent `TASK-FE-*` items only when a build step needs frontend work.
  - Primary task list: `.plan/tasks.md`
  - Reason: user wants agents to build in order, not complete all core work before all frontend work.

- DEC-002: Requirements are grouped into 56 stable IDs instead of preserving every bullet as a separate requirement.
  - Reason: the spec is broad and long; grouped requirements preserve traceability by spec section while keeping the task system usable for agents.
  - Constraint: future agents must not silently drop sub-bullets inside a grouped requirement.

## Ambiguities To Resolve Before Implementation Depends On Them

- AMB-001: Target minimum versions for macOS, Windows, and Linux are not specified.
- AMB-002: Exact Skia integration path is not specified.
- AMB-003: The `.fleck` file container format is not specified, only required properties.
- AMB-004: Required export quality thresholds and benchmark budgets are not specified.
- AMB-005: Background removal model/runtime choice and acceptable packaged model size are not specified.
- AMB-006: Telemetry is allowed only if opt-in, but the spec does not decide whether telemetry exists.
- AMB-007: Plugin implementation technology is not specified.
- AMB-008: Which app store image sizes are "useful" is not specified.
- AMB-009: Which SVG/PDF export cases are applicable is not specified.

## Deferrals

- DEC-FE-005-group-nesting: TASK-FE-005 wires the layer "Group" action to the core `layer.group` command (undoable, in history), but the layers panel renders groups as a flat header row rather than a collapsible nested tree.
  - Affected requirement: REQ-040 (grouping).
  - Reason: the frontend `Layer` DTO (`src/lib/fleck-data.ts`) is a flat list with no parent/child or expansion fields, so hierarchy cannot be rendered yet. The core model carries group membership, but `get_layers` does not project it.
  - Resolution path: extend the layer DTO with group/parent + collapsed state when the real `get_layers` bridge lands (TASK-020 territory), then render indentation/collapse. Tracked as partial coverage for REQ-040.

- DEC-FE-006-image-transform-edit: the image-object inspector (TASK-FE-006) shows position, scale, rotation, opacity, and crop **read-only**, with editable mutations limited to replace-source, rasterize, duplicate, and reveal-source.
  - Affected requirement: REQ-039 (image object inspector).
  - Reason: `fleck-core` (TASK-008) registered only import/place/duplicate/replace/rasterize image commands — there is no `image.set_position/scale/rotation/opacity/crop` or `image.delete_object` command to call, and adding core commands is out of scope for a frontend task.
  - Resolution path: add image-object transform/opacity/crop/delete commands to the core, then make the inspector fields editable through them (a follow-up core task). Tracked as partial coverage for REQ-039.

- DEC-011-selection-pixel-buffer: TASK-011 implements selection mask state, geometry operations, undoable command hooks, and conversion/export bounds, but destructive selected-pixel operations are metadata-only until raster layers carry editable pixel buffers.
  - Affected requirements: REQ-008 (delete/copy/move selected pixels), REQ-016 (pixel movement).
  - Reason: at TASK-011 time, the `Layer` model stored layer bounds and preview metadata, not per-layer pixel buffers. Writing true selected-pixel deletion, copy payloads, or move extraction there would have required inventing the pixel editing substrate that TASK-012 owned.
  - Resolution path: partially resolved by TASK-012, which adds layer raster buffers and selection-masked pixel writes. Selection-specific move extraction/duplication/paste remains a later refinement.

- DEC-012-pixel-tools-mvp: TASK-012 implements practical raster tool backends with simple local algorithms rather than production-grade photo-editing quality.
  - Affected requirements: REQ-015, REQ-016, REQ-048.
  - Reason: Fleck is still building the core substrate. The smallest useful implementation is an undoable RGBA raster buffer with deterministic tools, selection masking, and renderer/export visibility. Advanced brush engines, high-quality healing, transform handles, asynchronous stroke coalescing, and tuned performance budgets belong in later focused work.
  - Resolution path: TASK-023 should add benchmark budgets for representative brush strokes and large edits; future pixel-tool refinement can improve algorithms without changing the command surface.

## Environment Gaps

- ENV-001: Resolved. Local Rust verification was initially blocked because `rustc` and `cargo` were not installed in the current shell.
  - Affected task: TASK-001.
  - Resolution: Rust was installed, `cargo test --workspace` passed, `npm run desktop:build` passed, and the built app process started successfully.
