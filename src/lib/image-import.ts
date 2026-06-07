/**
 * High-level image import flows.
 *
 * Each flow obtains an image through a native hook (file picker, clipboard,
 * drag/drop) and then performs the placement through the undoable `image.*`
 * command engine, so imports land in history and select the new object. Kept
 * separate from the store-free `image-commands.ts` resolver to avoid an import
 * cycle (these depend on the command store).
 */
import { api } from "./api";
import { useCommandStore } from "@/store/command-store";
import { useUIStore } from "@/store/ui-store";

function revealImagesPanel() {
  useUIStore.getState().setSideTab("images");
}

/** Open a native picker and place the chosen file as a linked image object. */
export async function openImageFlow(): Promise<void> {
  const path = await api.pickImageFile();
  if (!path) return; // user cancelled the picker
  await useCommandStore.getState().execute("image.import_linked", { path });
  revealImagesPanel();
}

/** Paste an image from the clipboard as an embedded image object. */
export async function pasteImageFlow(): Promise<void> {
  const acquired = await api.acquireClipboardAsset();
  if (!acquired) return; // clipboard had no image
  await useCommandStore.getState().execute("image.import_clipboard", {
    asset_id: acquired.assetId,
    name: acquired.name,
  });
  revealImagesPanel();
}

/** Place an image file dropped onto the workspace as an embedded image object. */
export async function dropImageFlow(name: string): Promise<void> {
  const acquired = await api.acquireDroppedAsset(name);
  if (!acquired) return;
  await useCommandStore.getState().execute("image.import_drag_drop", {
    asset_id: acquired.assetId,
    name: acquired.name,
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
