/**
 * Frontend ⇆ core glue for image-object commands, mirroring `image-commands.ts`'s
 * sibling for layers. Maps UI intent onto `fleck-core::command`'s `image.*`
 * commands by generating the object/asset/layer IDs the core requires and
 * defaulting the target object to the current selection.
 *
 * This module is intentionally store-free so the command store can depend on it
 * without an import cycle. Higher-level native flows (open/paste/drop pickers)
 * live in `image-import.ts`.
 */
import type { ImageSourceState } from "./fleck-data";

/** Core `image.*` command IDs the resolver understands. */
export const IMAGE_COMMAND_IDS = new Set([
  "image.import_linked",
  "image.import_clipboard",
  "image.import_drag_drop",
  "image.place_asset",
  "image.duplicate_object",
  "image.replace_source",
  "image.rasterize_object",
]);

let idCounter = 0;

/** A reasonably unique object/layer ID for newly created image objects/layers. */
export function newImageId(prefix: "image" | "layer"): string {
  idCounter += 1;
  return `${prefix}-${Date.now().toString(36)}-${idCounter.toString(36)}`;
}

/** Derive a display name from a file path's basename. */
export function basename(path: string): string {
  const tail = path.split(/[\\/]/).pop() ?? path;
  return tail || "Image";
}

/**
 * Fill in the IDs and target an `image.*` invocation needs. Returns the params
 * to send plus the ID of any image object created, so the caller can select it.
 * (`rasterize` creates a layer, not an object, so it reports no created object.)
 */
export function resolveImageParams(
  commandId: string,
  parameters: Record<string, unknown>,
  selectedObjectId: string | null,
): { parameters: Record<string, unknown>; createdObjectId: string | null } {
  const p: Record<string, unknown> = { ...parameters };
  let createdObjectId: string | null = null;

  // Commands that act on an existing placed object default to the selection.
  const actsOnExisting =
    commandId === "image.duplicate_object" ||
    commandId === "image.replace_source" ||
    commandId === "image.rasterize_object";
  if (actsOnExisting && p.object_id == null && selectedObjectId) p.object_id = selectedObjectId;

  switch (commandId) {
    case "image.import_linked": {
      if (p.object_id == null) p.object_id = newImageId("image");
      if (p.asset_id == null) p.asset_id = newImageId("image");
      if (p.name == null && typeof p.path === "string") p.name = basename(p.path);
      if (p.name == null) p.name = "Image";
      createdObjectId = p.object_id as string;
      break;
    }
    case "image.import_clipboard":
    case "image.import_drag_drop":
    case "image.place_asset": {
      if (p.object_id == null) p.object_id = newImageId("image");
      if (p.name == null) p.name = "Image";
      createdObjectId = p.object_id as string;
      break;
    }
    case "image.duplicate_object": {
      if (p.new_object_id == null) p.new_object_id = newImageId("image");
      createdObjectId = p.new_object_id as string;
      break;
    }
    case "image.rasterize_object": {
      if (p.layer_id == null) p.layer_id = newImageId("layer");
      break;
    }
  }

  return { parameters: p, createdObjectId };
}

/** Inspector presentation for each source state (label only; colors live in the view). */
export const SOURCE_STATE_LABEL: Record<ImageSourceState, string> = {
  linked: "Linked",
  embedded: "Embedded",
  missing: "Missing",
  replaced: "Replaced",
};
