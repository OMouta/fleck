/**
 * Frontend ⇆ core glue for selection operations.
 *
 * Mirrors `fleck-core::command`'s `selection.*` registry: the canvas creates
 * selections by drag, the HUD/menus/palette dispatch follow-up edits, and the
 * Rust engine remains the source of truth (every action is undoable and named
 * in history). This module owns the frontend-only concerns:
 *
 *  1. Generating the stable object IDs the core requires for created selections,
 *     layers (`layer_from_selection`), and export areas (`export_area_from_selection`).
 *  2. Defaulting the target `id` to the active selection when callers (palette,
 *     keyboard shortcuts, menus) don't pass an explicit one.
 *
 * Point arrays for lasso/polygon are sent as `{x, y}` JSON objects, matching
 * `points_parameter` in the registry.
 */

/** Core `selection.*` command IDs the resolver understands. */
export const SELECTION_COMMAND_IDS = new Set([
  "selection.rect",
  "selection.ellipse",
  "selection.lasso",
  "selection.polygon",
  "selection.magic_wand",
  "selection.color_range",
  "selection.expand",
  "selection.contract",
  "selection.feather",
  "selection.invert",
  "selection.move",
  "selection.delete",
  "selection.copy",
  "selection.layer_from_selection",
  "selection.export_area_from_selection",
  "selection.direct_export",
]);

/** Selection commands that create a new selection mask. */
export const SELECTION_CREATE_IDS = new Set([
  "selection.rect",
  "selection.ellipse",
  "selection.lasso",
  "selection.polygon",
  "selection.magic_wand",
  "selection.color_range",
]);

let idCounter = 0;

export function newSelectionId(): string {
  idCounter += 1;
  return `selection-${Date.now().toString(36)}-${idCounter.toString(36)}`;
}

export function newSelectionChildId(prefix: "layer" | "area"): string {
  idCounter += 1;
  return `${prefix}-${Date.now().toString(36)}-${idCounter.toString(36)}`;
}

/** Pixel deltas for arrow-key nudging (Shift = larger nudge). */
export const SELECTION_NUDGE = 1;
export const SELECTION_NUDGE_LARGE = 10;

export type SelectionResolution = {
  parameters: Record<string, unknown>;
  /** ID of any selection created by this command, so the caller can focus it. */
  createdSelectionId: string | null;
  /** ID of any layer created (selection.layer_from_selection). */
  createdLayerId: string | null;
  /** ID of any export area created (selection.export_area_from_selection). */
  createdExportAreaId: string | null;
  /** True when the command deletes the selection mask. */
  removesSelection: boolean;
};

/**
 * Fill in the IDs a core `selection.*` invocation needs. Returns the parameter
 * object to send plus any IDs newly created so the store can update focus.
 */
export function resolveSelectionParams(
  commandId: string,
  parameters: Record<string, unknown>,
  activeSelectionId: string | null,
): SelectionResolution {
  const p: Record<string, unknown> = { ...parameters };
  let createdSelectionId: string | null = null;
  let createdLayerId: string | null = null;
  let createdExportAreaId: string | null = null;

  if (SELECTION_CREATE_IDS.has(commandId)) {
    if (p.id == null) p.id = newSelectionId();
    createdSelectionId = p.id as string;
  } else if (p.id == null && activeSelectionId) {
    p.id = activeSelectionId;
  }

  if (commandId === "selection.layer_from_selection") {
    if (p.layer_id == null) p.layer_id = newSelectionChildId("layer");
    if (p.name == null) p.name = "Selection layer";
    createdLayerId = p.layer_id as string;
  } else if (commandId === "selection.export_area_from_selection") {
    if (p.area_id == null) p.area_id = newSelectionChildId("area");
    if (p.name == null) p.name = "Selection area";
    createdExportAreaId = p.area_id as string;
  }

  return {
    parameters: p,
    createdSelectionId,
    createdLayerId,
    createdExportAreaId,
    removesSelection: commandId === "selection.delete",
  };
}
