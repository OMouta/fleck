import { Maximize2, Eye, Grid3x3, Ruler, FolderOpen, FilePlus2, Frame } from "lucide-react";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store/ui-store";

export function Canvas() {
  const activeTool = useUIStore((s) => s.activeTool);
  const zoom = useUIStore((s) => s.zoom);
  const setZoom = useUIStore((s) => s.setZoom);
  const showGrid = useUIStore((s) => s.showGrid);
  const toggleGrid = useUIStore((s) => s.toggleGrid);
  const showRulers = useUIStore((s) => s.showRulers);
  const toggleRulers = useUIStore((s) => s.toggleRulers);
  const preview = useUIStore((s) => s.previewExport);
  const togglePreview = useUIStore((s) => s.togglePreviewExport);

  return (
    <div className="relative flex-1 overflow-hidden bg-background" aria-label="Workspace canvas">
      {/* Infinite-canvas grid */}
      <div className={cn("absolute inset-0", showGrid && "fleck-grid")} />

      {/* Empty workspace state — no document loaded yet */}
      <div className="absolute inset-0 flex items-center justify-center p-6">
        <div className="flex max-w-sm flex-col items-center gap-5 text-center">
          <div>
            <h1 className="text-lg font-semibold tracking-tight text-foreground">Untitled workspace</h1>
            <p className="mt-1 text-[13px] text-muted-foreground">
              Open an image or start a new workspace to begin. Your canvas is empty.
            </p>
          </div>
          <div className="flex flex-wrap items-center justify-center gap-2">
            <EmptyAction icon={FolderOpen} label="Open image" onClick={() => api.openImage()} primary />
            <EmptyAction icon={FilePlus2} label="New workspace" onClick={() => api.newWorkspace()} />
            <EmptyAction icon={Frame} label="Create export area" onClick={() => api.createExportArea()} />
          </div>
        </div>
      </div>

      {/* Active tool indicator (top-left) */}
      <div className="pointer-events-none absolute left-3 top-3 flex items-center gap-2 rounded-md border border-border bg-card/80 px-2.5 py-1.5 text-xs backdrop-blur-sm animate-in-fade">
        <span className="size-1.5 rounded-full bg-primary" />
        <span className="font-medium capitalize text-foreground">{activeTool.replace("-", " ")}</span>
        <span className="text-muted-foreground">tool active</span>
      </div>

      {/* Floating canvas controls (bottom-center) */}
      <div className="absolute bottom-3 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-lg border border-border bg-card/90 p-1 shadow-lg backdrop-blur-sm">
        <ControlToggle active={showGrid} onClick={toggleGrid} label="Toggle pixel grid">
          <Grid3x3 className="size-4" />
        </ControlToggle>
        <ControlToggle active={showRulers} onClick={toggleRulers} label="Toggle rulers">
          <Ruler className="size-4" />
        </ControlToggle>
        <ControlToggle active={preview} onClick={togglePreview} label="Preview export area">
          <Eye className="size-4" />
        </ControlToggle>

        <div className="mx-1 h-5 w-px bg-border" />

        <button
          onClick={() => setZoom(zoom - 25)}
          className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          aria-label="Zoom out"
        >
          <span className="text-lg leading-none">−</span>
        </button>
        <button
          onClick={() => setZoom(100)}
          className="min-w-[52px] rounded-md px-1.5 py-1 font-mono text-xs text-foreground transition-colors hover:bg-secondary"
          title="Reset to 100%"
        >
          {zoom}%
        </button>
        <button
          onClick={() => setZoom(zoom + 25)}
          className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          aria-label="Zoom in"
        >
          <span className="text-lg leading-none">+</span>
        </button>

        <div className="mx-1 h-5 w-px bg-border" />

        <button
          onClick={() => setZoom(100)}
          className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          aria-label="Zoom to fit"
        >
          <Maximize2 className="size-4" />
        </button>
      </div>
    </div>
  );
}

function EmptyAction({
  icon: Icon,
  label,
  onClick,
  primary = false,
}: {
  icon: typeof FolderOpen;
  label: string;
  onClick: () => void;
  primary?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex h-8 items-center gap-1.5 rounded-md px-3 text-[13px] font-medium transition-colors",
        primary
          ? "bg-primary text-primary-foreground hover:bg-primary/90"
          : "border border-border text-foreground hover:bg-secondary",
      )}
    >
      <Icon className="size-4" />
      {label}
    </button>
  );
}

function ControlToggle({
  children,
  active,
  onClick,
  label,
}: {
  children: React.ReactNode;
  active: boolean;
  onClick: () => void;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      aria-label={label}
      title={label}
      aria-pressed={active}
      className={cn(
        "flex size-7 items-center justify-center rounded-md transition-colors",
        active ? "bg-primary/15 text-primary" : "text-muted-foreground hover:bg-secondary hover:text-foreground",
      )}
    >
      {children}
    </button>
  );
}
