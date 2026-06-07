import type { LucideIcon } from "lucide-react";
import {
  MousePointer2,
  Square,
  Lasso,
  Wand2,
  Brush,
  Eraser,
  PaintBucket,
  Crop,
  Type,
  Shapes,
  Pipette,
  Frame,
  Hand,
  ZoomIn,
} from "lucide-react";

export type Tool = {
  id: string;
  name: string;
  shortcut: string;
  hint: string;
  icon: LucideIcon;
};

// Every tool has a unique name, shortcut and one-line purpose — no duplicate labels.
export const TOOLS: Tool[] = [
  { id: "move", name: "Move", shortcut: "V", hint: "Move layers and selected pixels", icon: MousePointer2 },
  { id: "marquee", name: "Marquee", shortcut: "M", hint: "Select a rectangular region", icon: Square },
  { id: "lasso", name: "Lasso", shortcut: "L", hint: "Draw a freehand selection", icon: Lasso },
  { id: "wand", name: "Magic wand", shortcut: "W", hint: "Select by color similarity", icon: Wand2 },
  { id: "brush", name: "Brush", shortcut: "B", hint: "Paint soft pixels", icon: Brush },
  { id: "eraser", name: "Eraser", shortcut: "E", hint: "Erase to transparency", icon: Eraser },
  { id: "fill", name: "Fill", shortcut: "G", hint: "Flood-fill a region", icon: PaintBucket },
  { id: "crop", name: "Crop", shortcut: "C", hint: "Trim the working bounds", icon: Crop },
  { id: "text", name: "Text", shortcut: "T", hint: "Add an editable text layer", icon: Type },
  { id: "shape", name: "Shape", shortcut: "U", hint: "Draw rectangles, lines and arrows", icon: Shapes },
  { id: "picker", name: "Picker", shortcut: "I", hint: "Sample a color from the canvas", icon: Pipette },
  { id: "export-area", name: "Export area", shortcut: "A", hint: "Mark a named output region", icon: Frame },
  { id: "pan", name: "Pan", shortcut: "Space", hint: "Drag to pan the workspace", icon: Hand },
  { id: "zoom", name: "Zoom", shortcut: "Z", hint: "Zoom into a point", icon: ZoomIn },
];

// --- Geometry / viewport / rendering (mirror fleck-core::geometry + fleck-render) ---

export type Point = { x: number; y: number };
export type Size = { width: number; height: number };
export type Rect = { x: number; y: number; width: number; height: number };

/** Camera state (mirrors `geometry::Viewport`): origin in workspace units. */
export type Viewport = {
  origin: Point;
  zoom: number;
  screen: Size;
};

/** Mirrors `geometry::OverlaySettings`. */
export type OverlaySettings = {
  checkerboard: boolean;
  guides: boolean;
  pixelGrid: { enabled: boolean; minZoom: number };
  selections: boolean;
  transformHandles: boolean;
  exportAreas: boolean;
};

export type ViewportFocusKind = "fit" | "selection" | "export-area" | "actual" | "pixel-perfect";

/**
 * Read-only workspace geometry needed to draw a frame, in workspace coordinates.
 * Stands in for what `fleck-render` composites from core-owned state; the host
 * applies the viewport transform and paints it.
 */
export type RenderModel = {
  canvas: { width: number; height: number };
  layers: { id: string; rect: Rect; color: string; opacity: number; visible: boolean }[];
  exportAreas: { id: string; name: string; rect: Rect }[];
  guides: { axis: "horizontal" | "vertical"; position: number }[];
  selections: { id: string; rect: Rect }[];
};

/** Mirrors `fleck-core::model::BlendMode`. */
export type BlendMode =
  | "Normal"
  | "Multiply"
  | "Screen"
  | "Overlay"
  | "Darken"
  | "Lighten"
  | "ColorDodge"
  | "ColorBurn"
  | "HardLight"
  | "SoftLight"
  | "Difference"
  | "Exclusion"
  | "Hue"
  | "Saturation"
  | "Color"
  | "Luminosity";

export type Layer = {
  id: string;
  name: string;
  kind: "image" | "text" | "shape" | "mask" | "group";
  visible: boolean;
  locked: boolean;
  opacity: number;
  blend: BlendMode;
};

export type Output = {
  id: string;
  filename: string;
  format: string;
  size: string;
  bytes: string;
};

export type ExportArea = {
  id: string;
  name: string;
  dimensions: string;
  format: string;
  status: "ready" | "warning" | "stale";
  note?: string;
  outputs: Output[];
};

export type WorkspaceMeta = {
  name: string;
  dirty: boolean;
  layerCount: number;
  selectedCount: number;
  canvasSize: string;
};

/**
 * Workspace-file types mirror the Rust `fleck-core::persistence` DTOs so the UI
 * consumes structured results from the backend instead of parsing `.fleck` files.
 */
export type LoadWarning =
  | { kind: "migrated"; from: number; to: number }
  | { kind: "newer-file"; found: number; supported: number }
  | { kind: "newer-workspace"; found: number; supported: number };

/** A linked asset the backend could not resolve when opening a workspace. */
export type MissingAsset = {
  assetId: string;
  name: string;
  /** Path as stored in the workspace (may be relative). */
  path: string;
  /** Absolute path the backend tried to resolve. */
  resolvedPath: string;
};

export type OpenWorkspaceResult = {
  path: string;
  name: string;
  warnings: LoadWarning[];
  missingAssets: MissingAsset[];
};

export type RecentFile = {
  path: string;
  name: string;
  openedAt: string;
};

/**
 * Command + history types mirror the Rust `fleck-core::command` contract so the
 * palette and history panel consume the real registry/engine shape.
 */
export type CommandGroup =
  | "workspace"
  | "layer"
  | "selection"
  | "export"
  | "recipe"
  | "view"
  | "tool";

export type ParameterKind = "string" | "number" | "boolean" | "object_id";

export type ParameterPrompt = {
  key: string;
  label: string;
  kind: ParameterKind;
  required: boolean;
};

export type CommandDefinition = {
  id: string;
  label: string;
  description: string;
  group: CommandGroup;
  aliases: string[];
  shortcut?: string;
  undoable: boolean;
  parameterPrompts: ParameterPrompt[];
};

/** Result of executing a command (mirrors `CommandExecution`). */
export type CommandExecution = {
  commandId: string;
  operationLabel: string;
};

export type HistoryEntry = {
  id: string;
  commandId: string;
  label: string;
};

/** Mirrors `fleck-core::model::HistoryState`. */
export type HistoryState = {
  entries: HistoryEntry[];
  /** Index of the currently active state; null means before the first entry. */
  currentIndex: number | null;
};
