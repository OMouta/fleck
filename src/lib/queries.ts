/**
 * TanStack Query hooks that coordinate async access to backend (Rust-owned) state.
 *
 * Components read document state exclusively through these hooks and mutate it
 * through the mutations below, which invalidate the relevant queries. No document
 * array is ever held as React-owned state.
 */
import { useQuery } from "@tanstack/react-query";
import { api } from "./api";

export const queryKeys = {
  workspaceMeta: ["workspace", "meta"] as const,
  layers: ["document", "layers"] as const,
  exportAreas: ["document", "export-areas"] as const,
  history: ["document", "history"] as const,
  recentFiles: ["workspace", "recent-files"] as const,
  commands: ["commands", "definitions"] as const,
  historyJumpSupported: ["commands", "history-jump-supported"] as const,
  renderModel: ["document", "render-model"] as const,
};

export function useWorkspaceMeta() {
  return useQuery({ queryKey: queryKeys.workspaceMeta, queryFn: api.getWorkspaceMeta });
}

export function useLayers() {
  return useQuery({ queryKey: queryKeys.layers, queryFn: api.getLayers });
}

export function useExportAreas() {
  return useQuery({ queryKey: queryKeys.exportAreas, queryFn: api.getExportAreas });
}

export function useHistory() {
  return useQuery({ queryKey: queryKeys.history, queryFn: api.getHistory });
}

export function useRecentFiles() {
  return useQuery({ queryKey: queryKeys.recentFiles, queryFn: api.getRecentFiles });
}

export function useCommands() {
  return useQuery({ queryKey: queryKeys.commands, queryFn: api.getCommands, staleTime: Infinity });
}

export function useRenderModel() {
  return useQuery({ queryKey: queryKeys.renderModel, queryFn: api.getRenderModel });
}

export function useHistoryJumpSupported() {
  return useQuery({
    queryKey: queryKeys.historyJumpSupported,
    queryFn: api.supportsHistoryJump,
    staleTime: Infinity,
  });
}
