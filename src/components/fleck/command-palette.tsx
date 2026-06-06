import { useEffect, useMemo, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import {
  Search,
  FolderOpen,
  Save,
  FileDown,
  FileStack,
  Frame,
  Scissors,
  Wand2,
  Crop,
  Maximize,
  Palette,
  Grid3x3,
  Copy,
  CornerUpRight,
  type LucideIcon,
} from "lucide-react";
import type { CommandItem } from "@/lib/fleck-data";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store/ui-store";

const COMMANDS: CommandItem[] = [
  { id: "open-image", name: "Open image", group: "Workspace", shortcut: "⌘O", icon: FolderOpen },
  { id: "save-workspace", name: "Save workspace", group: "Workspace", shortcut: "⌘S", icon: Save },
  { id: "export-all", name: "Export all areas", group: "Export", shortcut: "⌘⇧E", icon: FileStack },
  { id: "export-selected", name: "Export selected area", group: "Export", shortcut: "⌘E", icon: FileDown },
  { id: "export-area-from-selection", name: "Create export area from selection", group: "Export", icon: Frame },
  { id: "recipe-favicon", name: "Generate favicon pack", group: "Recipe", icon: Grid3x3 },
  { id: "recipe-app-icon", name: "Generate app icon set", group: "Recipe", icon: FileStack },
  { id: "recipe-remove-bg", name: "Remove background and trim", group: "Recipe", icon: Wand2 },
  { id: "trim-transparent", name: "Trim transparent pixels", group: "Edit", icon: Scissors },
  { id: "crop-to-selection", name: "Crop to selection", group: "Edit", shortcut: "⌘⇧X", icon: Crop },
  { id: "resize-canvas", name: "Resize canvas", group: "Edit", icon: Maximize },
  { id: "brightness-contrast", name: "Adjust brightness & contrast", group: "Edit", icon: Palette },
  { id: "toggle-pixel-grid", name: "Toggle pixel grid", group: "View", shortcut: "⌘'", icon: Grid3x3 },
  { id: "zoom-export-area", name: "Zoom to export area", group: "View", icon: Frame },
  { id: "copy-base64", name: "Copy as Base64", group: "Export", icon: Copy },
  { id: "copy-markdown", name: "Copy as Markdown image", group: "Export", icon: CornerUpRight },
];

const GROUP_ORDER: CommandItem["group"][] = ["Recipe", "Export", "Edit", "Workspace", "View"];

export function CommandPalette() {
  const open = useUIStore((s) => s.paletteOpen);
  const setOpen = useUIStore((s) => s.setPaletteOpen);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);

  const results = useMemo(() => {
    const q = query.toLowerCase().trim();
    return q ? COMMANDS.filter((c) => c.name.toLowerCase().includes(q)) : COMMANDS;
  }, [query]);

  const grouped = useMemo(() => {
    const map = new Map<string, CommandItem[]>();
    for (const c of results) {
      if (!map.has(c.group)) map.set(c.group, []);
      map.get(c.group)!.push(c);
    }
    return GROUP_ORDER.filter((g) => map.has(g)).map((g) => ({ group: g, items: map.get(g)! }));
  }, [results]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActive(0);
    }
  }, [open]);

  const run = (cmd: CommandItem) => {
    api.runCommand(cmd.id);
    setOpen(false);
  };

  const onListKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const cmd = results[active];
      if (cmd) run(cmd);
    }
  };

  let flatIndex = -1;

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-background/70 backdrop-blur-sm animate-in-fade" />
        <Dialog.Content
          onKeyDown={onListKeyDown}
          className="fixed left-1/2 top-[12vh] z-50 w-[calc(100%-2rem)] max-w-xl -translate-x-1/2 overflow-hidden rounded-xl border border-border bg-popover shadow-2xl outline-none animate-in-pop"
        >
          <Dialog.Title className="sr-only">Command palette</Dialog.Title>
          <Dialog.Description className="sr-only">Search commands and recipes</Dialog.Description>

          <div className="flex items-center gap-2.5 border-b border-border px-4">
            <Search className="size-4 shrink-0 text-muted-foreground" />
            <input
              autoFocus
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setActive(0);
              }}
              placeholder="Search commands and recipes…"
              className="h-12 flex-1 bg-transparent text-sm text-foreground outline-none placeholder:text-muted-foreground"
            />
            <kbd className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">esc</kbd>
          </div>

          <div className="max-h-[52vh] overflow-y-auto p-1.5">
            {grouped.length === 0 && (
              <p className="px-3 py-8 text-center text-sm text-muted-foreground">No commands match “{query}”.</p>
            )}
            {grouped.map(({ group, items }) => (
              <div key={group} className="mb-1">
                <p className="px-2.5 py-1.5 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
                  {group}
                </p>
                {items.map((cmd) => {
                  flatIndex += 1;
                  const idx = flatIndex;
                  return (
                    <Row
                      key={cmd.id}
                      icon={cmd.icon}
                      name={cmd.name}
                      shortcut={cmd.shortcut}
                      group={cmd.group}
                      active={active === idx}
                      onMouseEnter={() => setActive(idx)}
                      onClick={() => run(cmd)}
                    />
                  );
                })}
              </div>
            ))}
          </div>

          <div className="flex items-center justify-between border-t border-border px-3 py-2 text-[11px] text-muted-foreground">
            <span className="flex items-center gap-2">
              <Kbd>↑</Kbd>
              <Kbd>↓</Kbd>
              navigate
            </span>
            <span className="flex items-center gap-1.5">
              <Kbd>enter</Kbd>
              run command
            </span>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function Row({
  icon: Icon,
  name,
  shortcut,
  group,
  active,
  onClick,
  onMouseEnter,
}: {
  icon: LucideIcon;
  name: string;
  shortcut?: string;
  group: string;
  active: boolean;
  onClick: () => void;
  onMouseEnter: () => void;
}) {
  return (
    <button
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      className={cn(
        "flex w-full items-center gap-3 rounded-md px-2.5 py-2 text-left transition-colors",
        active ? "bg-secondary" : "hover:bg-secondary/50",
      )}
    >
      <span
        className={cn(
          "flex size-7 shrink-0 items-center justify-center rounded-md",
          group === "Recipe" ? "bg-primary/15 text-primary" : "bg-background text-muted-foreground",
        )}
      >
        <Icon className="size-4" />
      </span>
      <span className="flex-1 text-[13px] text-foreground">{name}</span>
      {group === "Recipe" && (
        <span className="rounded bg-primary/15 px-1.5 py-0.5 text-[10px] font-medium text-primary">recipe</span>
      )}
      {shortcut && <kbd className="font-mono text-[11px] text-muted-foreground">{shortcut}</kbd>}
    </button>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return <kbd className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px]">{children}</kbd>;
}
