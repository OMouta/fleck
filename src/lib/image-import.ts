/**
 * High-level image import flows.
 *
 * Each flow obtains an image through a native hook (file picker, clipboard,
 * drag/drop) and then performs the placement through the undoable `image.*`
 * command engine, so imports land in history and select the new object.
 *
 * IDs (`asset_id`, `object_id`) for new objects are generated here so the core
 * command parameters carry everything the Rust handler needs. Keep this in
 * sync with `image_import_prompts` / `image_place_existing_prompts` in
 * `fleck-core/src/command.rs`.
 */
import { api } from "./api";
import { useCommandStore } from "@/store/command-store";
import { useUIStore } from "@/store/ui-store";

function revealImagesPanel() {
  useUIStore.getState().setSideTab("images");
}

let idCounter = 0;
function freshId(prefix: string): string {
  idCounter += 1;
  return `${prefix}-${Date.now().toString(36)}-${idCounter.toString(36)}`;
}

function fileStem(path: string): string {
  const base = path.split(/[\\/]/).pop() ?? path;
  const dot = base.lastIndexOf(".");
  return dot > 0 ? base.slice(0, dot) : base;
}

/** Open a native picker and place the chosen file as a linked image object. */
export async function openImageFlow(): Promise<void> {
  const path = await api.pickImageFile();
  if (!path) return; // user cancelled the picker
  await useCommandStore.getState().execute("image.import_linked", {
    path,
    asset_id: freshId("asset"),
    object_id: freshId("img"),
    name: fileStem(path),
  });
  revealImagesPanel();
}

/** Paste an image from the clipboard as an embedded image object. */
export async function pasteImageFlow(): Promise<void> {
  const acquired = await api.acquireClipboardAsset();
  if (!acquired) return; // clipboard had no image (or backend stubbed)
  await useCommandStore.getState().execute("image.import_clipboard", {
    asset_id: acquired.assetId,
    object_id: freshId("img"),
    name: fileStem(acquired.name),
  });
  revealImagesPanel();
}

/** Place an image file dropped onto the workspace as an embedded image object. */
export async function dropImageFlow(name: string): Promise<void> {
  const acquired = await api.acquireDroppedAsset(name);
  if (!acquired) return;
  await useCommandStore.getState().execute("image.import_drag_drop", {
    asset_id: acquired.assetId,
    object_id: freshId("img"),
    name: fileStem(acquired.name),
  });
  revealImagesPanel();
}

/** Pick a new source image and replace an object's source, preserving its settings. */
export async function replaceImageFlow(objectId: string): Promise<void> {
  const assetId = await api.acquireReplacementAsset();
  if (!assetId) return;
  await useCommandStore.getState().execute("image.replace_source", {
    object_id: objectId,
    asset_id: assetId,
  });
}

/** Reveal a linked image object's source file in the OS file manager. */
export async function revealImageSourceFlow(objectId: string): Promise<void> {
  await api.revealImageSource(objectId);
}
