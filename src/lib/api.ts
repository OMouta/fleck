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
  ExportArea,
  HistoryEntry,
  Layer,
  OpenWorkspaceResult,
  RecentFile,
  WorkspaceMeta,
} from "./fleck-data";

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
      mockDoc.meta = {
        name: "marketing-assets.fleck",
        dirty: false,
        layerCount: 0,
        selectedCount: 0,
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
      mockDoc.meta = { name, dirty: false, layerCount: 0, selectedCount: 0, canvasSize: "0 × 0 px" };
      return { path, name, warnings: [], missingAssets: [] } satisfies OpenWorkspaceResult;
    });
  },

  openImage(): Promise<void> {
    return bridge("open_image", {}, () => undefined);
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
      mockDoc.meta = { name: "Untitled.fleck", dirty: false, layerCount: 0, selectedCount: 0, canvasSize: "0 × 0 px" };
    });
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
