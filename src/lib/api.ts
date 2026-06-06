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
import type { ExportArea, HistoryEntry, Layer, WorkspaceMeta } from "./fleck-data";

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

const mockDoc: {
  meta: WorkspaceMeta;
  layers: Layer[];
  exportAreas: ExportArea[];
  history: HistoryEntry[];
} = {
  // Fresh, untitled workspace — the shell opens empty until a real document loads.
  history: [{ id: "h1", label: "New workspace", current: true }],
  meta: {
    name: "Untitled.fleck",
    dirty: false,
    layerCount: 0,
    selectedCount: 0,
    canvasSize: "0 × 0 px",
  },
  layers: [],
  exportAreas: [],
};

// --- Queries (read document state) -------------------------------------------

export const api = {
  getWorkspaceMeta(): Promise<WorkspaceMeta> {
    return bridge("get_workspace_meta", {}, () => ({ ...mockDoc.meta }));
  },

  getLayers(): Promise<Layer[]> {
    return bridge("get_layers", {}, () => mockDoc.layers.map((l) => ({ ...l })));
  },

  getExportAreas(): Promise<ExportArea[]> {
    return bridge("get_export_areas", {}, () =>
      mockDoc.exportAreas.map((a) => ({ ...a, outputs: a.outputs.map((o) => ({ ...o })) })),
    );
  },

  getHistory(): Promise<HistoryEntry[]> {
    return bridge("get_history", {}, () => mockDoc.history.map((h) => ({ ...h })));
  },

  // --- Mutations (request document changes) ----------------------------------

  setLayerVisibility(id: string, visible: boolean): Promise<void> {
    return bridge("set_layer_visibility", { id, visible }, () => {
      const layer = mockDoc.layers.find((l) => l.id === id);
      if (layer) layer.visible = visible;
    });
  },

  setLayerLocked(id: string, locked: boolean): Promise<void> {
    return bridge("set_layer_locked", { id, locked }, () => {
      const layer = mockDoc.layers.find((l) => l.id === id);
      if (layer) layer.locked = locked;
    });
  },

  // --- Actions (commands without immediate document reads) --------------------

  openImage(): Promise<void> {
    return bridge("open_image", {}, () => undefined);
  },

  saveWorkspace(): Promise<void> {
    return bridge("save_workspace", {}, () => {
      mockDoc.meta.dirty = false;
    });
  },

  newWorkspace(): Promise<void> {
    return bridge("new_workspace", {}, () => undefined);
  },

  createExportArea(): Promise<void> {
    return bridge("create_export_area", {}, () => undefined);
  },

  exportArea(id: string): Promise<void> {
    return bridge("export_area", { id }, () => undefined);
  },

  exportAll(): Promise<void> {
    return bridge("export_all", {}, () => undefined);
  },

  undo(): Promise<void> {
    return bridge("undo", {}, () => undefined);
  },

  redo(): Promise<void> {
    return bridge("redo", {}, () => undefined);
  },

  /** Generic command-palette dispatch. */
  runCommand(commandId: string): Promise<void> {
    return bridge("run_command", { commandId }, () => undefined);
  },
};
