/**
 * Zustand store for immediate UI state ONLY.
 *
 * Per `docs/architecture.md` and REQ-036, this store must never hold document
 * truth (layers, export areas, pixels). It only tracks ephemeral interaction
 * state: active tool, selection focus, and panel visibility. Camera/overlay
 * (viewport) state lives in `viewport-store`. Document state lives in the Rust
 * core and is read via TanStack Query.
 */
import { create } from "zustand";

export type SideTab = "layers" | "images" | "exports" | "history";

type UIState = {
  // Active tool selection
  activeTool: string;
  setActiveTool: (id: string) => void;

  // Which layer row is focused in the inspector (UI focus, not document data)
  selectedLayerId: string | null;
  setSelectedLayerId: (id: string) => void;

  // Which placed image object is focused in the Images panel inspector
  selectedImageObjectId: string | null;
  setSelectedImageObjectId: (id: string) => void;

  // Command palette visibility
  paletteOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
  togglePalette: () => void;

  // Side panel tab + which export area is selected (shared across the exports
  // panel, the export inspector, and the canvas highlight/overlay).
  sideTab: SideTab;
  setSideTab: (tab: SideTab) => void;
  selectedExportAreaId: string | null;
  setSelectedExportAreaId: (id: string | null) => void;
};

export const useUIStore = create<UIState>((set) => ({
  activeTool: "move",
  setActiveTool: (id) => set({ activeTool: id }),

  selectedLayerId: null,
  setSelectedLayerId: (id) => set({ selectedLayerId: id }),

  selectedImageObjectId: null,
  setSelectedImageObjectId: (id) => set({ selectedImageObjectId: id }),

  paletteOpen: false,
  setPaletteOpen: (open) => set({ paletteOpen: open }),
  togglePalette: () => set((s) => ({ paletteOpen: !s.paletteOpen })),

  sideTab: "layers",
  setSideTab: (tab) => set({ sideTab: tab }),
  selectedExportAreaId: null,
  setSelectedExportAreaId: (id) => set({ selectedExportAreaId: id }),
}));
