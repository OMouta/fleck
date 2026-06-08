import { Maximize, MousePointer2, Crosshair, SquareDashed } from "lucide-react";
import { TOOLS } from "@/lib/fleck-data";
import { useWorkspaceMeta } from "@/lib/queries";
import { useUIStore } from "@/store/ui-store";

export function StatusBar() {
  const activeTool = useUIStore((s) => s.activeTool);
  const activeSelectionId = useUIStore((s) => s.activeSelectionId);
  const { data: meta } = useWorkspaceMeta();
  const tool = TOOLS.find((t) => t.id === activeTool);

  return (
    <footer className="flex h-7 shrink-0 items-center justify-between border-t border-border bg-sidebar px-3 font-mono text-[11px] text-muted-foreground">
      <div className="flex items-center gap-4">
        <span className="flex items-center gap-1.5">
          <MousePointer2 className="size-3" />
          {tool?.name ?? "Move"}
        </span>
        <span className="hidden items-center gap-1.5 sm:flex">
          <Crosshair className="size-3" />
          x 248 · y 132
        </span>
        <span className="hidden md:inline">RGBA 24, 196, 142, 255</span>
      </div>

      <div className="flex items-center gap-4">
        {activeSelectionId && (
          <span className="hidden items-center gap-1.5 text-primary sm:flex" title={activeSelectionId}>
            <SquareDashed className="size-3" />
            {activeSelectionId}
          </span>
        )}
        <span className="hidden sm:inline">
          {meta?.layerCount ?? 0} layers · {meta?.selectedCount ?? 0} selections
        </span>
        <span className="flex items-center gap-1.5">
          <Maximize className="size-3" />
          {meta?.canvasSize ?? "0 × 0 px"}
        </span>
        <span className="flex items-center gap-1.5 text-primary">
          <span className="size-1.5 rounded-full bg-primary" />
          Local · {meta?.dirty ? "unsaved" : "saved"}
        </span>
      </div>
    </footer>
  );
}
