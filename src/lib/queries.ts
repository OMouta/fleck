/**
 * TanStack Query hooks that coordinate async access to backend (Rust-owned) state.
 *
 * Components read document state exclusively through these hooks and mutate it
 * through the mutations below, which invalidate the relevant queries. No document
 * array is ever held as React-owned state.
 */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./api";

export const queryKeys = {
  workspaceMeta: ["workspace", "meta"] as const,
  layers: ["document", "layers"] as const,
  exportAreas: ["document", "export-areas"] as const,
  history: ["document", "history"] as const,
  recentFiles: ["workspace", "recent-files"] as const,
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

export function useToggleLayerVisibility() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, visible }: { id: string; visible: boolean }) => api.setLayerVisibility(id, visible),
    onSuccess: () => qc.invalidateQueries({ queryKey: queryKeys.layers }),
  });
}

export function useToggleLayerLocked() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, locked }: { id: string; locked: boolean }) => api.setLayerLocked(id, locked),
    onSuccess: () => qc.invalidateQueries({ queryKey: queryKeys.layers }),
  });
}

export function useSaveWorkspace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.saveWorkspace(),
    onSuccess: () => qc.invalidateQueries({ queryKey: queryKeys.workspaceMeta }),
  });
}
