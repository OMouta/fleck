/**
 * Viewport (camera) + overlay state for the canvas host.
 *
 * The camera is presentation state, so it lives here (UI), not in the document.
 * Pan/zoom math uses the `lib/viewport` helpers that mirror `fleck-core`. Focus
 * actions that need document bounds (fit / selection / export area) route through
 * the shared `api` command; actual-size and pixel-perfect are pure camera moves.
 */
import { create } from "zustand";
import { api } from "@/lib/api";
import type { OverlaySettings, Point, Size, Viewport, ViewportFocusKind } from "@/lib/fleck-data";
import {
  MIN_PIXEL_GRID_ZOOM,
  clampZoom,
  panByScreenDelta,
  zoomAroundScreenPoint,
} from "@/lib/viewport";

type OverlayToggleKey = Exclude<keyof OverlaySettings, "pixelGrid">;

type ViewportState = {
  origin: Point;
  zoom: number;
  screen: Size;
  overlays: OverlaySettings;
  /** True while a drag-pan is in progress (drives the cursor). */
  panning: boolean;

  setScreen: (screen: Size) => void;
  panByScreen: (dx: number, dy: number) => void;
  zoomAt: (screen: Point, factor: number) => void;
  zoomCentered: (factor: number) => void;
  setPanning: (panning: boolean) => void;

  /** Focus actions: fit / selection / export-area / actual (100%) / pixel-perfect. */
  focus: (kind: ViewportFocusKind) => Promise<void>;

  toggleOverlay: (key: OverlayToggleKey) => void;
  togglePixelGrid: () => void;
};

const view = (s: ViewportState): Viewport => ({ origin: s.origin, zoom: s.zoom, screen: s.screen });

export const useViewportStore = create<ViewportState>((set, get) => ({
  origin: { x: 0, y: 0 },
  zoom: 1,
  screen: { width: 0, height: 0 },
  panning: false,
  overlays: {
    checkerboard: true,
    guides: true,
    pixelGrid: { enabled: true, minZoom: MIN_PIXEL_GRID_ZOOM },
    selections: true,
    transformHandles: true,
    exportAreas: true,
  },

  setScreen: (screen) => set({ screen }),

  panByScreen: (dx, dy) => {
    const next = panByScreenDelta(view(get()), dx, dy);
    set({ origin: next.origin });
  },

  zoomAt: (screen, factor) => {
    const s = get();
    const next = zoomAroundScreenPoint(view(s), screen, s.zoom * factor);
    set({ origin: next.origin, zoom: next.zoom });
  },

  zoomCentered: (factor) => {
    const s = get();
    get().zoomAt({ x: s.screen.width / 2, y: s.screen.height / 2 }, factor);
  },

  setPanning: (panning) => set({ panning }),

  focus: async (kind) => {
    const s = get();
    const center: Point = { x: s.screen.width / 2, y: s.screen.height / 2 };
    if (kind === "actual") {
      const next = zoomAroundScreenPoint(view(s), center, 1);
      set({ origin: next.origin, zoom: next.zoom });
      return;
    }
    if (kind === "pixel-perfect") {
      const next = zoomAroundScreenPoint(view(s), center, Math.max(1, Math.round(s.zoom)));
      set({ origin: next.origin, zoom: next.zoom });
      return;
    }
    const result = await api.getViewportFocus(kind, s.screen);
    if (result) set({ origin: result.origin, zoom: clampZoom(result.zoom) });
  },

  toggleOverlay: (key) => set((s) => ({ overlays: { ...s.overlays, [key]: !s.overlays[key] } })),

  togglePixelGrid: () =>
    set((s) => ({
      overlays: { ...s.overlays, pixelGrid: { ...s.overlays.pixelGrid, enabled: !s.overlays.pixelGrid.enabled } },
    })),
}));
