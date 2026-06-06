import { Command, Save, Share2, Download, Undo2, Redo2 } from "lucide-react";
import { api } from "@/lib/api";
import { useWorkspaceMeta } from "@/lib/queries";
import { useUIStore } from "@/store/ui-store";
import { useWorkspaceFilesStore } from "@/store/workspace-files-store";
import { FileMenu } from "@/components/fleck/file-menu";
import { WindowControls } from "@/components/fleck/window-controls";

function FleckMark() {
  return (
    <div className="flex size-7 items-center justify-center rounded-md bg-primary/15 ring-1 ring-primary/30">
      {/* Fleck glyph: the five-block "F" from the app icon, theme-aware via currentColor */}
      <svg viewBox="0 0 24 24" className="size-4 text-primary" aria-hidden="true">
        <g fill="currentColor">
          <rect x="3" y="3" width="5" height="5" />
          <rect x="9" y="3" width="8" height="5" />
          <rect x="3" y="9" width="5" height="5" />
          <rect x="9" y="9" width="5" height="5" />
          <rect x="3" y="15" width="5" height="5" />
        </g>
      </svg>
    </div>
  );
}

export function MenuBar() {
  const setPaletteOpen = useUIStore((s) => s.setPaletteOpen);
  const { data: meta } = useWorkspaceMeta();
  const save = useWorkspaceFilesStore((s) => s.save);

  return (
    <header
      data-tauri-drag-region=""
      className="flex h-11 shrink-0 items-center justify-between border-b border-border bg-sidebar px-3"
    >
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <FleckMark />
          <span className="text-sm font-semibold tracking-tight">Fleck</span>
        </div>
        <div className="h-4 w-px bg-border" />
        <nav className="hidden items-center gap-0.5 md:flex" aria-label="Main menu">
          <FileMenu />
          {["Edit", "Layer", "Select", "Export", "View"].map((item) => (
            <button
              key={item}
              className="rounded-md px-2.5 py-1 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
            >
              {item}
            </button>
          ))}
        </nav>
      </div>

      <div data-tauri-drag-region="" className="flex flex-1 items-center justify-center gap-2">
        <div className="pointer-events-none flex items-center gap-1.5">
          <span className="font-mono text-xs text-muted-foreground">{meta?.name ?? "Untitled.fleck"}</span>
          {meta?.dirty && <span className="size-1.5 rounded-full bg-warning" title="Unsaved changes" />}
        </div>
      </div>

      <div className="flex items-center gap-1.5">
        <div className="mr-1 hidden items-center gap-0.5 sm:flex">
          <IconButton label="Undo" shortcut="⌘Z" onClick={() => api.undo()}>
            <Undo2 className="size-4" />
          </IconButton>
          <IconButton label="Redo" shortcut="⌘⇧Z" onClick={() => api.redo()}>
            <Redo2 className="size-4" />
          </IconButton>
        </div>

        <button
          onClick={() => setPaletteOpen(true)}
          className="group flex h-8 items-center gap-2 rounded-md border border-border bg-secondary/60 pl-2.5 pr-1.5 text-[13px] text-muted-foreground transition-colors hover:border-primary/40 hover:text-foreground"
        >
          <Command className="size-3.5" />
          <span className="hidden sm:inline">Run command</span>
          <kbd className="ml-1 rounded bg-background px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground ring-1 ring-border">
            ⌘K
          </kbd>
        </button>

        <IconButton label="Save workspace" shortcut="⌘S" onClick={() => save()}>
          <Save className="size-4" />
        </IconButton>
        <IconButton label="Share .fleck file" onClick={() => api.runCommand("share-workspace")}>
          <Share2 className="size-4" />
        </IconButton>

        <button
          onClick={() => api.exportAll()}
          className="flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground transition-transform active:scale-[0.97]"
        >
          <Download className="size-4" />
          Export all
        </button>

        <WindowControls />
      </div>
    </header>
  );
}

function IconButton({
  children,
  label,
  shortcut,
  onClick,
}: {
  children: React.ReactNode;
  label: string;
  shortcut?: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={shortcut ? `${label} · ${shortcut}` : label}
      aria-label={label}
      className="flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
    >
      {children}
    </button>
  );
}
