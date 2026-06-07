/**
 * Frontend ⇆ core glue for layer operations.
 *
 * The layers panel, inspector, and context menus all express edits as core
 * command invocations (mirroring `fleck-core::command`'s `layer.*` commands) so
 * every mutation is undoable and shows up in history. This module owns the two
 * frontend-only concerns those commands need:
 *
 *  1. Generating the stable object IDs the core requires for created objects
 *     (`layer.create`/`duplicate`/`group`/`flatten`).
 *  2. Defaulting the target layer ID to the current selection when a caller
 *     (e.g. the command palette) doesn't pass an explicit one.
 *
 * Opacity is converted at the call site: the `Layer` DTO carries 0–100 for the
 * UI, while `layer.set_opacity` takes the core's 0.0–1.0 scale.
 */
import type { BlendMode } from "./fleck-data";

/** Core `layer.*` command IDs the resolver understands. */
export const LAYER_COMMAND_IDS = new Set([
  "layer.create",
  "layer.duplicate",
  "layer.delete",
  "layer.rename",
  "layer.reorder",
  "layer.set_visible",
  "layer.set_locked",
  "layer.set_opacity",
  "layer.set_blend_mode",
  "layer.merge_down",
  "layer.flatten",
  "layer.group",
]);

let idCounter = 0;

/** A reasonably unique object ID for newly created layers/groups. */
export function newObjectId(prefix: "layer" | "group"): string {
  idCounter += 1;
  return `${prefix}-${Date.now().toString(36)}-${idCounter.toString(36)}`;
}

/**
 * Fill in the object IDs and target a core `layer.*` invocation needs. Returns
 * the parameter object to send, plus the ID of any object the command creates
 * so the caller can select it afterwards.
 */
export function resolveLayerParams(
  commandId: string,
  parameters: Record<string, unknown>,
  selectedId: string | null,
): { parameters: Record<string, unknown>; createdId: string | null } {
  const p: Record<string, unknown> = { ...parameters };
  let createdId: string | null = null;

  // Every command except create/flatten acts on an existing target layer.
  const needsTarget = commandId !== "layer.create" && commandId !== "layer.flatten";
  if (needsTarget && p.id == null && selectedId) p.id = selectedId;

  switch (commandId) {
    case "layer.create": {
      if (p.id == null) p.id = newObjectId("layer");
      if (p.name == null) p.name = "New layer";
      createdId = p.id as string;
      break;
    }
    case "layer.duplicate": {
      if (p.new_id == null) p.new_id = newObjectId("layer");
      createdId = p.new_id as string;
      break;
    }
    case "layer.group": {
      if (p.group_id == null) p.group_id = newObjectId("group");
      if (p.name == null) p.name = "Group";
      createdId = p.group_id as string;
      break;
    }
    case "layer.flatten": {
      if (p.flattened_id == null) p.flattened_id = newObjectId("layer");
      createdId = p.flattened_id as string;
      break;
    }
  }

  return { parameters: p, createdId };
}

/** Blend modes offered in the inspector, paired with their core param string. */
export const BLEND_MODES: { value: string; label: BlendMode }[] = [
  { value: "normal", label: "Normal" },
  { value: "multiply", label: "Multiply" },
  { value: "screen", label: "Screen" },
  { value: "overlay", label: "Overlay" },
  { value: "darken", label: "Darken" },
  { value: "lighten", label: "Lighten" },
  { value: "color_dodge", label: "ColorDodge" },
  { value: "color_burn", label: "ColorBurn" },
  { value: "hard_light", label: "HardLight" },
  { value: "soft_light", label: "SoftLight" },
  { value: "difference", label: "Difference" },
  { value: "exclusion", label: "Exclusion" },
  { value: "hue", label: "Hue" },
  { value: "saturation", label: "Saturation" },
  { value: "color", label: "Color" },
  { value: "luminosity", label: "Luminosity" },
];

/** Map a `Layer.blend` DTO label to the snake_case `layer.set_blend_mode` param. */
export function blendParam(blend: BlendMode): string {
  return BLEND_MODES.find((m) => m.label === blend)?.value ?? "normal";
}
