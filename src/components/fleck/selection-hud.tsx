import { type LucideIcon } from "lucide-react";
import {
  Maximize2,
  Minimize2,
  Feather,
  FlipVertical2,
  Copy,
  Trash2,
  Layers,
  Frame,
  FileDown,
  SquareDashed,
} from "lucide-react";
import type { Rect } from "@/lib/fleck-data";
import { useRenderModel } from "@/lib/queries";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store/ui-store";
import { useCommandStore } from "@/store/command-store";

/**
 * Selection HUD: floating action card on the canvas while a selection mask is
 * active. Doubles as a compact inspector (id + bounds) and the keyboard-free
 * surface for `selection.*` edits — keeping selection state visible in canvas
 * and inspector contexts (REQ-038 / REQ-045).
 */
export function SelectionHUD() {
  const activeId = useUIStore((s) => s.activeSelectionId);
  const setActiveId = useUIStore((s) => s.setActiveSelectionId);
  const execute = useCommandStore((s) => s.execute);
  const { data: model } = useRenderModel();

  const selection = activeId ? model?.selections.find((s) => s.id === activeId) : null;
  if (!activeId || !selection) return null;

  const rect: Rect = selection.rect;

  const run = (commandId: string, parameters: Record<string, unknown> = {}) =>
    execute(commandId, { id: activeId, ...parameters });

  return (
    <div
      onPointerDown={(e) => e.stopPropagation()}
      className="pointer-events-auto absolute left-3 top-14 flex max-w-[26rem] flex-col gap-2 rounded-lg border border-border bg-card/90 p-2 shadow-lg backdrop-blur-sm"
      aria-label="Selection"
      role="region"
    >
      <div className="flex items-center justify-between gap-3 px-1">
        <div className="flex min-w-0 items-center gap-1.5 text-[11px]">
          <SquareDashed className="size-3.5 text-primary" />
          <span className="truncate font-mono text-foreground" title={activeId}>
            {activeId}
          </span>
        </div>
        <button
          onClick={() => setActiveId(null)}
          className="rounded text-[10px] text-muted-foreground transition-colors hover:text-foreground"
          aria-label="Deselect"
        >
          Deselect
        </button>
      </div>
      <div className="flex items-center gap-2 px-1 font-mono text-[10px] text-muted-foreground">
        <span>
          {Math.round(rect.width)} × {Math.round(rect.height)} px
        </span>
        <span>·</span>
        <span>
          {Math.round(rect.x)}, {Math.round(rect.y)}
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-0.5 border-t border-border pt-1.5">
        <Action icon={Maximize2} label="Expand 1 px" onClick={() => run("selection.expand", { amount: 1 })} />
        <Action icon={Minimize2} label="Contract 1 px" onClick={() => run("selection.contract", { amount: 1 })} />
        <Action icon={Feather} label="Feather 2 px" onClick={() => run("selection.feather", { radius: 2 })} />
        <Action icon={FlipVertical2} label="Invert" onClick={() => run("selection.invert")} />
        <Divider />
        <Action icon={Copy} label="Copy" shortcut="⌘C" onClick={() => run("selection.copy")} />
        <Action
          icon={Layers}
          label="Layer from selection"
          onClick={() => run("selection.layer_from_selection")}
        />
        <Action
          icon={Frame}
          label="Export area from selection"
          onClick={() => run("selection.export_area_from_selection")}
        />
        <Action icon={FileDown} label="Export selection" onClick={() => run("selection.direct_export")} />
        <Divider />
        <Action
          icon={Trash2}
          label="Delete selection"
          shortcut="Del"
          destructive
          onClick={() => run("selection.delete")}
        />
      </div>
    </div>
  );
}

function Action({
  icon: Icon,
  label,
  shortcut,
  destructive,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  shortcut?: string;
  destructive?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={shortcut ? `${label} · ${shortcut}` : label}
      aria-label={label}
      className={cn(
        "flex size-7 items-center justify-center rounded-md transition-colors",
        destructive
          ? "text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
          : "text-muted-foreground hover:bg-secondary hover:text-foreground",
      )}
    >
      <Icon className="size-3.5" />
    </button>
  );
}

function Divider() {
  return <span className="mx-0.5 h-5 w-px shrink-0 bg-border" aria-hidden="true" />;
}
