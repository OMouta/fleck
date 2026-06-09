import { useEffect, useMemo, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import {
  Search,
  FolderOpen,
  Save,
  FileDown,
  FileStack,
  Frame,
  Wand2,
  Crop,
  Maximize,
  Palette,
  Grid3x3,
  Copy,
  Layers,
  Eye,
  Wrench,
  SquareDashed,
  ImageIcon,
  History as HistoryIcon,
  ChevronLeft,
  CornerDownLeft,
  type LucideIcon,
} from "lucide-react";
import type { CommandDefinition, CommandGroup } from "@/lib/fleck-data";
import { useCommands } from "@/lib/queries";
import { bestFuzzyScore } from "@/lib/fuzzy";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store/ui-store";
import { useCommandStore } from "@/store/command-store";

const GROUP_LABEL: Record<CommandGroup, string> = {
  recipe: "Recipes",
  export: "Export",
  layer: "Layers",
  image_object: "Images",
  selection: "Selection",
  view: "View",
  workspace: "Workspace",
  tool: "Tools",
};

const GROUP_ORDER: CommandGroup[] = [
  "recipe",
  "export",
  "layer",
  "image_object",
  "selection",
  "view",
  "workspace",
  "tool",
];

const GROUP_ICON: Record<CommandGroup, LucideIcon> = {
  workspace: FolderOpen,
  layer: Layers,
  image_object: ImageIcon,
  selection: SquareDashed,
  export: FileDown,
  recipe: Wand2,
  view: Eye,
  tool: Wrench,
};

// Per-command icon overrides for a more legible list.
const COMMAND_ICON: Record<string, LucideIcon> = {
  "workspace.save": Save,
  "workspace.open": FolderOpen,
  "image.open": ImageIcon,
  "export.all": FileStack,
  "export.area-selected": FileDown,
  "export.copy-base64": Copy,
  "export.copy-markdown": Copy,
  "selection.crop": Crop,
  "edit.resize-canvas": Maximize,
  "edit.brightness-contrast": Palette,
  "view.toggle-pixel-grid": Grid3x3,
  "view.zoom-area": Frame,
  "recipe.favicon": Grid3x3,
  "recipe.app-icon": FileStack,
};

function iconFor(def: CommandDefinition): LucideIcon {
  return COMMAND_ICON[def.id] ?? GROUP_ICON[def.group];
}

type Section = { group: CommandGroup | "recent"; label: string; items: CommandDefinition[] };

export function CommandPalette() {
  const open = useUIStore((s) => s.paletteOpen);
  const setOpen = useUIStore((s) => s.setPaletteOpen);
  const { data: commands = [] } = useCommands();

  const selectedLayerId = useUIStore((s) => s.selectedLayerId);
  const selectedImageObjectId = useUIStore((s) => s.selectedImageObjectId);
  const sideTab = useUIStore((s) => s.sideTab);
  const selectedAreaId = useUIStore((s) => s.selectedAreaId);

  const recentIds = useCommandStore((s) => s.recentCommandIds);
  const lastInvocation = useCommandStore((s) => s.lastInvocation);
  const execute = useCommandStore((s) => s.execute);
  const repeatLast = useCommandStore((s) => s.repeatLast);

  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [promptFor, setPromptFor] = useState<CommandDefinition | null>(null);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActive(0);
      setPromptFor(null);
    }
  }, [open]);

  // Boost commands relevant to what the user is currently focused on.
  const contextBoost = useMemo(() => {
    return (group: CommandGroup): number => {
      if (group === "layer" && selectedLayerId) return 6;
      if (group === "image_object" && (selectedImageObjectId || sideTab === "images")) return 6;
      if (group === "export" && (selectedAreaId || sideTab === "exports")) return 6;
      return 0;
    };
  }, [selectedLayerId, selectedImageObjectId, selectedAreaId, sideTab]);

  const byId = useMemo(() => new Map(commands.map((c) => [c.id, c])), [commands]);
  const lastDef = lastInvocation ? byId.get(lastInvocation.id) : undefined;

  const sections = useMemo<Section[]>(() => {
    const q = query.trim();

    if (q === "") {
      const recent = recentIds.map((id) => byId.get(id)).filter((c): c is CommandDefinition => !!c);
      const recentSet = new Set(recent.map((c) => c.id));
      const out: Section[] = [];
      if (recent.length) out.push({ group: "recent", label: "Recent", items: recent });

      const groups = [...GROUP_ORDER].sort((a, b) => contextBoost(b) - contextBoost(a));
      for (const group of groups) {
        const items = commands
          .filter((c) => c.group === group && !recentSet.has(c.id))
          .sort((a, b) => a.label.localeCompare(b.label));
        if (items.length) out.push({ group, label: GROUP_LABEL[group], items });
      }
      return out;
    }

    // Query present: fuzzy score over label + aliases + description, with context boost.
    const scored = commands
      .map((c) => {
        const base = bestFuzzyScore(q, [c.label, ...c.aliases, c.description]);
        return base === null ? null : { c, score: base + contextBoost(c.group) };
      })
      .filter((x): x is { c: CommandDefinition; score: number } => x !== null);

    const groupBest = new Map<CommandGroup, number>();
    for (const { c, score } of scored) {
      groupBest.set(c.group, Math.max(groupBest.get(c.group) ?? -Infinity, score));
    }
    const orderedGroups = [...groupBest.keys()].sort((a, b) => groupBest.get(b)! - groupBest.get(a)!);

    return orderedGroups.map((group) => ({
      group,
      label: GROUP_LABEL[group],
      items: scored
        .filter((x) => x.c.group === group)
        .sort((a, b) => b.score - a.score)
        .map((x) => x.c),
    }));
  }, [query, commands, recentIds, byId, contextBoost]);

  // Flat list of runnable rows for keyboard navigation. The repeat-last row (only
  // shown with no query) sits at index 0.
  const showRepeat = query.trim() === "" && !!lastDef;
  const flat = useMemo(() => {
    const rows = sections.flatMap((s) => s.items);
    return showRepeat && lastDef ? [lastDef, ...rows] : rows;
  }, [sections, showRepeat, lastDef]);

  useEffect(() => {
    if (active >= flat.length) setActive(Math.max(0, flat.length - 1));
  }, [flat.length, active]);

  const run = (def: CommandDefinition) => {
    if (def.parameterPrompts.length > 0) {
      setPromptFor(def);
      return;
    }
    execute(def.id);
    setOpen(false);
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (promptFor) return; // prompt form handles its own keys
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, flat.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const def = flat[active];
      if (showRepeat && active === 0 && lastDef) {
        repeatLast();
        setOpen(false);
        return;
      }
      if (def) run(def);
    }
  };

  let runnableIndex = -1;

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-background/70 backdrop-blur-sm animate-in-fade" />
        <Dialog.Content
          onKeyDown={onKeyDown}
          className="fixed left-1/2 top-[12vh] z-50 w-[calc(100%-2rem)] max-w-xl -translate-x-1/2 overflow-hidden rounded-xl border border-border bg-popover shadow-2xl outline-none animate-in-pop"
        >
          <Dialog.Title className="sr-only">Command palette</Dialog.Title>
          <Dialog.Description className="sr-only">Search and run commands and recipes</Dialog.Description>

          {promptFor ? (
            <ParameterForm
              command={promptFor}
              onBack={() => setPromptFor(null)}
              onSubmit={(params) => {
                execute(promptFor.id, params);
                setOpen(false);
              }}
            />
          ) : (
            <>
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
                {flat.length === 0 && (
                  <p className="px-3 py-8 text-center text-sm text-muted-foreground">No commands match “{query}”.</p>
                )}

                {showRepeat && lastDef && (
                  <div className="mb-1">
                    <p className="px-2.5 py-1.5 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
                      Repeat last
                    </p>
                    {(() => {
                      runnableIndex += 1;
                      const idx = runnableIndex;
                      return (
                        <Row
                          icon={HistoryIcon}
                          def={lastDef}
                          active={active === idx}
                          onMouseEnter={() => setActive(idx)}
                          onClick={() => {
                            repeatLast();
                            setOpen(false);
                          }}
                        />
                      );
                    })()}
                  </div>
                )}

                {sections.map((section) => (
                  <div key={section.group} className="mb-1">
                    <p className="px-2.5 py-1.5 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
                      {section.label}
                    </p>
                    {section.items.map((def) => {
                      runnableIndex += 1;
                      const idx = runnableIndex;
                      return (
                        <Row
                          key={section.group + def.id}
                          icon={iconFor(def)}
                          def={def}
                          isRecipe={def.group === "recipe"}
                          active={active === idx}
                          onMouseEnter={() => setActive(idx)}
                          onClick={() => run(def)}
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
            </>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function Row({
  icon: Icon,
  def,
  active,
  isRecipe = false,
  onClick,
  onMouseEnter,
}: {
  icon: LucideIcon;
  def: CommandDefinition;
  active: boolean;
  isRecipe?: boolean;
  onClick: () => void;
  onMouseEnter: () => void;
}) {
  const hasParams = def.parameterPrompts.length > 0;
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
          isRecipe ? "bg-primary/15 text-primary" : "bg-background text-muted-foreground",
        )}
      >
        <Icon className="size-4" />
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-[13px] text-foreground">{def.label}</span>
        <span className="truncate text-[11px] text-muted-foreground">{def.description}</span>
      </span>
      {hasParams && (
        <span className="rounded bg-secondary px-1.5 py-0.5 text-[10px] text-muted-foreground">needs input</span>
      )}
      {isRecipe && (
        <span className="rounded bg-primary/15 px-1.5 py-0.5 text-[10px] font-medium text-primary">recipe</span>
      )}
      {def.shortcut && <kbd className="font-mono text-[11px] text-muted-foreground">{def.shortcut}</kbd>}
    </button>
  );
}

function ParameterForm({
  command,
  onBack,
  onSubmit,
}: {
  command: CommandDefinition;
  onBack: () => void;
  onSubmit: (params: Record<string, unknown>) => void;
}) {
  const [values, setValues] = useState<Record<string, string>>({});
  const firstInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    firstInput.current?.focus();
  }, []);

  const missingRequired = command.parameterPrompts.some(
    (p) => p.required && p.kind !== "boolean" && !(values[p.key] ?? "").trim(),
  );

  const submit = () => {
    if (missingRequired) return;
    const params: Record<string, unknown> = {};
    for (const p of command.parameterPrompts) {
      const raw = values[p.key] ?? "";
      if (p.kind === "number") {
        if (raw.trim() !== "") params[p.key] = Number(raw);
      } else if (p.kind === "boolean") {
        params[p.key] = raw === "true";
      } else if (raw.trim() !== "") {
        params[p.key] = raw;
      }
    }
    onSubmit(params);
  };

  return (
    <div
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          onBack();
        }
      }}
    >
      <div className="flex items-center gap-2 border-b border-border px-3 py-2.5">
        <button
          onClick={onBack}
          className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          aria-label="Back to commands"
        >
          <ChevronLeft className="size-4" />
        </button>
        <div className="min-w-0">
          <p className="truncate text-[13px] font-medium text-foreground">{command.label}</p>
          <p className="truncate text-[11px] text-muted-foreground">{command.description}</p>
        </div>
      </div>

      <div className="space-y-3 p-4">
        {command.parameterPrompts.map((p, i) => (
          <div key={p.key} className="flex flex-col gap-1">
            <label className="text-[11px] text-muted-foreground">
              {p.label}
              {p.required && <span className="ml-1 text-warning">*</span>}
            </label>
            {p.kind === "boolean" ? (
              <label className="flex items-center gap-2 text-[13px] text-foreground">
                <input
                  type="checkbox"
                  checked={values[p.key] === "true"}
                  onChange={(e) => setValues((v) => ({ ...v, [p.key]: e.target.checked ? "true" : "false" }))}
                />
                Enabled
              </label>
            ) : (
              <input
                ref={i === 0 ? firstInput : undefined}
                type={p.kind === "number" ? "number" : "text"}
                value={values[p.key] ?? ""}
                onChange={(e) => setValues((v) => ({ ...v, [p.key]: e.target.value }))}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    submit();
                  }
                }}
                className="h-9 rounded-md border border-border bg-background px-2.5 text-[13px] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
            )}
          </div>
        ))}
      </div>

      <div className="flex items-center justify-between border-t border-border px-3 py-2">
        <span className="text-[11px] text-muted-foreground">
          <Kbd>esc</Kbd> back
        </span>
        <button
          onClick={submit}
          disabled={missingRequired}
          className="flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
        >
          <CornerDownLeft className="size-3.5" />
          Run command
        </button>
      </div>
    </div>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return <kbd className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px]">{children}</kbd>;
}
