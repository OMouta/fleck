/**
 * Frontend ⇆ core glue for export-area and output commands, mirroring the
 * `layer-commands` / `image-commands` siblings. The exports panel, inspector,
 * canvas context menu, and command palette all express export edits as core
 * command invocations (`export_area.*` / `output.*`) so every mutation is
 * undoable and shows up in history.
 *
 * This module owns the two frontend-only concerns those commands need:
 *
 *  1. Generating the stable object IDs the core requires for created objects
 *     (`export_area.create`/`duplicate`, `output.add`/`duplicate`).
 *  2. Defaulting the target export area (and its primary output) to the current
 *     selection when a caller doesn't pass an explicit one.
 *
 * It is intentionally store-free so the command store can depend on it without
 * an import cycle.
 */

/** Core `export_area.*` / `output.*` command IDs the resolver understands. */
export const EXPORT_COMMAND_IDS = new Set([
  "export_area.create",
  "export_area.rename",
  "export_area.move",
  "export_area.resize",
  "export_area.set_padding",
  "export_area.set_background",
  "export_area.duplicate",
  "export_area.delete",
  "export_area.set_tags",
  "export_area.attach_output",
  "export_area.detach_output",
  "output.add",
  "output.remove",
  "output.duplicate",
  "output.update",
]);

let idCounter = 0;

/** A reasonably unique object ID for newly created export areas / outputs. */
export function newExportId(prefix: "export-area" | "output"): string {
  idCounter += 1;
  return `${prefix}-${Date.now().toString(36)}-${idCounter.toString(36)}`;
}

/** Default bounds for a fresh export area when the caller doesn't supply size. */
export const DEFAULT_EXPORT_AREA_SIZE = { width: 512, height: 512 } as const;

/**
 * Fill in the IDs and target an export command needs. Returns the parameters to
 * send plus the ID of any export area the command creates, so the caller can
 * select it afterwards. `selectedOutputId` is the area's primary output, used as
 * the default target for output-scoped commands invoked without an explicit one.
 */
export function resolveExportParams(
  commandId: string,
  parameters: Record<string, unknown>,
  selectedAreaId: string | null,
  selectedOutputId: string | null,
): { parameters: Record<string, unknown>; createdAreaId: string | null } {
  const p: Record<string, unknown> = { ...parameters };
  let createdAreaId: string | null = null;

  // Area-scoped commands (everything but create + the output.* family that takes
  // its own ids) act on an existing area; default it to the current selection.
  const areaScoped =
    commandId === "export_area.rename" ||
    commandId === "export_area.move" ||
    commandId === "export_area.resize" ||
    commandId === "export_area.set_padding" ||
    commandId === "export_area.set_background" ||
    commandId === "export_area.duplicate" ||
    commandId === "export_area.delete" ||
    commandId === "export_area.set_tags" ||
    commandId === "export_area.attach_output" ||
    commandId === "export_area.detach_output";
  if (areaScoped && p.id == null && p.area_id == null && selectedAreaId) {
    // create/rename/move/resize/duplicate/delete/set_tags key the area as `id`;
    // attach/detach key it as `area_id`.
    if (commandId === "export_area.attach_output" || commandId === "export_area.detach_output") {
      p.area_id = selectedAreaId;
    } else {
      p.id = selectedAreaId;
    }
  }

  // Output-scoped commands act on an existing output; default to the area's
  // primary output when the caller didn't name one.
  const outputScoped =
    commandId === "output.remove" || commandId === "output.duplicate" || commandId === "output.update";
  if (outputScoped && p.id == null && selectedOutputId) p.id = selectedOutputId;

  switch (commandId) {
    case "export_area.create": {
      if (p.id == null) p.id = newExportId("export-area");
      if (p.name == null) p.name = "Export area";
      if (p.width == null) p.width = DEFAULT_EXPORT_AREA_SIZE.width;
      if (p.height == null) p.height = DEFAULT_EXPORT_AREA_SIZE.height;
      createdAreaId = p.id as string;
      break;
    }
    case "export_area.duplicate": {
      if (p.new_id == null) p.new_id = newExportId("export-area");
      createdAreaId = p.new_id as string;
      break;
    }
    case "output.add": {
      if (p.id == null) p.id = newExportId("output");
      if (p.filename == null) p.filename = "export.png";
      if (p.format == null) p.format = "png";
      break;
    }
    case "output.duplicate": {
      if (p.new_id == null) p.new_id = newExportId("output");
      break;
    }
  }

  return { parameters: p, createdAreaId };
}

/** Output formats offered in the inspector, paired with their core param string. */
export const OUTPUT_FORMATS: { value: string; label: string; lossy: boolean }[] = [
  { value: "png", label: "PNG", lossy: false },
  { value: "jpeg", label: "JPEG", lossy: true },
  { value: "webp", label: "WebP", lossy: true },
  { value: "avif", label: "AVIF", lossy: true },
  { value: "gif", label: "GIF", lossy: false },
  { value: "bmp", label: "BMP", lossy: false },
  { value: "tiff", label: "TIFF", lossy: false },
  { value: "ico", label: "ICO", lossy: false },
];

/** Map a format label (as emitted by the backend DTO) to its core param string. */
export function formatParam(label: string): string {
  return OUTPUT_FORMATS.find((f) => f.label.toLowerCase() === label.toLowerCase())?.value ?? "png";
}

/** Whether a format label is lossy (and therefore carries a quality setting). */
export function isLossyFormat(label: string): boolean {
  return OUTPUT_FORMATS.find((f) => f.label.toLowerCase() === label.toLowerCase())?.lossy ?? false;
}

/** Export backgrounds offered in the inspector, paired with their core param string. */
export const EXPORT_BACKGROUNDS: { value: string; label: string }[] = [
  { value: "transparent", label: "Transparent" },
  { value: "white", label: "Solid white" },
  { value: "black", label: "Solid black" },
  { value: "checkerboard_preview", label: "Checkerboard" },
];

/** Scale presets offered in the inspector (core takes a float `scale`). */
export const SCALE_PRESETS: { value: number; label: string }[] = [
  { value: 0.5, label: "0.5×" },
  { value: 1, label: "1×" },
  { value: 2, label: "2×" },
  { value: 3, label: "3×" },
];
