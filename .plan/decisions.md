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

## Environment Gaps

- ENV-001: Resolved. Local Rust verification was initially blocked because `rustc` and `cargo` were not installed in the current shell.
  - Affected task: TASK-001.
  - Resolution: Rust was installed, `cargo test --workspace` passed, `npm run desktop:build` passed, and the built app process started successfully.
