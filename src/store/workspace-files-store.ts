/**
 * Transient UI state for the workspace-file flow (open / save / save-as) and the
 * follow-up dialogs (missing linked assets, unsupported newer versions).
 *
 * This holds only ephemeral interaction/flow state and the structured result the
 * backend returned for the dialog to display — never the document itself, which
 * stays in the Rust core and is read via TanStack Query. Actions invoke the
 * shared `api` wrapper and invalidate the relevant queries so panels refresh.
 */
import { create } from "zustand";
import { api } from "@/lib/api";
import { queryClient } from "@/lib/query-client";
import { queryKeys } from "@/lib/queries";
import type { OpenWorkspaceResult } from "@/lib/fleck-data";

type FileDialog = "missing-assets" | "newer-version" | null;

type WorkspaceFilesState = {
  /** Which follow-up dialog is currently shown after an open. */
  dialog: FileDialog;
  /** The load result backing the active dialog. */
  pending: OpenWorkspaceResult | null;

  openWorkspace: () => Promise<void>;
  openWorkspacePath: (path: string) => Promise<void>;
  newWorkspace: () => Promise<void>;
  openImage: () => Promise<void>;
  save: () => Promise<void>;
  saveAs: () => Promise<void>;

  /** Advance past the newer-version warning into loading (read-only). */
  acceptNewerVersion: () => void;
  /** Relink one missing asset, then drop it from the pending list. */
  relinkAsset: (assetId: string) => Promise<void>;
  /** Dismiss the active dialog without taking further action. */
  dismissDialog: () => void;
};

/** Refresh everything that depends on the loaded document. */
function invalidateDocument() {
  queryClient.invalidateQueries({ queryKey: queryKeys.workspaceMeta });
  queryClient.invalidateQueries({ queryKey: queryKeys.layers });
  queryClient.invalidateQueries({ queryKey: queryKeys.exportAreas });
  queryClient.invalidateQueries({ queryKey: queryKeys.history });
}

export const useWorkspaceFilesStore = create<WorkspaceFilesState>((set, get) => ({
  dialog: null,
  pending: null,

  openWorkspace: async () => {
    const result = await api.openWorkspace();
    if (!result) return; // user cancelled the native picker
    handleOpenResult(set, result);
  },

  openWorkspacePath: async (path) => {
    const result = await api.openWorkspacePath(path);
    if (!result) return;
    handleOpenResult(set, result);
  },

  newWorkspace: async () => {
    await api.newWorkspace();
    invalidateDocument();
  },

  openImage: async () => {
    await api.openImage();
    invalidateDocument();
  },

  save: async () => {
    await api.saveWorkspace();
    queryClient.invalidateQueries({ queryKey: queryKeys.workspaceMeta });
  },

  saveAs: async () => {
    const path = await api.saveWorkspaceAs();
    if (!path) return;
    queryClient.invalidateQueries({ queryKey: queryKeys.workspaceMeta });
    queryClient.invalidateQueries({ queryKey: queryKeys.recentFiles });
  },

  acceptNewerVersion: () => {
    const { pending } = get();
    // After accepting the version warning, fall through to missing assets if any.
    if (pending && pending.missingAssets.length > 0) {
      set({ dialog: "missing-assets" });
    } else {
      finishOpen(set);
    }
  },

  relinkAsset: async (assetId) => {
    await api.relinkAsset(assetId);
    const { pending } = get();
    if (!pending) return;
    const missingAssets = pending.missingAssets.filter((a) => a.assetId !== assetId);
    if (missingAssets.length === 0) {
      finishOpen(set);
    } else {
      set({ pending: { ...pending, missingAssets } });
    }
  },

  dismissDialog: () => finishOpen(set),
}));

/** Decide which dialog (if any) an open result requires, else finish loading. */
function handleOpenResult(
  set: (partial: Partial<WorkspaceFilesState>) => void,
  result: OpenWorkspaceResult,
) {
  if (result.warnings.some((w) => w.kind === "newer-file" || w.kind === "newer-workspace")) {
    set({ pending: result, dialog: "newer-version" });
  } else if (result.missingAssets.length > 0) {
    set({ pending: result, dialog: "missing-assets" });
  } else {
    set({ pending: result, dialog: null });
    invalidateDocument();
  }
}

/** Close any dialog and refresh document-derived views. */
function finishOpen(set: (partial: Partial<WorkspaceFilesState>) => void) {
  set({ dialog: null, pending: null });
  invalidateDocument();
}
