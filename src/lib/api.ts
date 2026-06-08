/**
 * Backend API wrapper. Every call goes through Tauri's `invoke`.
 *
 * There is no browser/dev fallback. If the desktop shell is not present the
 * call throws — surfacing missing backend work loudly instead of papering over
 * it with a fake document. See `docs/architecture.md`: Rust owns document
 * truth, React owns UI.
 */
import type {
  CommandDefinition,
  CommandExecution,
  ExportArea,
  ExportResult,
  HistoryState,
  ImageObject,
  Layer,
  OpenWorkspaceResult,
  Point,
  RecentFile,
  RenderModel,
  Size,
  ViewportFocusKind,
  WorkspaceMeta,
} from "./fleck-data";

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

let cachedInvoke: Invoke | null = null;
let invokeLoader: Promise<Invoke> | null = null;

function ensureTauri(): Promise<Invoke> {
  if (cachedInvoke) return Promise.resolve(cachedInvoke);
  if (invokeLoader) return invokeLoader;
  const internals = (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  if (!internals) {
    return Promise.reject(
      new Error(
        "Tauri bridge unavailable. Run the desktop app (`pnpm tauri dev`) — the frontend has no in-browser fallback.",
      ),
    );
  }
  invokeLoader = import("@tauri-apps/api/core").then((m) => {
    cachedInvoke = m.invoke as Invoke;
    return cachedInvoke;
  });
  return invokeLoader;
}

async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  const invoke = await ensureTauri();
  return invoke(command, args) as Promise<T>;
}

export const api = {
  getWorkspaceMeta: () => call<WorkspaceMeta>("get_workspace_meta"),
  getLayers: () => call<Layer[]>("get_layers"),
  getImageObjects: () => call<ImageObject[]>("get_image_objects"),
  getExportAreas: () => call<ExportArea[]>("get_export_areas"),
  getHistory: () => call<HistoryState>("get_history"),
  getCommands: () => call<CommandDefinition[]>("get_commands"),

  newWorkspace: () => call<void>("new_workspace"),
  openWorkspace: () => call<OpenWorkspaceResult | null>("open_workspace"),
  openWorkspacePath: (path: string) => call<OpenWorkspaceResult | null>("open_workspace_path", { path }),
  saveWorkspace: () => call<void>("save_workspace"),
  saveWorkspaceAs: () => call<string | null>("save_workspace_as"),
  getRecentFiles: () => call<RecentFile[]>("get_recent_files"),
  relinkAsset: (assetId: string) => call<void>("relink_asset", { assetId }),

  pickImageFile: () => call<string | null>("pick_image_file"),
  acquireClipboardAsset: () => call<{ assetId: string; name: string } | null>("acquire_clipboard_asset"),
  acquireDroppedAsset: (name: string) =>
    call<{ assetId: string; name: string } | null>("acquire_dropped_asset", { name }),
  acquireReplacementAsset: () => call<string | null>("acquire_replacement_asset"),
  revealImageSource: (objectId: string) => call<void>("reveal_image_source", { objectId }),

  getRenderModel: () => call<RenderModel>("get_render_model"),
  getViewportFocus: (kind: ViewportFocusKind, screen: Size, targetId?: string | null) =>
    call<{ origin: Point; zoom: number } | null>("get_viewport_focus", { kind, screen, targetId: targetId ?? null }),

  createExportArea: () => call<void>("create_export_area"),
  exportArea: (id: string) => call<ExportResult>("export_area", { id }),
  exportAll: () => call<ExportResult>("export_all"),
  revealExportedFile: (destination: string) => call<void>("reveal_exported_file", { destination }),
  copyExportResult: (outputId: string, mode: "image" | "base64" | "markdown") =>
    call<void>("copy_export_result", { outputId, mode }),

  runCommand: (commandId: string, parameters: Record<string, unknown> = {}) =>
    call<CommandExecution>("run_command", { commandId, parameters }),
  undo: () => call<CommandExecution | null>("undo"),
  redo: () => call<CommandExecution | null>("redo"),
  jumpToHistory: (index: number) => call<void>("jump_to_history", { index }),
  supportsHistoryJump: () => call<boolean>("supports_history_jump"),
};
