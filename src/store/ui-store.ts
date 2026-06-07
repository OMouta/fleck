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

export type SideTab = "layers" | "exports";

type UIState = {
  // Active tool selection
  activeTool: string;
  setActiveTool: (id: string) => void;

  // Which layer row is focused in the inspector (UI focus, not document data)
  selectedLayerId: string | null;
  setSelectedLayerId: (id: string) => void;

  // Command palette visibility
  paletteOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
  togglePalette: () => void;

  // Side panel tab + which export area is expanded
  sideTab: SideTab;
  setSideTab: (tab: SideTab) => void;
  openExportAreaId: string | null;
  setOpenExportAreaId: (id: string | null) => void;
};

export const useUIStore = create<UIState>((set) => ({
  activeTool: "move",
  setActiveTool: (id) => set({ activeTool: id }),

  selectedLayerId: null,
  setSelectedLayerId: (id) => set({ selectedLayerId: id }),

  paletteOpen: false,
  setPaletteOpen: (open) => set({ paletteOpen: open }),
  togglePalette: () => set((s) => ({ paletteOpen: !s.paletteOpen })),

  sideTab: "layers",
  setSideTab: (tab) => set({ sideTab: tab }),
  openExportAreaId: null,
  setOpenExportAreaId: (id) => set({ openExportAreaId: id }),
}));
