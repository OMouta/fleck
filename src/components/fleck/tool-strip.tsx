import { TOOLS } from "@/lib/fleck-data";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store/ui-store";

export function ToolStrip() {
  const active = useUIStore((s) => s.activeTool);
  const onSelect = useUIStore((s) => s.setActiveTool);

  return (
    <aside
      className="flex w-12 shrink-0 flex-col items-center gap-1 border-r border-border bg-sidebar py-2"
      aria-label="Tool strip"
    >
      {TOOLS.map((tool) => {
        const Icon = tool.icon;
        const isActive = active === tool.id;
        // Insert a divider before pan/zoom navigation tools
        const divider = tool.id === "pan";
        return (
          <div key={tool.id} className="contents">
            {divider && <div className="my-1 h-px w-6 bg-border" />}
            <button
              onClick={() => onSelect(tool.id)}
              className={cn(
                "group relative flex size-9 items-center justify-center rounded-md transition-all duration-150 focus-visible:ring-2 focus-visible:ring-ring outline-none",
                isActive
                  ? "bg-primary/15 text-primary ring-1 ring-primary/40"
                  : "text-muted-foreground hover:bg-secondary hover:text-foreground",
              )}
              aria-pressed={isActive}
              aria-label={`${tool.name} tool (${tool.shortcut})`}
            >
              <Icon className="size-[18px]" />
              {/* Unique tooltip: name + purpose + shortcut, no repeated labels */}
              <span className="pointer-events-none absolute left-12 z-50 hidden whitespace-nowrap rounded-md border border-border bg-popover px-2.5 py-1.5 text-left shadow-lg group-hover:block animate-in-fade">
                <span className="flex items-center gap-2">
                  <span className="text-xs font-medium text-popover-foreground">{tool.name}</span>
                  <kbd className="rounded bg-secondary px-1 py-0.5 font-mono text-[10px] text-muted-foreground">
                    {tool.shortcut}
                  </kbd>
                </span>
                <span className="mt-0.5 block text-[11px] text-muted-foreground">{tool.hint}</span>
              </span>
            </button>
          </div>
        );
      })}
    </aside>
  );
}
