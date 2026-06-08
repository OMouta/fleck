/**
 * Shared backend API wrapper.
 *
 * Every call into backend behaviour goes through this module. Today it falls back
 * to an in-memory mock that stands in for the Rust-owned document, but the shape
 * is final: the UI never reads or mutates document state directly — it asks the
 * backend. When the Tauri command bridge lands, `bridge()` starts forwarding to
 * `invoke()` and the mock fallback drops away without touching any component.
 *
 * See `docs/architecture.md`: Rust owns document truth, React owns UI.
 */
import type {
  CommandDefinition,
  CommandExecution,
  ExportArea,
  ExportResult,
  HistoryEntry,
  HistoryState,
  ImageObject,
  ImageSourceState,
  Layer,
  OpenWorkspaceResult,
  Output,
  Point,
  RecentFile,
  Rect,
  RenderModel,
  Size,
  ViewportFocusKind,
  WorkspaceMeta,
} from "./fleck-data";
import { COMMAND_DEFINITIONS } from "./command-registry";
import { BLEND_MODES } from "./layer-commands";
import { fitRect } from "./viewport";

/** Returns the Tauri `invoke` if running inside the desktop shell, else null. */
function getInvoke(): ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null {
  const tauri = (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  if (!tauri) return null;
  // Lazy require keeps the browser/dev build from importing the desktop API.
  return (cmd, args) =>
    import("@tauri-apps/api/core").then((m) => m.invoke(cmd, args));
}

/**
 * Single entry point for backend commands. Forwards to the Rust core through
 * Tauri when available; otherwise resolves with the provided mock value so the
 * frontend remains fully functional in a plain browser dev session.
 */
async function bridge<T>(command: string, args: Record<string, unknown>, mock: () => T | Promise<T>): Promise<T> {
  const invoke = getInvoke();
  if (invoke) {
    return invoke(command, args) as Promise<T>;
  }
  // Simulate async backend latency so loading/optimistic paths are exercised.
  await new Promise((r) => setTimeout(r, 60));
  return mock();
}

// --- Mock document (stands in for Rust-owned authoritative state) -------------

/** Mock asset row (stands in for `fleck-core::model::Asset` + link resolution). */
type MockAsset = {
  id: string;
  name: string;
  source: "linked" | "embedded";
  path: string | null;
  format: string | null;
  width: number;
  height: number;
  /** A linked asset whose file could not be resolved. */
  missing: boolean;
};

/** Mock placed image object (stands in for `fleck-core::model::ImageObject`). */
type MockImageObject = {
  id: string;
  name: string;
  sourceAssetId: string;
  position: { x: number; y: number };
  scale: { width: number; height: number };
  rotationDegrees: number;
  /** 0–100 for the UI (core stores 0.0–1.0). */
  opacity: number;
  crop: { x: number; y: number; width: number; height: number } | null;
  rasterizedLayerId: string | null;
  /** Set once `image.replace_source` swapped this object's source asset. */
  replaced: boolean;
};

/** Core background param string (mirrors the strings `command.rs` parses). */
type MockBackground = "transparent" | "white" | "black" | "checkerboard_preview";

/** Mock output definition (stands in for `fleck-core::model::OutputDefinition`). */
type MockOutput = {
  id: string;
  filename: string;
  folder: string | null;
  /** Core format param string, e.g. "png", "jpeg", "webp". */
  format: string;
  width: number | null;
  height: number | null;
  scale: number;
  quality: number | null;
  background: MockBackground;
  transparency: "preserve" | "flatten";
  metadata: "preserve" | "strip";
};

/** Mock export area (stands in for `fleck-core::model::ExportArea` — metadata only). */
type MockExportArea = {
  id: string;
  name: string;
  bounds: { x: number; y: number; width: number; height: number };
  padding: { top: number; right: number; bottom: number; left: number };
  background: MockBackground;
  outputIds: string[];
  includedLayerIds: string[];
  excludedLayerIds: string[];
  tags: string[];
};

const mockDoc: {
  meta: WorkspaceMeta;
  /** Canvas dimensions in workspace pixels (0 = no document loaded yet). */
  canvas: { width: number; height: number };
  layers: Layer[];
  assets: MockAsset[];
  imageObjects: MockImageObject[];
  exportAreas: MockExportArea[];
  outputs: MockOutput[];
  /** Undo stack + cursor, mirroring `CommandEngine` (undoable commands only). */
  history: { entries: HistoryEntry[]; currentIndex: number | null };
} = {
  // Fresh, untitled workspace — the shell opens empty until a real document loads.
  history: { entries: [], currentIndex: null },
  canvas: { width: 0, height: 0 },
  meta: {
    name: "Untitled.fleck",
    dirty: false,
    layerCount: 0,
    selectedCount: 0,
    canvasSize: "0 × 0 px",
  },
  layers: [],
  assets: [],
  imageObjects: [],
  exportAreas: [],
  outputs: [],
};

/**
 * Synthesize a representative render model from the mock canvas size. A real
 * backend would composite this from actual layers/areas; here it gives the host
 * something coherent to draw and navigate once a workspace is loaded.
 */
function buildRenderModel(): RenderModel {
  const { width, height } = mockDoc.canvas;
  if (width <= 0 || height <= 0) {
    return { canvas: { width: 0, height: 0 }, layers: [], exportAreas: [], guides: [], selections: [] };
  }
  const inset: Rect = { x: width * 0.12, y: height * 0.16, width: width * 0.45, height: height * 0.5 };
  return {
    canvas: { width, height },
    layers: [
      { id: "rl-base", rect: { x: 0, y: 0, width, height }, color: "#2b3b55", opacity: 1, visible: true },
      { id: "rl-art", rect: inset, color: "#3a86ff", opacity: 0.9, visible: true },
    ],
    // Export areas are document metadata, so the canvas draws exactly what the
    // exports panel lists — keeping selection/highlight in sync across both.
    exportAreas: mockDoc.exportAreas.map((area) => ({ id: area.id, name: area.name, rect: { ...area.bounds } })),
    guides: [
      { axis: "vertical", position: width / 2 },
      { axis: "horizontal", position: height / 2 },
    ],
    selections: [{ id: "sel-1", rect: inset }],
  };
}

let historyCounter = 0;

/** Push an undoable operation onto the mock undo stack (truncating any redo tail). */
function pushHistory(commandId: string, label: string) {
  const cut = mockDoc.history.currentIndex === null ? 0 : mockDoc.history.currentIndex + 1;
  mockDoc.history.entries = mockDoc.history.entries.slice(0, cut);
  mockDoc.history.entries.push({ id: `history-${historyCounter++}`, commandId, label });
  mockDoc.history.currentIndex = mockDoc.history.entries.length - 1;
  mockDoc.meta = { ...mockDoc.meta, dirty: true };
}

// --- Mock layer operations ---------------------------------------------------
// Apply `layer.*` core commands to the mock document so the layers panel,
// inspector, and history behave end-to-end in a browser dev session. The real
// backend performs these in the Rust core; the resolved parameter shapes here
// match `fleck-core::command`'s `layer.*` commands exactly.

/** Human labels for layer history entries (mirrors core `CommandEffect` labels). */
const LAYER_OP_LABELS: Record<string, string> = {
  "layer.create": "Add Layer",
  "layer.duplicate": "Duplicate Layer",
  "layer.delete": "Delete Layer",
  "layer.rename": "Rename Layer",
  "layer.reorder": "Reorder Layer",
  "layer.set_visible": "Set Layer Visibility",
  "layer.set_locked": "Set Layer Lock",
  "layer.set_opacity": "Set Layer Opacity",
  "layer.set_blend_mode": "Set Layer Blend Mode",
  "layer.merge_down": "Merge Layer Down",
  "layer.flatten": "Flatten Visible Layers",
  "layer.group": "Create Layer Group",
};

const clamp01 = (n: number) => (Number.isFinite(n) ? Math.min(1, Math.max(0, n)) : 0);

/** Map a snake_case `layer.set_blend_mode` param back to a `Layer.blend` label. */
function blendLabel(value: string): Layer["blend"] {
  return BLEND_MODES.find((m) => m.value === value)?.label ?? "Normal";
}

function newMockLayer(id: string, name: string, kind: Layer["kind"]): Layer {
  return { id, name, kind, visible: true, locked: false, opacity: 100, blend: "Normal" };
}

/**
 * Mutate `mockDoc.layers` for a resolved `layer.*` command. Returns whether the
 * document actually changed, so no-ops (missing target, locked guard) don't push
 * a history entry — matching the core, which rejects those before recording.
 */
function applyLayerMutation(commandId: string, p: Record<string, unknown>): boolean {
  const layers = mockDoc.layers;
  const indexOf = (id: unknown) => layers.findIndex((l) => l.id === id);

  switch (commandId) {
    case "layer.create": {
      layers.unshift(newMockLayer(String(p.id), String(p.name ?? "New layer"), "image"));
      return true;
    }
    case "layer.duplicate": {
      const i = indexOf(p.id);
      if (i === -1) return false;
      layers.splice(i, 0, { ...layers[i], id: String(p.new_id), name: `${layers[i].name} copy` });
      return true;
    }
    case "layer.delete": {
      const i = indexOf(p.id);
      if (i === -1 || layers[i].locked) return false;
      layers.splice(i, 1);
      return true;
    }
    case "layer.rename": {
      const l = layers[indexOf(p.id)];
      if (!l || l.locked) return false;
      l.name = String(p.name ?? l.name);
      return true;
    }
    case "layer.reorder": {
      const i = indexOf(p.id);
      if (i === -1 || layers[i].locked) return false;
      const to = Math.max(0, Math.min(layers.length - 1, Math.trunc(Number(p.index))));
      if (to === i) return false;
      const [moved] = layers.splice(i, 1);
      layers.splice(to, 0, moved);
      return true;
    }
    case "layer.set_visible": {
      const l = layers[indexOf(p.id)];
      if (!l) return false;
      l.visible = Boolean(p.visible);
      return true;
    }
    case "layer.set_locked": {
      const l = layers[indexOf(p.id)];
      if (!l) return false;
      l.locked = Boolean(p.locked);
      return true;
    }
    case "layer.set_opacity": {
      const l = layers[indexOf(p.id)];
      if (!l || l.locked) return false;
      l.opacity = Math.round(clamp01(Number(p.opacity)) * 100);
      return true;
    }
    case "layer.set_blend_mode": {
      const l = layers[indexOf(p.id)];
      if (!l || l.locked) return false;
      l.blend = blendLabel(String(p.blend_mode));
      return true;
    }
    case "layer.merge_down": {
      const i = indexOf(p.id);
      // Needs a layer below to merge into; the source row collapses away.
      if (i === -1 || i >= layers.length - 1 || layers[i].locked) return false;
      layers.splice(i, 1);
      return true;
    }
    case "layer.flatten": {
      const kept = layers.filter((l) => !l.visible);
      if (kept.length === layers.length) return false; // nothing visible to flatten
      mockDoc.layers = [newMockLayer(String(p.flattened_id), "Flattened", "image"), ...kept];
      return true;
    }
    case "layer.group": {
      const i = indexOf(p.id);
      if (i === -1) return false;
      // The flat layer DTO can't express nesting yet, so the group shows as a
      // header row above its source. Hierarchical rendering is deferred — see
      // `.plan/decisions.md` (DEC-FE-005-group-nesting).
      layers.splice(i, 0, newMockLayer(String(p.group_id), String(p.name ?? "Group"), "group"));
      return true;
    }
    default:
      return false;
  }
}

// --- Mock image-object operations --------------------------------------------
// Apply `image.*` core commands to the mock document and project placed image
// objects (joined with their asset) into the `ImageObject` DTO the UI reads.

/** Human labels for image history entries (mirrors core `CommandEffect` labels). */
const IMAGE_OP_LABELS: Record<string, string> = {
  "image.import_linked": "Import Image",
  "image.import_clipboard": "Import Clipboard Image",
  "image.import_drag_drop": "Import Dropped Image",
  "image.place_asset": "Place Image Asset",
  "image.duplicate_object": "Duplicate Image Object",
  "image.replace_source": "Replace Image Source",
  "image.rasterize_object": "Rasterize Image Object",
};

let mockAssetCounter = 0;
function mockAssetId(): string {
  mockAssetCounter += 1;
  return `asset-${Date.now().toString(36)}-${mockAssetCounter.toString(36)}`;
}

/** Join a placed object with its asset into the read DTO + resolved source state. */
function projectImageObject(o: MockImageObject): ImageObject {
  const asset = mockDoc.assets.find((a) => a.id === o.sourceAssetId);
  const sourceState: ImageSourceState = o.replaced
    ? "replaced"
    : !asset || asset.missing
      ? "missing"
      : asset.source === "linked"
        ? "linked"
        : "embedded";
  return {
    id: o.id,
    name: o.name,
    sourceAssetId: o.sourceAssetId,
    sourceState,
    sourceName: asset?.name ?? "(missing asset)",
    sourcePath: asset?.path ?? null,
    format: asset?.format ?? null,
    dimensions: asset ? `${asset.width} × ${asset.height} px` : null,
    position: { ...o.position },
    scale: { ...o.scale },
    rotationDegrees: o.rotationDegrees,
    opacity: o.opacity,
    crop: o.crop ? { ...o.crop } : null,
    rasterizedLayerId: o.rasterizedLayerId,
  };
}

/** Create a placed object from an existing asset using sensible default placement. */
function placeMockObject(objectId: string, assetId: string, name: string): MockImageObject {
  const asset = mockDoc.assets.find((a) => a.id === assetId);
  return {
    id: objectId,
    name,
    sourceAssetId: assetId,
    position: { x: 0, y: 0 },
    scale: { width: asset?.width ?? 256, height: asset?.height ?? 256 },
    rotationDegrees: 0,
    opacity: 100,
    crop: null,
    rasterizedLayerId: null,
    replaced: false,
  };
}

/**
 * Mutate `mockDoc` for a resolved `image.*` command. Returns whether the document
 * changed, so no-ops (missing target/asset) don't record a history entry.
 */
function applyImageMutation(commandId: string, p: Record<string, unknown>): boolean {
  const objects = mockDoc.imageObjects;
  const findObject = (id: unknown) => objects.find((o) => o.id === id);

  switch (commandId) {
    case "image.import_linked": {
      const path = typeof p.path === "string" ? p.path : null;
      mockDoc.assets.push({
        id: String(p.asset_id),
        name: String(p.name ?? "image"),
        source: "linked",
        path,
        format: formatFromPath(path),
        width: 1024,
        height: 1024,
        missing: false,
      });
      objects.push(placeMockObject(String(p.object_id), String(p.asset_id), String(p.name ?? "Image")));
      return true;
    }
    case "image.import_clipboard":
    case "image.import_drag_drop":
    case "image.place_asset": {
      if (!mockDoc.assets.some((a) => a.id === p.asset_id)) return false;
      objects.push(placeMockObject(String(p.object_id), String(p.asset_id), String(p.name ?? "Image")));
      return true;
    }
    case "image.duplicate_object": {
      const src = findObject(p.object_id);
      if (!src) return false;
      objects.push({ ...src, id: String(p.new_object_id), name: `${src.name} Copy` });
      return true;
    }
    case "image.replace_source": {
      const obj = findObject(p.object_id);
      if (!obj || !mockDoc.assets.some((a) => a.id === p.asset_id)) return false;
      obj.sourceAssetId = String(p.asset_id);
      obj.replaced = true;
      obj.rasterizedLayerId = null;
      return true;
    }
    case "image.rasterize_object": {
      const obj = findObject(p.object_id);
      if (!obj) return false;
      mockDoc.layers.unshift({
        id: String(p.layer_id),
        name: obj.name,
        kind: "image",
        visible: true,
        locked: false,
        opacity: 100,
        blend: "Normal",
      });
      obj.rasterizedLayerId = String(p.layer_id);
      return true;
    }
    default:
      return false;
  }
}

/** Guess an uppercase format label from a file extension. */
function formatFromPath(path: string | null): string | null {
  const ext = path?.split(".").pop()?.toLowerCase();
  if (!ext) return null;
  if (ext === "jpg" || ext === "jpeg") return "JPEG";
  return ext.toUpperCase();
}

// --- Mock export-area / output operations ------------------------------------
// Apply `export_area.*` and `output.*` core commands to the mock document and
// project export areas (joined with their outputs + computed preview metadata)
// into the `ExportArea` DTO the UI reads. The projection mirrors
// `fleck-core::export::preview_export_area`: padded pixel bounds, per-output
// preview dimensions, and the warning set all come from here so the UI consumes
// preview metadata rather than recomputing it.

/** Human labels for export history entries (mirrors core `CommandEffect` labels). */
const EXPORT_OP_LABELS: Record<string, string> = {
  "export_area.create": "Create Export Area",
  "export_area.rename": "Rename Export Area",
  "export_area.move": "Move Export Area",
  "export_area.resize": "Resize Export Area",
  "export_area.duplicate": "Duplicate Export Area",
  "export_area.delete": "Delete Export Area",
  "export_area.set_tags": "Set Export Area Tags",
  "export_area.attach_output": "Attach Output To Export Area",
  "export_area.detach_output": "Detach Output From Export Area",
  "output.add": "Add Output",
  "output.remove": "Remove Output",
  "output.duplicate": "Duplicate Output",
  "output.update": "Update Output",
};

const FORMAT_LABELS: Record<string, string> = {
  png: "PNG",
  jpeg: "JPEG",
  jpg: "JPEG",
  webp: "WebP",
  avif: "AVIF",
  gif: "GIF",
  bmp: "BMP",
  tiff: "TIFF",
  ico: "ICO",
  icns: "ICNS",
  svg_rasterized: "SVG",
  pdf: "PDF",
};

function formatLabel(format: string): string {
  return FORMAT_LABELS[format.toLowerCase()] ?? format.toUpperCase();
}

function backgroundLabel(background: MockBackground): string {
  switch (background) {
    case "transparent":
      return "Transparent";
    case "white":
      return "Solid #ffffff";
    case "black":
      return "Solid #000000";
    case "checkerboard_preview":
      return "Checkerboard";
  }
}

/** Summarize per-side padding the way the inspector shows it. */
function paddingLabel(p: MockExportArea["padding"]): string {
  if (p.top === 0 && p.right === 0 && p.bottom === 0 && p.left === 0) return "None";
  if (p.top === p.right && p.right === p.bottom && p.bottom === p.left) return `${round(p.top)} px`;
  return `T${round(p.top)} R${round(p.right)} B${round(p.bottom)} L${round(p.left)}`;
}

function round(n: number): number {
  return Math.round(n * 10) / 10;
}

/** Format a float scale as "1×", "2×", "0.5×". */
function scaleLabel(scale: number): string {
  return `${Number(scale.toFixed(3))}×`;
}

/** Padded export pixel bounds (mirrors `export::padded_pixel_rect`). */
function paddedPixels(area: MockExportArea): { width: number; height: number } {
  return {
    width: Math.max(1, Math.ceil(area.bounds.width + area.padding.left + area.padding.right)),
    height: Math.max(1, Math.ceil(area.bounds.height + area.padding.top + area.padding.bottom)),
  };
}

/** Layers that participate in an area (mirrors `export::participating_layers`). */
function participatingLayerIds(area: MockExportArea): string[] {
  return mockDoc.layers
    .filter((layer) => layer.visible && layer.opacity > 0)
    .filter((layer) => !area.excludedLayerIds.includes(layer.id))
    .filter((layer) => area.includedLayerIds.length === 0 || area.includedLayerIds.includes(layer.id))
    .map((layer) => layer.id);
}

/** Compute the human warning set from preview metadata (mirrors `ExportWarning`). */
function exportWarnings(area: MockExportArea, outputs: MockOutput[]): string[] {
  const warnings: string[] = [];
  if (area.includedLayerIds.some((id) => area.excludedLayerIds.includes(id))) {
    warnings.push("A layer is both included and excluded");
  }
  if (outputs.length === 0) warnings.push("No outputs configured");
  if (participatingLayerIds(area).length === 0) warnings.push("No layers participate in this export");
  for (const output of outputs) {
    if (output.format.toLowerCase() === "jpeg" && output.transparency === "preserve") {
      warnings.push(`${output.filename}: JPEG cannot preserve transparency`);
    }
    if (output.background === "checkerboard_preview") {
      warnings.push(`${output.filename}: checkerboard is a preview-only background`);
    }
  }
  return warnings;
}

/** Pixel dimensions an output renders to (mirrors `export::output_preview`). */
function outputPixels(output: MockOutput, padded: { width: number; height: number }): { width: number; height: number } {
  return {
    width: output.width ?? Math.max(1, Math.round(padded.width * output.scale)),
    height: output.height ?? Math.max(1, Math.round(padded.height * output.scale)),
  };
}

/**
 * Rough encoded-size estimate for the export preview (REQ-032). The real
 * pipeline reports exact bytes after encoding; this heuristic just gives the
 * preview a representative number per format before anything is written.
 */
function estimateBytes(width: number, height: number, format: string, quality: number | null): number {
  const pixels = width * height;
  switch (format.toLowerCase()) {
    case "jpeg":
      return Math.round(pixels * 0.42 * ((quality ?? 80) / 100));
    case "webp":
      return Math.round(pixels * 0.32 * ((quality ?? 80) / 100));
    case "avif":
      return Math.round(pixels * 0.22 * ((quality ?? 75) / 100));
    case "gif":
      return Math.round(pixels * 0.8);
    case "png":
      return Math.round(pixels * 1.9);
    default:
      return Math.round(pixels * 1.4);
  }
}

/** Human-readable byte size, e.g. "812 B", "12.4 KB", "1.3 MB". */
function humanBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Project a mock output + the area's padded bounds into the read DTO. */
function projectOutput(output: MockOutput, padded: { width: number; height: number }): Output {
  const { width, height } = outputPixels(output, padded);
  return {
    id: output.id,
    filename: output.filename,
    format: formatLabel(output.format),
    scale: scaleLabel(output.scale),
    quality: output.quality,
    transparency: output.transparency === "flatten" ? "Flatten" : "Preserve",
    destination: output.folder ? `${output.folder}/${output.filename}` : null,
    dimensions: `${width} × ${height} px`,
    estimatedSize: `~${humanBytes(estimateBytes(width, height, output.format, output.quality))}`,
  };
}

/** Run an export job over a set of mock areas, producing a representative result. */
function runMockExport(scope: string, areas: MockExportArea[]): ExportResult {
  const outputs: ExportResult["outputs"] = [];
  const warnings = new Set<string>();
  for (const area of areas) {
    const padded = paddedPixels(area);
    const areaOutputs = area.outputIds
      .map((id) => mockDoc.outputs.find((o) => o.id === id))
      .filter((o): o is MockOutput => o !== undefined);
    for (const warning of exportWarnings(area, areaOutputs)) warnings.add(warning);
    for (const output of areaOutputs) {
      const { width, height } = outputPixels(output, padded);
      outputs.push({
        id: output.id,
        filename: output.filename,
        destination: output.folder ? `${output.folder}/${output.filename}` : null,
        format: formatLabel(output.format),
        dimensions: `${width} × ${height} px`,
        // The mock can't encode real pixels, so it reports the estimate as the
        // produced size; the desktop pipeline returns exact bytes here.
        size: humanBytes(estimateBytes(width, height, output.format, output.quality)),
        dataUrl: null,
      });
    }
  }
  return { scope, outputs, warnings: [...warnings], failures: [] };
}

/** Project a mock export area (joined with outputs + preview metadata) into the DTO. */
function projectExportArea(area: MockExportArea): ExportArea {
  const outputs = area.outputIds
    .map((id) => mockDoc.outputs.find((o) => o.id === id))
    .filter((o): o is MockOutput => o !== undefined);
  const padded = paddedPixels(area);
  const warnings = exportWarnings(area, outputs);
  return {
    id: area.id,
    name: area.name,
    dimensions: `${round(area.bounds.width)} × ${round(area.bounds.height)} px`,
    position: `${round(area.bounds.x)}, ${round(area.bounds.y)}`,
    padding: paddingLabel(area.padding),
    background: backgroundLabel(area.background),
    format: outputs[0] ? formatLabel(outputs[0].format) : "—",
    status: warnings.length > 0 ? "warning" : "ready",
    warnings,
    outputs: outputs.map((o) => projectOutput(o, padded)),
  };
}

const num = (v: unknown, fallback = 0): number => (Number.isFinite(Number(v)) ? Number(v) : fallback);

/** Map a background command string to the mock enum (defaults to transparent). */
function backgroundParam(value: unknown): MockBackground {
  return value === "white" || value === "black" || value === "checkerboard_preview" ? value : "transparent";
}

/**
 * Mutate `mockDoc` for a resolved `export_area.*` / `output.*` command. Returns
 * whether the document changed, so no-ops don't record a history entry.
 */
function applyExportMutation(commandId: string, p: Record<string, unknown>): boolean {
  const areas = mockDoc.exportAreas;
  const findArea = (id: unknown) => areas.find((a) => a.id === id);
  const findOutput = (id: unknown) => mockDoc.outputs.find((o) => o.id === id);

  switch (commandId) {
    case "export_area.create": {
      const width = num(p.width);
      const height = num(p.height);
      if (width <= 0 || height <= 0) return false;
      areas.push({
        id: String(p.id),
        name: String(p.name ?? "Export area"),
        bounds: { x: num(p.x), y: num(p.y), width, height },
        padding: { top: 0, right: 0, bottom: 0, left: 0 },
        background: "transparent",
        outputIds: [],
        includedLayerIds: [],
        excludedLayerIds: [],
        tags: [],
      });
      return true;
    }
    case "export_area.rename": {
      const area = findArea(p.id);
      if (!area) return false;
      area.name = String(p.name ?? area.name);
      return true;
    }
    case "export_area.move": {
      const area = findArea(p.id);
      if (!area) return false;
      area.bounds.x = num(p.x, area.bounds.x);
      area.bounds.y = num(p.y, area.bounds.y);
      return true;
    }
    case "export_area.resize": {
      const area = findArea(p.id);
      const width = num(p.width);
      const height = num(p.height);
      if (!area || width <= 0 || height <= 0) return false;
      area.bounds.width = width;
      area.bounds.height = height;
      return true;
    }
    case "export_area.duplicate": {
      const src = findArea(p.id);
      if (!src) return false;
      areas.push({
        ...src,
        id: String(p.new_id),
        name: `${src.name} Copy`,
        bounds: { ...src.bounds },
        padding: { ...src.padding },
        outputIds: [...src.outputIds],
        includedLayerIds: [...src.includedLayerIds],
        excludedLayerIds: [...src.excludedLayerIds],
        tags: [...src.tags],
      });
      return true;
    }
    case "export_area.delete": {
      const i = areas.findIndex((a) => a.id === p.id);
      if (i === -1) return false;
      areas.splice(i, 1);
      return true;
    }
    case "export_area.set_tags": {
      const area = findArea(p.id);
      if (!area) return false;
      const raw = Array.isArray(p.tags) ? (p.tags as unknown[]).map(String) : String(p.tags ?? "").split(",");
      area.tags = raw.map((t) => t.trim()).filter((t, i, arr) => t && arr.indexOf(t) === i);
      return true;
    }
    case "export_area.attach_output": {
      const area = findArea(p.area_id);
      if (!area || !findOutput(p.output_id)) return false;
      if (!area.outputIds.includes(String(p.output_id))) area.outputIds.push(String(p.output_id));
      return true;
    }
    case "export_area.detach_output": {
      const area = findArea(p.area_id);
      if (!area) return false;
      area.outputIds = area.outputIds.filter((id) => id !== p.output_id);
      return true;
    }
    case "output.add": {
      const filename = String(p.filename ?? "").trim();
      if (!filename) return false;
      mockDoc.outputs.push({
        id: String(p.id),
        filename,
        folder: typeof p.folder === "string" && p.folder ? p.folder : null,
        format: String(p.format ?? "png"),
        width: p.width == null ? null : Math.trunc(num(p.width)),
        height: p.height == null ? null : Math.trunc(num(p.height)),
        scale: p.scale == null ? 1 : num(p.scale, 1),
        quality: p.quality == null ? null : Math.trunc(num(p.quality)),
        background: backgroundParam(p.background),
        transparency: p.transparency === "flatten" ? "flatten" : "preserve",
        metadata: p.metadata === "preserve" ? "preserve" : "strip",
      });
      return true;
    }
    case "output.remove": {
      const i = mockDoc.outputs.findIndex((o) => o.id === p.id);
      if (i === -1) return false;
      mockDoc.outputs.splice(i, 1);
      for (const area of areas) area.outputIds = area.outputIds.filter((id) => id !== p.id);
      return true;
    }
    case "output.duplicate": {
      const src = findOutput(p.id);
      if (!src) return false;
      const [stem, ext] = src.filename.includes(".")
        ? [src.filename.slice(0, src.filename.lastIndexOf(".")), src.filename.slice(src.filename.lastIndexOf("."))]
        : [src.filename, ""];
      mockDoc.outputs.push({ ...src, id: String(p.new_id), filename: `${stem}-copy${ext}` });
      return true;
    }
    case "output.update": {
      const output = findOutput(p.id);
      if (!output) return false;
      if (p.filename != null) output.filename = String(p.filename);
      if (p.format != null) output.format = String(p.format);
      if (p.scale != null) output.scale = num(p.scale, output.scale);
      if (p.quality !== undefined) output.quality = p.quality == null ? null : Math.trunc(num(p.quality));
      if (p.background != null) output.background = backgroundParam(p.background);
      if (p.transparency != null) output.transparency = p.transparency === "flatten" ? "flatten" : "preserve";
      if (p.metadata != null) output.metadata = p.metadata === "preserve" ? "preserve" : "strip";
      return true;
    }
    default:
      return false;
  }
}

/** Seed a representative set of export areas + outputs for a loaded mock workspace. */
function seedMockExports(width: number, height: number) {
  mockDoc.outputs = [
    { id: "out-frame-png", filename: "frame.png", folder: null, format: "png", width: null, height: null, scale: 1, quality: null, background: "transparent", transparency: "preserve", metadata: "strip" },
    { id: "out-frame-jpg", filename: "frame.jpg", folder: "social", format: "jpeg", width: null, height: null, scale: 1, quality: 82, background: "transparent", transparency: "preserve", metadata: "strip" },
    { id: "out-icon-1x", filename: "icon.png", folder: "icons", format: "png", width: null, height: null, scale: 1, quality: null, background: "transparent", transparency: "preserve", metadata: "strip" },
    { id: "out-icon-2x", filename: "icon@2x.png", folder: "icons", format: "png", width: null, height: null, scale: 2, quality: null, background: "transparent", transparency: "preserve", metadata: "strip" },
  ];
  mockDoc.exportAreas = [
    {
      id: "ea-frame",
      name: "frame",
      bounds: { x: 0, y: 0, width, height },
      padding: { top: 0, right: 0, bottom: 0, left: 0 },
      background: "transparent",
      outputIds: ["out-frame-png", "out-frame-jpg"],
      includedLayerIds: [],
      excludedLayerIds: [],
      tags: ["marketing"],
    },
    {
      id: "ea-icon",
      name: "icon",
      bounds: { x: Math.round(width * 0.66), y: Math.round(height * 0.1), width: 256, height: 256 },
      padding: { top: 8, right: 8, bottom: 8, left: 8 },
      background: "transparent",
      outputIds: ["out-icon-1x", "out-icon-2x"],
      includedLayerIds: [],
      excludedLayerIds: [],
      tags: ["icons"],
    },
  ];
}

// --- Queries (read document state) -------------------------------------------

export const api = {
  getWorkspaceMeta(): Promise<WorkspaceMeta> {
    return bridge("get_workspace_meta", {}, () => ({ ...mockDoc.meta }));
  },

  getLayers(): Promise<Layer[]> {
    return bridge("get_layers", {}, () => mockDoc.layers.map((l) => ({ ...l })));
  },

  getImageObjects(): Promise<ImageObject[]> {
    return bridge("get_image_objects", {}, () => mockDoc.imageObjects.map(projectImageObject));
  },

  getExportAreas(): Promise<ExportArea[]> {
    return bridge("get_export_areas", {}, () => mockDoc.exportAreas.map(projectExportArea));
  },

  getHistory(): Promise<HistoryState> {
    return bridge("get_history", {}, () => ({
      entries: mockDoc.history.entries.map((h) => ({ ...h })),
      currentIndex: mockDoc.history.currentIndex,
    }));
  },

  /** The command registry definitions (mirrors `CommandRegistry::definitions`). */
  getCommands(): Promise<CommandDefinition[]> {
    return bridge("get_commands", {}, () => COMMAND_DEFINITIONS.map((c) => ({ ...c })));
  },

  // --- Mutations (request document changes) ----------------------------------
  // Layer edits run through the command engine (`runCommand` below) so they are
  // undoable and recorded in history, exactly like the Rust core. There are no
  // direct layer setters here by design.

  // --- Workspace file operations ---------------------------------------------
  // These commands open native dialogs and read/write `.fleck` files in the Rust
  // core. The UI never parses or mutates workspace files itself — it only invokes
  // these commands and renders the structured results. (Native dialog + recent-
  // file persistence wiring on the Rust side is TASK-020; the mock below stands
  // in until then.)

  /**
   * Opens a native file picker, loads the chosen `.fleck` via the core, and
   * returns load warnings + unresolved linked assets. Resolves to null if the
   * user cancels the picker.
   */
  openWorkspace(): Promise<OpenWorkspaceResult | null> {
    return bridge("open_workspace", {}, () => {
      // Representative file that exercises both the version-warning and
      // missing-linked-asset paths so those dialogs are demonstrable.
      mockDoc.canvas = { width: 1200, height: 630 };
      mockDoc.layers = [
        { id: "layer-badge", name: "Badge", kind: "shape", visible: true, locked: false, opacity: 100, blend: "Normal" },
        { id: "layer-art", name: "Artwork", kind: "image", visible: true, locked: false, opacity: 90, blend: "Normal" },
        { id: "layer-bg", name: "Background", kind: "image", visible: true, locked: true, opacity: 100, blend: "Normal" },
      ];
      // Placed image objects spanning the resolvable states so the Images panel
      // can demonstrate linked / embedded / missing distinctly.
      mockDoc.assets = [
        { id: "asset-tex", name: "texture.jpg", source: "linked", path: "C:/work/linked/texture.jpg", format: "JPEG", width: 2048, height: 2048, missing: false },
        { id: "asset-badge", name: "badge.png", source: "embedded", path: null, format: "PNG", width: 512, height: 512, missing: false },
        { id: "asset-hero", name: "hero-render.png", source: "linked", path: "C:/work/linked/hero-render.png", format: "PNG", width: 1600, height: 900, missing: true },
      ];
      mockDoc.imageObjects = [
        { id: "img-tex", name: "Texture", sourceAssetId: "asset-tex", position: { x: 0, y: 0 }, scale: { width: 600, height: 600 }, rotationDegrees: 12, opacity: 60, crop: { x: 0, y: 0, width: 1024, height: 1024 }, rasterizedLayerId: null, replaced: false },
        { id: "img-badge", name: "Badge", sourceAssetId: "asset-badge", position: { x: 840, y: 40 }, scale: { width: 160, height: 160 }, rotationDegrees: 0, opacity: 90, crop: null, rasterizedLayerId: null, replaced: false },
        { id: "img-hero", name: "Hero", sourceAssetId: "asset-hero", position: { x: 0, y: 0 }, scale: { width: 1200, height: 675 }, rotationDegrees: 0, opacity: 100, crop: null, rasterizedLayerId: null, replaced: false },
      ];
      seedMockExports(1200, 630);
      mockDoc.meta = {
        name: "marketing-assets.fleck",
        dirty: false,
        layerCount: mockDoc.layers.length,
        selectedCount: 1,
        canvasSize: "1200 × 630 px",
      };
      return {
        path: "C:/work/marketing-assets.fleck",
        name: "marketing-assets.fleck",
        warnings: [{ kind: "newer-workspace", found: 2, supported: 1 }],
        missingAssets: [
          {
            assetId: "a1",
            name: "hero-render.png",
            path: "linked/hero-render.png",
            resolvedPath: "C:/work/linked/hero-render.png",
          },
        ],
      } satisfies OpenWorkspaceResult;
    });
  },

  /** Opens a workspace by an explicit path (e.g. from the recent-files list). */
  openWorkspacePath(path: string): Promise<OpenWorkspaceResult | null> {
    return bridge("open_workspace_path", { path }, () => {
      const name = path.split(/[\\/]/).pop() ?? path;
      mockDoc.canvas = { width: 512, height: 512 };
      mockDoc.layers = [
        { id: "layer-icon", name: "Icon", kind: "image", visible: true, locked: false, opacity: 100, blend: "Normal" },
        { id: "layer-grid", name: "Grid", kind: "shape", visible: false, locked: false, opacity: 60, blend: "Multiply" },
      ];
      mockDoc.assets = [
        { id: "asset-mark", name: "mark.png", source: "embedded", path: null, format: "PNG", width: 512, height: 512, missing: false },
      ];
      mockDoc.imageObjects = [
        { id: "img-mark", name: "Mark", sourceAssetId: "asset-mark", position: { x: 64, y: 64 }, scale: { width: 384, height: 384 }, rotationDegrees: 0, opacity: 100, crop: null, rasterizedLayerId: null, replaced: false },
      ];
      mockDoc.outputs = [
        { id: "out-app-png", filename: "app-icon.png", folder: null, format: "png", width: null, height: null, scale: 1, quality: null, background: "transparent", transparency: "preserve", metadata: "strip" },
      ];
      mockDoc.exportAreas = [
        { id: "ea-app", name: "app-icon", bounds: { x: 0, y: 0, width: 512, height: 512 }, padding: { top: 0, right: 0, bottom: 0, left: 0 }, background: "transparent", outputIds: ["out-app-png"], includedLayerIds: [], excludedLayerIds: [], tags: [] },
      ];
      mockDoc.meta = {
        name,
        dirty: false,
        layerCount: mockDoc.layers.length,
        selectedCount: 0,
        canvasSize: "512 × 512 px",
      };
      return { path, name, warnings: [], missingAssets: [] } satisfies OpenWorkspaceResult;
    });
  },

  // --- Image acquisition (native hooks) --------------------------------------
  // These obtain image bytes/paths through native dialogs, the clipboard, or
  // drag/drop. The actual placement is then performed by the undoable `image.*`
  // commands (run via the command engine). Real byte/clipboard/reveal access is
  // Tauri-backed (TASK-020); the mocks below stand in for a browser dev session.

  /** Opens a native image picker; resolves to the chosen path, or null if cancelled. */
  pickImageFile(): Promise<string | null> {
    return bridge("pick_image_file", {}, () => "C:/work/linked/imported-art.png");
  },

  /** Decodes a clipboard image into a new embedded asset; resolves its id + name. */
  acquireClipboardAsset(): Promise<{ assetId: string; name: string } | null> {
    return bridge("acquire_clipboard_asset", {}, () => {
      const assetId = mockAssetId();
      const name = "pasted-image.png";
      mockDoc.assets.push({
        id: assetId,
        name,
        source: "embedded",
        path: null,
        format: "PNG",
        width: 800,
        height: 600,
        missing: false,
      });
      return { assetId, name };
    });
  },

  /** Decodes a dropped image file into a new embedded asset; resolves its id + name. */
  acquireDroppedAsset(name: string): Promise<{ assetId: string; name: string } | null> {
    return bridge("acquire_dropped_asset", { name }, () => {
      const assetId = mockAssetId();
      mockDoc.assets.push({
        id: assetId,
        name,
        source: "embedded",
        path: null,
        format: formatFromPath(name) ?? "PNG",
        width: 1280,
        height: 720,
        missing: false,
      });
      return { assetId, name };
    });
  },

  /** Picks a replacement image and registers it as a new asset; resolves its id. */
  acquireReplacementAsset(): Promise<string | null> {
    return bridge("acquire_replacement_asset", {}, () => {
      const assetId = mockAssetId();
      mockDoc.assets.push({
        id: assetId,
        name: "replacement.png",
        source: "embedded",
        path: null,
        format: "PNG",
        width: 1024,
        height: 1024,
        missing: false,
      });
      return assetId;
    });
  },

  /** Reveals a linked image object's source file in the OS file manager. */
  revealImageSource(objectId: string): Promise<void> {
    return bridge("reveal_image_source", { objectId }, () => undefined);
  },

  saveWorkspace(): Promise<void> {
    return bridge("save_workspace", {}, () => {
      mockDoc.meta.dirty = false;
    });
  },

  /** Opens a native save dialog; resolves to the chosen path, or null if cancelled. */
  saveWorkspaceAs(): Promise<string | null> {
    return bridge("save_workspace_as", {}, () => {
      mockDoc.meta = { ...mockDoc.meta, name: "Copy of " + mockDoc.meta.name, dirty: false };
      return "C:/work/" + mockDoc.meta.name;
    });
  },

  getRecentFiles(): Promise<RecentFile[]> {
    return bridge("get_recent_files", {}, () => [
      { path: "C:/work/brand-assets.fleck", name: "brand-assets.fleck", openedAt: "2 hours ago" },
      { path: "C:/work/marketing-assets.fleck", name: "marketing-assets.fleck", openedAt: "yesterday" },
      { path: "C:/icons/app-icons.fleck", name: "app-icons.fleck", openedAt: "3 days ago" },
    ]);
  },

  /** Opens a picker to relink a missing asset to a file on disk. */
  relinkAsset(assetId: string): Promise<void> {
    return bridge("relink_asset", { assetId }, () => undefined);
  },

  newWorkspace(): Promise<void> {
    return bridge("new_workspace", {}, () => {
      mockDoc.canvas = { width: 0, height: 0 };
      mockDoc.layers = [];
      mockDoc.assets = [];
      mockDoc.imageObjects = [];
      mockDoc.exportAreas = [];
      mockDoc.outputs = [];
      mockDoc.history = { entries: [], currentIndex: null };
      mockDoc.meta = { name: "Untitled.fleck", dirty: false, layerCount: 0, selectedCount: 0, canvasSize: "0 × 0 px" };
    });
  },

  // --- Viewport / rendering ---------------------------------------------------
  // The camera (pan/zoom) lives on the frontend for responsive interaction;
  // these commands cover the parts that need core-owned document bounds.

  /**
   * Read-only geometry for drawing the current frame, in workspace coordinates.
   * Stands in for `fleck-render`'s composited frame; the host applies the
   * viewport transform and paints it.
   */
  getRenderModel(): Promise<RenderModel> {
    return bridge("get_render_model", {}, () => buildRenderModel());
  },

  /**
   * Compute a target viewport for a focus action that needs document bounds
   * (fit / zoom-to-selection / zoom-to-export-area). `actual` and `pixel-perfect`
   * are handled on the frontend and don't call this.
   */
  getViewportFocus(
    kind: ViewportFocusKind,
    screen: Size,
    targetId?: string | null,
  ): Promise<{ origin: Point; zoom: number } | null> {
    return bridge("get_viewport_focus", { kind, screen, targetId }, () => {
      const model = buildRenderModel();
      let rect: Rect | null = null;
      if (kind === "selection") rect = model.selections[0]?.rect ?? null;
      else if (kind === "export-area")
        rect = (targetId && model.exportAreas.find((a) => a.id === targetId)?.rect) || model.exportAreas[0]?.rect || null;
      else rect = model.canvas.width > 0 ? { x: 0, y: 0, width: model.canvas.width, height: model.canvas.height } : null;
      if (!rect) return null;
      const fitted = fitRect(rect, screen);
      return { origin: fitted.origin, zoom: fitted.zoom };
    });
  },

  createExportArea(): Promise<void> {
    return bridge("create_export_area", {}, () => undefined);
  },

  /** Run the export job for one area; resolves with the produced result/report. */
  exportArea(id: string): Promise<ExportResult> {
    return bridge("export_area", { id }, () => {
      const area = mockDoc.exportAreas.find((a) => a.id === id);
      return runMockExport(area?.name ?? "Export area", area ? [area] : []);
    });
  },

  /** Run the export job for every area; resolves with the aggregate report. */
  exportAll(): Promise<ExportResult> {
    return bridge("export_all", {}, () => runMockExport("All areas", mockDoc.exportAreas));
  },

  /**
   * Reveal a produced output in the OS file manager. Native integration is
   * Tauri-backed (TASK-020); the mock stands in for a browser dev session.
   */
  revealExportedFile(destination: string): Promise<void> {
    return bridge("reveal_exported_file", { destination }, () => undefined);
  },

  /**
   * Copy an export result to the clipboard. `mode` selects image bytes vs. a
   * Base64 / Markdown text encoding. Native clipboard access is Tauri-backed
   * (TASK-020); the mock stands in for a browser dev session.
   */
  copyExportResult(outputId: string, mode: "image" | "base64" | "markdown"): Promise<void> {
    return bridge("copy_export_result", { outputId, mode }, () => undefined);
  },

  // --- Command engine ---------------------------------------------------------

  /**
   * Execute a registered command by id, optionally with collected parameters.
   * Undoable commands append to history (mirrors `CommandEngine::execute`).
   */
  runCommand(commandId: string, parameters: Record<string, unknown> = {}): Promise<CommandExecution> {
    return bridge("run_command", { commandId, parameters }, () => {
      // Layer commands mutate the mock document directly; the engine still
      // records every applied change as an undoable history entry.
      if (commandId.startsWith("layer.")) {
        const changed = applyLayerMutation(commandId, parameters);
        const label = LAYER_OP_LABELS[commandId] ?? commandId;
        if (changed) pushHistory(commandId, label);
        return { commandId, operationLabel: label } satisfies CommandExecution;
      }
      if (commandId.startsWith("image.")) {
        const changed = applyImageMutation(commandId, parameters);
        const label = IMAGE_OP_LABELS[commandId] ?? commandId;
        if (changed) pushHistory(commandId, label);
        return { commandId, operationLabel: label } satisfies CommandExecution;
      }
      if (commandId.startsWith("export_area.") || commandId.startsWith("output.")) {
        const changed = applyExportMutation(commandId, parameters);
        const label = EXPORT_OP_LABELS[commandId] ?? commandId;
        if (changed) pushHistory(commandId, label);
        return { commandId, operationLabel: label } satisfies CommandExecution;
      }
      const def = COMMAND_DEFINITIONS.find((c) => c.id === commandId);
      const label = operationLabel(def?.label ?? commandId, parameters);
      if (def?.undoable) pushHistory(commandId, label);
      return { commandId, operationLabel: label } satisfies CommandExecution;
    });
  },

  undo(): Promise<CommandExecution | null> {
    return bridge("undo", {}, () => {
      const { entries, currentIndex } = mockDoc.history;
      if (currentIndex === null) return null;
      const entry = entries[currentIndex];
      mockDoc.history.currentIndex = currentIndex === 0 ? null : currentIndex - 1;
      return { commandId: entry.commandId, operationLabel: `Undo ${entry.label}` };
    });
  },

  redo(): Promise<CommandExecution | null> {
    return bridge("redo", {}, () => {
      const { entries, currentIndex } = mockDoc.history;
      const next = currentIndex === null ? 0 : currentIndex + 1;
      const entry = entries[next];
      if (!entry) return null;
      mockDoc.history.currentIndex = next;
      return { commandId: entry.commandId, operationLabel: `Redo ${entry.label}` };
    });
  },

  /**
   * Jump to an arbitrary history state. Supported here by stepping the cursor;
   * `index` of -1 means "before the first entry". Backends that can't jump
   * should omit this command — the UI hides the affordance when unsupported.
   */
  jumpToHistory(index: number): Promise<void> {
    return bridge("jump_to_history", { index }, () => {
      const max = mockDoc.history.entries.length - 1;
      mockDoc.history.currentIndex = index < 0 ? null : Math.min(index, max);
    });
  },

  /** Whether the backend supports jump-to-state (vs. stepwise undo/redo only). */
  supportsHistoryJump(): Promise<boolean> {
    return bridge("supports_history_jump", {}, () => true);
  },
};

/** Render an operation label, interpolating a `name` parameter when present. */
function operationLabel(base: string, parameters: Record<string, unknown>): string {
  const name = parameters.name;
  if (typeof name === "string" && name.trim()) return `${base} → ${name}`;
  return base;
}
