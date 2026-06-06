import * as Dialog from "@radix-ui/react-dialog";
import { AlertTriangle, FileWarning, FolderSearch, ImageOff } from "lucide-react";
import type { LoadWarning } from "@/lib/fleck-data";
import { useWorkspaceFilesStore } from "@/store/workspace-files-store";

/**
 * Follow-up dialogs after opening a workspace: unsupported newer versions and
 * missing linked assets. Both are driven entirely by structured data the backend
 * returned — the UI never inspects the file itself.
 */
export function WorkspaceDialogs() {
  const dialog = useWorkspaceFilesStore((s) => s.dialog);
  const pending = useWorkspaceFilesStore((s) => s.pending);
  const acceptNewerVersion = useWorkspaceFilesStore((s) => s.acceptNewerVersion);
  const relinkAsset = useWorkspaceFilesStore((s) => s.relinkAsset);
  const dismissDialog = useWorkspaceFilesStore((s) => s.dismissDialog);

  const open = dialog !== null;
  const onOpenChange = (next: boolean) => {
    if (!next) dismissDialog();
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-background/70 backdrop-blur-sm animate-in-fade" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-[calc(100%-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-xl border border-border bg-popover shadow-2xl outline-none animate-in-pop">
          {dialog === "newer-version" && pending && (
            <NewerVersionBody
              fileName={pending.name}
              warnings={pending.warnings}
              onOpenReadOnly={acceptNewerVersion}
              onCancel={dismissDialog}
            />
          )}
          {dialog === "missing-assets" && pending && (
            <MissingAssetsBody
              fileName={pending.name}
              assets={pending.missingAssets}
              onRelink={relinkAsset}
              onContinue={dismissDialog}
            />
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function warningText(w: LoadWarning): string {
  switch (w.kind) {
    case "newer-file":
      return `File format v${w.found} (this build supports up to v${w.supported}).`;
    case "newer-workspace":
      return `Workspace format v${w.found} (this build supports up to v${w.supported}).`;
    case "migrated":
      return `Migrated from format v${w.from} to v${w.to}.`;
  }
}

function NewerVersionBody({
  fileName,
  warnings,
  onOpenReadOnly,
  onCancel,
}: {
  fileName: string;
  warnings: LoadWarning[];
  onOpenReadOnly: () => void;
  onCancel: () => void;
}) {
  const newer = warnings.filter((w) => w.kind === "newer-file" || w.kind === "newer-workspace");
  return (
    <div>
      <div className="flex items-start gap-3 border-b border-border p-4">
        <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-warning/15 text-warning">
          <FileWarning className="size-5" />
        </span>
        <div className="min-w-0">
          <Dialog.Title className="text-sm font-semibold text-foreground">
            Created with a newer version of Fleck
          </Dialog.Title>
          <Dialog.Description className="mt-1 text-[13px] text-muted-foreground">
            <span className="font-mono">{fileName}</span> uses features this build doesn’t support yet. Opening it
            read-only avoids overwriting data you can’t edit here.
          </Dialog.Description>
        </div>
      </div>

      <ul className="space-y-1.5 p-4">
        {newer.map((w, i) => (
          <li key={i} className="flex items-center gap-2 text-[12px] text-foreground">
            <AlertTriangle className="size-3.5 shrink-0 text-warning" />
            {warningText(w)}
          </li>
        ))}
      </ul>

      <div className="flex items-center justify-end gap-2 border-t border-border p-3">
        <button
          onClick={onCancel}
          className="h-8 rounded-md px-3 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
        >
          Cancel
        </button>
        <button
          onClick={onOpenReadOnly}
          className="h-8 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground transition-colors hover:bg-primary/90"
        >
          Open read-only
        </button>
      </div>
    </div>
  );
}

function MissingAssetsBody({
  fileName,
  assets,
  onRelink,
  onContinue,
}: {
  fileName: string;
  assets: { assetId: string; name: string; path: string; resolvedPath: string }[];
  onRelink: (assetId: string) => void;
  onContinue: () => void;
}) {
  return (
    <div>
      <div className="flex items-start gap-3 border-b border-border p-4">
        <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-warning/15 text-warning">
          <ImageOff className="size-5" />
        </span>
        <div className="min-w-0">
          <Dialog.Title className="text-sm font-semibold text-foreground">
            {assets.length} linked asset{assets.length === 1 ? "" : "s"} couldn’t be found
          </Dialog.Title>
          <Dialog.Description className="mt-1 text-[13px] text-muted-foreground">
            <span className="font-mono">{fileName}</span> links to files that aren’t at their saved locations. Relink
            them, or continue and relink later.
          </Dialog.Description>
        </div>
      </div>

      <ul className="max-h-64 space-y-1.5 overflow-y-auto p-3">
        {assets.map((asset) => (
          <li
            key={asset.assetId}
            className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2"
          >
            <div className="min-w-0 flex-1">
              <p className="truncate text-[13px] text-foreground">{asset.name}</p>
              <p className="truncate font-mono text-[10px] text-muted-foreground" title={asset.resolvedPath}>
                {asset.resolvedPath}
              </p>
            </div>
            <button
              onClick={() => onRelink(asset.assetId)}
              className="flex h-7 shrink-0 items-center gap-1.5 rounded-md border border-border px-2.5 text-[12px] text-foreground transition-colors hover:bg-secondary"
            >
              <FolderSearch className="size-3.5" />
              Locate…
            </button>
          </li>
        ))}
      </ul>

      <div className="flex items-center justify-end gap-2 border-t border-border p-3">
        <button
          onClick={onContinue}
          className="h-8 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground transition-colors hover:bg-primary/90"
        >
          Continue without relinking
        </button>
      </div>
    </div>
  );
}
