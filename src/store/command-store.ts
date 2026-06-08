/**
 * Frontend facade over the Rust command engine. Every command runs through here
 * so execution, undo/redo, and history jumps all go through the shared `api`
 * command bridge and keep TanStack Query in sync.
 *
 * Holds only UI-side conveniences (recent command ids, last invocation for
 * "repeat last"). The command registry, document, and history live in the core.
 */
import { create } from "zustand";
import { api } from "@/lib/api";
import { queryClient } from "@/lib/query-client";
import { queryKeys } from "@/lib/queries";
import { LAYER_COMMAND_IDS, resolveLayerParams } from "@/lib/layer-commands";
import { IMAGE_COMMAND_IDS, resolveImageParams } from "@/lib/image-commands";
import { EXPORT_COMMAND_IDS, resolveExportParams } from "@/lib/export-commands";
import { useUIStore } from "@/store/ui-store";

const RECENT_LIMIT = 6;

type Invocation = { id: string; parameters: Record<string, unknown> };

type CommandState = {
  recentCommandIds: string[];
  lastInvocation: Invocation | null;

  execute: (id: string, parameters?: Record<string, unknown>) => Promise<void>;
  repeatLast: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  jumpTo: (index: number) => Promise<void>;
};

/** Refresh history plus everything a command may have changed. */
function invalidateAfterCommand() {
  queryClient.invalidateQueries({ queryKey: queryKeys.history });
  queryClient.invalidateQueries({ queryKey: queryKeys.workspaceMeta });
  queryClient.invalidateQueries({ queryKey: queryKeys.layers });
  queryClient.invalidateQueries({ queryKey: queryKeys.imageObjects });
  queryClient.invalidateQueries({ queryKey: queryKeys.exportAreas });
  queryClient.invalidateQueries({ queryKey: queryKeys.renderModel });
}

export const useCommandStore = create<CommandState>((set, get) => ({
  recentCommandIds: [],
  lastInvocation: null,

  execute: async (id, parameters = {}) => {
    // Layer and image-object commands need core IDs generated and the target
    // defaulted to the current selection before they reach the engine (see the
    // layer-commands / image-commands resolvers).
    let runParams = parameters;
    let createdLayerId: string | null = null;
    let createdImageObjectId: string | null = null;
    let createdExportAreaId: string | null = null;
    if (LAYER_COMMAND_IDS.has(id)) {
      const resolved = resolveLayerParams(id, parameters, useUIStore.getState().selectedLayerId);
      runParams = resolved.parameters;
      createdLayerId = resolved.createdId;
    } else if (IMAGE_COMMAND_IDS.has(id)) {
      const resolved = resolveImageParams(id, parameters, useUIStore.getState().selectedImageObjectId);
      runParams = resolved.parameters;
      createdImageObjectId = resolved.createdObjectId;
    } else if (EXPORT_COMMAND_IDS.has(id)) {
      // Output-scoped commands always carry an explicit output id from the UI, so
      // the area's primary output default isn't needed here (passed as null).
      const resolved = resolveExportParams(id, parameters, useUIStore.getState().selectedExportAreaId, null);
      runParams = resolved.parameters;
      createdExportAreaId = resolved.createdAreaId;
    }

    await api.runCommand(id, runParams);

    // Keep focus on whatever the command just created so the inspector follows.
    if (createdLayerId) useUIStore.getState().setSelectedLayerId(createdLayerId);
    if (createdImageObjectId) useUIStore.getState().setSelectedImageObjectId(createdImageObjectId);
    if (createdExportAreaId) {
      useUIStore.getState().setSelectedExportAreaId(createdExportAreaId);
      useUIStore.getState().setSideTab("exports");
    }

    set((s) => ({
      lastInvocation: { id, parameters },
      recentCommandIds: [id, ...s.recentCommandIds.filter((c) => c !== id)].slice(0, RECENT_LIMIT),
    }));
    invalidateAfterCommand();
  },

  repeatLast: async () => {
    const last = get().lastInvocation;
    if (!last) return;
    await get().execute(last.id, last.parameters);
  },

  undo: async () => {
    await api.undo();
    invalidateAfterCommand();
  },

  redo: async () => {
    await api.redo();
    invalidateAfterCommand();
  },

  jumpTo: async (index) => {
    await api.jumpToHistory(index);
    invalidateAfterCommand();
  },
}));
