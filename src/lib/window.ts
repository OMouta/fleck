/**
 * Thin wrapper over the Tauri window API for the custom titlebar.
 *
 * All calls are no-ops outside the desktop shell (e.g. browser dev), so the same
 * UI runs in both. `@tauri-apps/api/window` is imported lazily so it only loads
 * inside Tauri.
 */
export function isTauri(): boolean {
  return typeof globalThis !== "undefined" && "__TAURI_INTERNALS__" in globalThis;
}

async function currentWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

export const appWindow = {
  async minimize() {
    if (isTauri()) await (await currentWindow()).minimize();
  },
  async toggleMaximize() {
    if (isTauri()) await (await currentWindow()).toggleMaximize();
  },
  async close() {
    if (isTauri()) await (await currentWindow()).close();
  },
  async isMaximized(): Promise<boolean> {
    return isTauri() ? (await currentWindow()).isMaximized() : false;
  },
  /** Subscribe to resize events; returns an unlisten function. */
  async onResized(handler: () => void): Promise<() => void> {
    if (!isTauri()) return () => {};
    return (await currentWindow()).onResized(handler);
  },
};
