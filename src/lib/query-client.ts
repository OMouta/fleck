import { QueryClient } from "@tanstack/react-query";

/**
 * Single query client coordinating async access to Rust-owned document state.
 * Exported as a module singleton so imperative flows (e.g. the workspace-file
 * store) can invalidate queries after a backend mutation without prop-drilling.
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5_000,
      refetchOnWindowFocus: false,
    },
  },
});
