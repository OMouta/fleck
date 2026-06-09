/**
 * Zustand store for immediate UI state ONLY.
 *
 * Per `docs/architecture.md` and REQ-036, this store must never hold document
 * truth (layers, areas, pixels). It only tracks ephemeral interaction
 * state: active tool, selection focus, and panel visibility. Camera/overlay
 * (viewport) state lives in `viewport-store`. Document state lives in the Rust
 * core and is read via TanStack Query.
 */
import { create } from "zustand";

export type SideTab = "layers" | "images" | "areas" | "history";

/** Marquee variants commit to `selection.rect` or `selection.ellipse`. */
export type MarqueeShape = "rect" | "ellipse";

/**
 * Lasso variants commit to `selection.lasso` (freehand point sampling) or
 * `selection.polygon` (clicked vertices closed on dblclick / Enter).
 */
export type LassoMode = "freehand" | "polygon";

/** RGBA in 0–255 (matches the `r/g/b/a` core parameter prompts). */
export type ToolColor = { r: number; g: number; b: number; a: number };

/**
 * Active pixel tool options. Carried as command parameters when the tool fires
 * (brush radius/opacity/color, fill tolerance, etc.) so the user controls them
 * from the tool-options bar without re-prompting per stroke.
 */
export type ToolOptions = {
  /** Brush/pencil/clone/heal/blur/sharpen radius. */
  brushRadius: number;
  /** Brush/pencil stroke opacity (0–1). */
  brushOpacity: number;
  eraserRadius: number;
  eraserOpacity: number;
  /** Magic-wand and color-range tolerance (0–1). */
  wandTolerance: number;
  /** Active paint colour, shared by brush/pencil/fill/gradient. */
  color: ToolColor;
};

const DEFAULT_TOOL_OPTIONS: ToolOptions = {
  brushRadius: 8,
  brushOpacity: 1,
  eraserRadius: 12,
  eraserOpacity: 1,
  wandTolerance: 0.1,
  color: { r: 0, g: 0, b: 0, a: 255 },
};

type UIState = {
  // Active tool selection
  activeTool: string;
  setActiveTool: (id: string) => void;

  /** Marquee tool shape variant — rect vs ellipse. */
  marqueeShape: MarqueeShape;
  setMarqueeShape: (shape: MarqueeShape) => void;

  /** Lasso tool mode — freehand vs polygon. */
  lassoMode: LassoMode;
  setLassoMode: (mode: LassoMode) => void;

  /** Pixel tool options consumed by stroke/fill/gradient invocations. */
  toolOptions: ToolOptions;
  setToolOption: <K extends keyof ToolOptions>(key: K, value: ToolOptions[K]) => void;

  // Which layer row is focused in the inspector (UI focus, not document data)
  selectedLayerId: string | null;
  setSelectedLayerId: (id: string) => void;

  // Which placed image object is focused in the Images panel inspector
  selectedImageObjectId: string | null;
  setSelectedImageObjectId: (id: string) => void;

  /**
   * Active selection mask (UI focus on `Workspace::selections`). Drives the
   * canvas HUD, status bar, and the default `id` parameter for `selection.*`
   * follow-up commands.
   */
  activeSelectionId: string | null;
  setActiveSelectionId: (id: string | null) => void;

  // Command palette visibility
  paletteOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
  togglePalette: () => void;

  // Side panel tab + which area is selected (shared across the areas
  // panel, the export inspector, and the canvas highlight/overlay).
  sideTab: SideTab;
  setSideTab: (tab: SideTab) => void;
  selectedAreaId: string | null;
  setSelectedAreaId: (id: string | null) => void;

  // Export preview/result dialog visibility (targets the selected area).
  exportPreviewOpen: boolean;
  setExportPreviewOpen: (open: boolean) => void;
};

export const useUIStore = create<UIState>((set) => ({
  activeTool: "move",
  setActiveTool: (id) => set({ activeTool: id }),

  marqueeShape: "rect",
  setMarqueeShape: (shape) => set({ marqueeShape: shape }),

  lassoMode: "freehand",
  setLassoMode: (mode) => set({ lassoMode: mode }),

  toolOptions: DEFAULT_TOOL_OPTIONS,
  setToolOption: (key, value) =>
    set((s) => ({ toolOptions: { ...s.toolOptions, [key]: value } })),

  selectedLayerId: null,
  setSelectedLayerId: (id) => set({ selectedLayerId: id }),

  selectedImageObjectId: null,
  setSelectedImageObjectId: (id) => set({ selectedImageObjectId: id }),

  activeSelectionId: null,
  setActiveSelectionId: (id) => set({ activeSelectionId: id }),

  paletteOpen: false,
  setPaletteOpen: (open) => set({ paletteOpen: open }),
  togglePalette: () => set((s) => ({ paletteOpen: !s.paletteOpen })),

  sideTab: "layers",
  setSideTab: (tab) => set({ sideTab: tab }),
  selectedAreaId: null,
  setSelectedAreaId: (id) => set({ selectedAreaId: id }),

  exportPreviewOpen: false,
  setExportPreviewOpen: (open) => set({ exportPreviewOpen: open }),
}));
