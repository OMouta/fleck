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
  HistoryEntry,
  HistoryState,
  Layer,
  OpenWorkspaceResult,
  Point,
  RecentFile,
  Rect,
  RenderModel,
  Size,
  ViewportFocusKind,
  WorkspaceMeta,
} from "./fleck-data";
import { COMMAND_DEFINITIONS } from "./command-registry";
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

const mockDoc: {
  meta: WorkspaceMeta;
  /** Canvas dimensions in workspace pixels (0 = no document loaded yet). */
  canvas: { width: number; height: number };
  layers: Layer[];
  exportAreas: ExportArea[];
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
  exportAreas: [],
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
  const badge: Rect = { x: width * 0.66, y: height * 0.1, width: width * 0.26, height: width * 0.26 };
  return {
    canvas: { width, height },
    layers: [
      { id: "rl-base", rect: { x: 0, y: 0, width, height }, color: "#2b3b55", opacity: 1, visible: true },
      { id: "rl-art", rect: inset, color: "#3a86ff", opacity: 0.9, visible: true },
    ],
    exportAreas: [
      { id: "ea-frame", name: "frame", rect: { x: 0, y: 0, width, height } },
      { id: "ea-icon", name: "icon", rect: badge },
    ],
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
      mockDoc.canvas = { width: 1200, height: 630 };
      mockDoc.meta = {
        name: "marketing-assets.fleck",
        dirty: false,
        layerCount: 2,
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
      mockDoc.meta = { name, dirty: false, layerCount: 2, selectedCount: 0, canvasSize: "512 × 512 px" };
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
      mockDoc.canvas = { width: 0, height: 0 };
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
  getViewportFocus(kind: ViewportFocusKind, screen: Size): Promise<{ origin: Point; zoom: number } | null> {
    return bridge("get_viewport_focus", { kind, screen }, () => {
      const model = buildRenderModel();
      let rect: Rect | null = null;
      if (kind === "selection") rect = model.selections[0]?.rect ?? null;
      else if (kind === "export-area") rect = model.exportAreas[0]?.rect ?? null;
      else rect = model.canvas.width > 0 ? { x: 0, y: 0, width: model.canvas.width, height: model.canvas.height } : null;
      if (!rect) return null;
      const fitted = fitRect(rect, screen);
      return { origin: fitted.origin, zoom: fitted.zoom };
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

  // --- Command engine ---------------------------------------------------------

  /**
   * Execute a registered command by id, optionally with collected parameters.
   * Undoable commands append to history (mirrors `CommandEngine::execute`).
   */
  runCommand(commandId: string, parameters: Record<string, unknown> = {}): Promise<CommandExecution> {
    return bridge("run_command", { commandId, parameters }, () => {
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
