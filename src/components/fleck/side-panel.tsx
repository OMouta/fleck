import { useEffect, useRef, useState, type ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Eye,
  EyeOff,
  Lock,
  LockOpen,
  Plus,
  ImageIcon,
  Type,
  Square,
  CircleDashed,
  Layers,
  FileDown,
  AlertTriangle,
  RotateCcw,
  Check,
  ChevronRight,
  ChevronDown,
  Copy,
  Trash2,
  Pencil,
  ArrowUp,
  ArrowDown,
  ArrowDownToLine,
  FolderPlus,
  History as HistoryIcon,
  Undo2,
  Redo2,
} from "lucide-react";
import type { ExportArea, Layer } from "@/lib/fleck-data";
import { api } from "@/lib/api";
import { useExportAreas, useHistory, useHistoryJumpSupported, useLayers } from "@/lib/queries";
import { BLEND_MODES } from "@/lib/layer-commands";
import { cn } from "@/lib/utils";
import { useUIStore, type SideTab } from "@/store/ui-store";
import { useCommandStore } from "@/store/command-store";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";

const KIND_ICON = {
  image: ImageIcon,
  text: Type,
  shape: Square,
  mask: CircleDashed,
  group: Layers,
} as const;

export function SidePanel() {
  const tab = useUIStore((s) => s.sideTab);
  const setTab = useUIStore((s) => s.setSideTab);
  const { data: layers = [] } = useLayers();
  const { data: exportAreas = [] } = useExportAreas();

  return (
    <Tabs
      value={tab}
      onValueChange={(v) => setTab(v as SideTab)}
      asChild
    >
      <aside className="flex w-72 shrink-0 flex-col border-l border-border bg-sidebar" aria-label="Editor panels">
        <TabsList className="border-b border-border p-1.5">
          <TabsTrigger value="layers">
            <Layers className="size-4" />
            Layers
            <TabCount value={layers.length} active={tab === "layers"} />
          </TabsTrigger>
          <TabsTrigger value="exports">
            <FileDown className="size-4" />
            Exports
            <TabCount value={exportAreas.length} active={tab === "exports"} />
          </TabsTrigger>
          <TabsTrigger value="history">
            <HistoryIcon className="size-4" />
            History
          </TabsTrigger>
        </TabsList>

        <TabsContent value="layers">
          <LayersPanel />
        </TabsContent>
        <TabsContent value="exports">
          <ExportsPanel />
        </TabsContent>
        <TabsContent value="history">
          <HistoryPanel />
        </TabsContent>
      </aside>
    </Tabs>
  );
}

function TabCount({ value, active }: { value: number; active: boolean }) {
  return (
    <span
      className={cn(
        "rounded px-1 font-mono text-[10px]",
        active ? "bg-background text-muted-foreground" : "text-muted-foreground",
      )}
    >
      {value}
    </span>
  );
}

/** All layer edits an action surface can request; resolved to core commands below. */
type LayerAction =
  | "rename"
  | "duplicate"
  | "delete"
  | "toggle-visible"
  | "toggle-lock"
  | "move-up"
  | "move-down"
  | "merge-down"
  | "flatten"
  | "group";

function LayersPanel() {
  const { data: layers = [], isLoading } = useLayers();
  const selected = useUIStore((s) => s.selectedLayerId);
  const onSelect = useUIStore((s) => s.setSelectedLayerId);
  const execute = useCommandStore((s) => s.execute);

  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [dragId, setDragId] = useState<string | null>(null);
  // Insertion point (0..layers.length) the dragged row would drop into.
  const [dropIndex, setDropIndex] = useState<number | null>(null);

  const selectedLayer = layers.find((l) => l.id === selected) ?? layers[0];

  // Every layer mutation goes through the command engine so it is undoable and
  // recorded in history (see layer-commands + command-store).
  const handleAction = (action: LayerAction, layer: Layer, index: number) => {
    switch (action) {
      case "rename":
        setRenamingId(layer.id);
        break;
      case "duplicate":
        execute("layer.duplicate", { id: layer.id });
        break;
      case "delete":
        execute("layer.delete", { id: layer.id });
        break;
      case "toggle-visible":
        execute("layer.set_visible", { id: layer.id, visible: !layer.visible });
        break;
      case "toggle-lock":
        execute("layer.set_locked", { id: layer.id, locked: !layer.locked });
        break;
      case "move-up":
        execute("layer.reorder", { id: layer.id, index: index - 1 });
        break;
      case "move-down":
        execute("layer.reorder", { id: layer.id, index: index + 1 });
        break;
      case "merge-down":
        execute("layer.merge_down", { id: layer.id });
        break;
      case "flatten":
        execute("layer.flatten", {});
        break;
      case "group":
        execute("layer.group", { id: layer.id });
        break;
    }
  };

  const commitRename = (layer: Layer, name: string) => {
    setRenamingId(null);
    if (name && name !== layer.name) execute("layer.rename", { id: layer.id, name });
  };

  const finishDrop = () => {
    if (dragId !== null && dropIndex !== null) {
      const from = layers.findIndex((l) => l.id === dragId);
      if (from !== -1) {
        // dropIndex is an insertion point; removing the source shifts later slots.
        let to = dropIndex > from ? dropIndex - 1 : dropIndex;
        to = Math.max(0, Math.min(layers.length - 1, to));
        if (to !== from) execute("layer.reorder", { id: dragId, index: to });
      }
    }
    setDragId(null);
    setDropIndex(null);
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Layer stack</span>
        <button
          onClick={() => execute("layer.create", {})}
          className="flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Add layer"
          aria-label="Add layer"
        >
          <Plus className="size-4" />
        </button>
      </div>

      <div
        className="flex-1 overflow-y-auto px-1.5 pb-2"
        onDragOver={(e) => {
          if (dragId) e.preventDefault();
        }}
        onDrop={(e) => {
          if (dragId) {
            e.preventDefault();
            finishDrop();
          }
        }}
      >
        {isLoading && <p className="px-3 py-4 text-[13px] text-muted-foreground">Loading layers…</p>}
        {!isLoading && layers.length === 0 && (
          <p className="px-3 py-6 text-center text-[13px] text-muted-foreground">
            No layers yet. Add a layer or open an image.
          </p>
        )}
        {layers.map((layer, index) => (
          <LayerRow
            key={layer.id}
            layer={layer}
            index={index}
            count={layers.length}
            selected={selectedLayer?.id === layer.id}
            renaming={renamingId === layer.id}
            dragging={dragId === layer.id}
            dropBefore={dropIndex === index}
            dropAfterLast={index === layers.length - 1 && dropIndex === layers.length}
            onSelect={() => onSelect(layer.id)}
            onAction={(a) => handleAction(a, layer, index)}
            onCommitRename={(name) => commitRename(layer, name)}
            onCancelRename={() => setRenamingId(null)}
            onDragStart={() => setDragId(layer.id)}
            onDragOverRow={(after) => setDropIndex(index + (after ? 1 : 0))}
            onDragEnd={() => {
              setDragId(null);
              setDropIndex(null);
            }}
          />
        ))}
      </div>

      {selectedLayer ? (
        <LayerInspector
          layer={selectedLayer}
          index={layers.findIndex((l) => l.id === selectedLayer.id)}
          count={layers.length}
          onAction={(a) =>
            handleAction(a, selectedLayer, layers.findIndex((l) => l.id === selectedLayer.id))
          }
          onCommitRename={(name) => commitRename(selectedLayer, name)}
          onSetOpacity={(pct) => execute("layer.set_opacity", { id: selectedLayer.id, opacity: pct / 100 })}
          onSetBlend={(value) => execute("layer.set_blend_mode", { id: selectedLayer.id, blend_mode: value })}
        />
      ) : (
        <div className="border-t border-border p-3" aria-label="Inspector">
          <p className="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
          <p className="text-[13px] text-muted-foreground">No selection.</p>
        </div>
      )}
    </div>
  );
}

function LayerRow({
  layer,
  index,
  count,
  selected,
  renaming,
  dragging,
  dropBefore,
  dropAfterLast,
  onSelect,
  onAction,
  onCommitRename,
  onCancelRename,
  onDragStart,
  onDragOverRow,
  onDragEnd,
}: {
  layer: Layer;
  index: number;
  count: number;
  selected: boolean;
  renaming: boolean;
  dragging: boolean;
  dropBefore: boolean;
  dropAfterLast: boolean;
  onSelect: () => void;
  onAction: (action: LayerAction) => void;
  onCommitRename: (name: string) => void;
  onCancelRename: () => void;
  onDragStart: () => void;
  onDragOverRow: (after: boolean) => void;
  onDragEnd: () => void;
}) {
  const Icon = KIND_ICON[layer.kind];

  return (
    <div className="relative">
      {dropBefore && <DropLine />}
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <div
            draggable={!renaming}
            onDragStart={onDragStart}
            onDragEnd={onDragEnd}
            onDragOver={(e) => {
              e.preventDefault();
              const rect = e.currentTarget.getBoundingClientRect();
              onDragOverRow(e.clientY > rect.top + rect.height / 2);
            }}
            className={cn(
              "group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors",
              selected ? "bg-primary/12 ring-1 ring-primary/30" : "hover:bg-secondary/70",
              dragging && "opacity-40",
              !layer.visible && "opacity-50",
            )}
          >
            <button
              onClick={() => onAction("toggle-visible")}
              className="flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:text-foreground"
              aria-label={layer.visible ? `Hide ${layer.name}` : `Show ${layer.name}`}
              aria-pressed={layer.visible}
            >
              {layer.visible ? <Eye className="size-3.5" /> : <EyeOff className="size-3.5" />}
            </button>

            {renaming ? (
              <NameInput
                initial={layer.name}
                autoFocus
                onCommit={onCommitRename}
                onCancel={onCancelRename}
                className="h-6 flex-1 rounded border border-border bg-background px-1.5 text-[13px] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
            ) : (
              <button
                onClick={onSelect}
                onDoubleClick={() => onAction("rename")}
                className="flex flex-1 items-center gap-2 overflow-hidden text-left"
                aria-pressed={selected}
                aria-current={selected}
              >
                <Icon className={cn("size-4 shrink-0", selected ? "text-primary" : "text-muted-foreground")} />
                <span className="flex-1 truncate text-[13px] text-foreground">{layer.name}</span>
                {(!layer.visible || layer.locked) && (
                  <span className="sr-only">
                    {!layer.visible && "hidden"} {layer.locked && "locked"}
                  </span>
                )}
                {layer.opacity < 100 && (
                  <span className="font-mono text-[10px] text-muted-foreground">{layer.opacity}%</span>
                )}
              </button>
            )}

            <button
              onClick={() => onAction("toggle-lock")}
              className={cn(
                "flex size-5 shrink-0 items-center justify-center rounded hover:text-foreground",
                layer.locked ? "text-warning" : "text-transparent group-hover:text-muted-foreground",
              )}
              aria-label={layer.locked ? `Unlock ${layer.name}` : `Lock ${layer.name}`}
              aria-pressed={layer.locked}
            >
              <Lock className="size-3" />
            </button>
          </div>
        </ContextMenuTrigger>

        <LayerMenu layer={layer} index={index} count={count} onAction={onAction} />
      </ContextMenu>
      {dropAfterLast && <DropLine />}
    </div>
  );
}

/** Thin 2px insertion indicator shown between rows while dragging. */
function DropLine() {
  return <div className="pointer-events-none absolute inset-x-1 z-10 h-0.5 -translate-y-px rounded-full bg-primary" />;
}

/** Right-click action list for a layer row. */
function LayerMenu({
  layer,
  index,
  count,
  onAction,
}: {
  layer: Layer;
  index: number;
  count: number;
  onAction: (action: LayerAction) => void;
}) {
  const locked = layer.locked;
  const isLast = index === count - 1;
  return (
    <ContextMenuContent>
      <ContextMenuItem disabled={locked} onSelect={() => onAction("rename")}>
        <Pencil />
        Rename
        <ContextMenuShortcut>F2</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("duplicate")}>
        <Copy />
        Duplicate
        <ContextMenuShortcut>⌘J</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem variant="destructive" disabled={locked} onSelect={() => onAction("delete")}>
        <Trash2 />
        Delete
      </ContextMenuItem>

      <ContextMenuSeparator />
      <ContextMenuItem onSelect={() => onAction("toggle-visible")}>
        {layer.visible ? <EyeOff /> : <Eye />}
        {layer.visible ? "Hide" : "Show"}
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("toggle-lock")}>
        {layer.locked ? <LockOpen /> : <Lock />}
        {layer.locked ? "Unlock" : "Lock"}
      </ContextMenuItem>

      <ContextMenuSeparator />
      <ContextMenuItem disabled={locked || index === 0} onSelect={() => onAction("move-up")}>
        <ArrowUp />
        Move up
      </ContextMenuItem>
      <ContextMenuItem disabled={locked || isLast} onSelect={() => onAction("move-down")}>
        <ArrowDown />
        Move down
      </ContextMenuItem>

      <ContextMenuSeparator />
      <ContextMenuItem disabled={locked || isLast} onSelect={() => onAction("merge-down")}>
        <ArrowDownToLine />
        Merge down
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("flatten")}>
        <Layers />
        Flatten visible
      </ContextMenuItem>
      <ContextMenuItem disabled={locked} onSelect={() => onAction("group")}>
        <FolderPlus />
        Group
      </ContextMenuItem>
    </ContextMenuContent>
  );
}

function LayerInspector({
  layer,
  index,
  count,
  onAction,
  onCommitRename,
  onSetOpacity,
  onSetBlend,
}: {
  layer: Layer;
  index: number;
  count: number;
  onAction: (action: LayerAction) => void;
  onCommitRename: (name: string) => void;
  onSetOpacity: (pct: number) => void;
  onSetBlend: (value: string) => void;
}) {
  const locked = layer.locked;
  return (
    <div className="border-t border-border p-3" aria-label="Inspector">
      <div className="mb-2.5 flex items-center justify-between">
        <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
        {locked && (
          <span className="flex items-center gap-1 text-[10px] font-medium uppercase tracking-wide text-warning">
            <Lock className="size-3" />
            Locked
          </span>
        )}
      </div>

      <div className="space-y-2.5">
        <Field label="Name">
          <NameInput
            // Remount when the layer or its name changes so the field reflects state.
            key={`${layer.id}:${layer.name}`}
            initial={layer.name}
            disabled={locked}
            onCommit={onCommitRename}
            className="h-7 w-full rounded border border-border bg-background px-1.5 text-[13px] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
          />
        </Field>

        <Field label="Opacity">
          <OpacitySlider layer={layer} disabled={locked} onCommit={onSetOpacity} />
        </Field>

        <Field label="Blend">
          <BlendMenu value={layer.blend} disabled={locked} onSelect={onSetBlend} />
        </Field>
      </div>

      <div className="mt-3 flex items-center gap-1.5 border-t border-border pt-3">
        <InspectorButton onClick={() => onAction("duplicate")} icon={Copy} label="Duplicate" />
        <InspectorButton
          onClick={() => onAction("merge-down")}
          icon={ArrowDownToLine}
          label="Merge down"
          disabled={locked || index === count - 1}
        />
        <InspectorButton
          onClick={() => onAction("delete")}
          icon={Trash2}
          label="Delete"
          disabled={locked}
          destructive
        />
      </div>
    </div>
  );
}

function InspectorButton({
  onClick,
  icon: Icon,
  label,
  disabled,
  destructive,
}: {
  onClick: () => void;
  icon: LucideIcon;
  label: string;
  disabled?: boolean;
  destructive?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={label}
      aria-label={label}
      className={cn(
        "flex h-7 flex-1 items-center justify-center gap-1.5 rounded-md text-[11px] transition-colors disabled:pointer-events-none disabled:opacity-40",
        destructive
          ? "text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
          : "text-muted-foreground hover:bg-secondary hover:text-foreground",
      )}
    >
      <Icon className="size-3.5" />
      {label}
    </button>
  );
}

/** Opacity range that previews live but commits a single undoable step on release. */
function OpacitySlider({
  layer,
  disabled,
  onCommit,
}: {
  layer: Layer;
  disabled: boolean;
  onCommit: (pct: number) => void;
}) {
  const [value, setValue] = useState(layer.opacity);
  const dirty = useRef(false);

  useEffect(() => {
    setValue(layer.opacity);
  }, [layer.id, layer.opacity]);

  const commit = () => {
    if (!dirty.current) return;
    dirty.current = false;
    if (value !== layer.opacity) onCommit(value);
  };

  return (
    <div className="flex items-center gap-2">
      <input
        type="range"
        min={0}
        max={100}
        value={value}
        disabled={disabled}
        aria-label="Layer opacity"
        onChange={(e) => {
          dirty.current = true;
          setValue(Number(e.target.value));
        }}
        onPointerUp={commit}
        onKeyUp={commit}
        onBlur={commit}
        className="h-1 flex-1 cursor-pointer accent-primary disabled:cursor-not-allowed disabled:opacity-50"
      />
      <span className="w-9 text-right font-mono text-[11px] text-foreground">{value}%</span>
    </div>
  );
}

function BlendMenu({
  value,
  disabled,
  onSelect,
}: {
  value: Layer["blend"];
  disabled: boolean;
  onSelect: (value: string) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          disabled={disabled}
          className="flex w-full items-center justify-between gap-1 rounded bg-secondary px-1.5 py-1 text-[11px] text-foreground transition-colors hover:bg-secondary/70 disabled:opacity-50"
          aria-label={`Blend mode: ${value}`}
        >
          {value}
          <ChevronDown className="size-3 text-muted-foreground" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="max-h-64 min-w-40 overflow-y-auto">
        {BLEND_MODES.map((m) => (
          <DropdownMenuItem key={m.value} onSelect={() => onSelect(m.value)}>
            {m.label}
            {m.label === value && <Check className="ml-auto size-3.5 text-primary" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/** Inline editable name field shared by layer rows and the inspector. */
function NameInput({
  initial,
  autoFocus,
  disabled,
  onCommit,
  onCancel,
  className,
}: {
  initial: string;
  autoFocus?: boolean;
  disabled?: boolean;
  onCommit: (name: string) => void;
  onCancel?: () => void;
  className?: string;
}) {
  const ref = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus) {
      ref.current?.focus();
      ref.current?.select();
    }
  }, [autoFocus]);

  return (
    <input
      ref={ref}
      defaultValue={initial}
      disabled={disabled}
      onBlur={() => onCommit(ref.current?.value.trim() ?? "")}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Enter") {
          e.preventDefault();
          onCommit(ref.current?.value.trim() ?? "");
        } else if (e.key === "Escape") {
          e.preventDefault();
          if (ref.current) ref.current.value = initial;
          onCancel ? onCancel() : ref.current?.blur();
        }
      }}
      className={className}
    />
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <span className="w-14 shrink-0 text-[11px] text-muted-foreground">{label}</span>
      <div className="flex-1 overflow-hidden">{children}</div>
    </div>
  );
}

function ExportsPanel() {
  const { data: areas = [], isLoading } = useExportAreas();
  const open = useUIStore((s) => s.openExportAreaId);
  const setOpen = useUIStore((s) => s.setOpenExportAreaId);

  return (
    <div className="flex flex-1 flex-col overflow-y-auto p-1.5">
      {isLoading && <p className="px-3 py-4 text-[13px] text-muted-foreground">Loading export areas…</p>}
      {!isLoading && areas.length === 0 && (
        <p className="px-3 py-6 text-center text-[13px] text-muted-foreground">
          No export areas yet. Use the export area tool to mark a region.
        </p>
      )}
      {areas.map((area) => {
        const isOpen = open === area.id;
        return (
          <div key={area.id} className="mb-1 overflow-hidden rounded-md border border-border">
            <button
              onClick={() => setOpen(isOpen ? null : area.id)}
              className="flex w-full items-center gap-2 bg-card px-2.5 py-2 text-left transition-colors hover:bg-secondary/60"
              aria-expanded={isOpen}
            >
              <ChevronRight
                className={cn("size-3.5 shrink-0 text-muted-foreground transition-transform", isOpen && "rotate-90")}
              />
              <span className="flex-1 truncate font-mono text-[13px] text-foreground">{area.name}</span>
              <StatusDot status={area.status} />
              <span className="font-mono text-[10px] text-muted-foreground">{area.dimensions}</span>
            </button>

            {isOpen && <ExportAreaDetails area={area} />}
          </div>
        );
      })}
    </div>
  );
}

function ExportAreaDetails({ area }: { area: ExportArea }) {
  return (
    <div className="border-t border-border bg-background/40 animate-in-fade">
      {area.note && (
        <div className="flex items-center gap-1.5 px-2.5 py-1.5 text-[11px] text-warning">
          <AlertTriangle className="size-3 shrink-0" />
          {area.note}
        </div>
      )}
      {area.outputs.map((out) => (
        <div key={out.id} className="flex items-center gap-2 px-2.5 py-1.5">
          <FileDown className="size-3.5 shrink-0 text-muted-foreground" />
          <span className="flex-1 truncate font-mono text-[12px] text-foreground">{out.filename}</span>
          <span className="font-mono text-[10px] text-muted-foreground">{out.size}</span>
          <span className="w-12 text-right font-mono text-[10px] text-muted-foreground">{out.bytes}</span>
        </div>
      ))}
      <div className="flex items-center gap-1.5 border-t border-border p-1.5">
        <button
          onClick={() => api.exportArea(area.id)}
          className="flex flex-1 items-center justify-center gap-1.5 rounded-md bg-primary py-1.5 text-[12px] font-medium text-primary-foreground transition-transform active:scale-[0.98]"
        >
          <FileDown className="size-3.5" />
          Export area
        </button>
        <button
          onClick={() => api.runCommand("add-output")}
          className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Add output"
          aria-label="Add output"
        >
          <Plus className="size-4" />
        </button>
      </div>
    </div>
  );
}

function StatusDot({ status }: { status: ExportArea["status"] }) {
  if (status === "ready")
    return (
      <span title="Up to date" className="flex items-center text-primary">
        <Check className="size-3.5" />
      </span>
    );
  if (status === "warning")
    return (
      <span title="Has warnings" className="flex items-center text-warning">
        <AlertTriangle className="size-3.5" />
      </span>
    );
  return (
    <span title="Source changed — re-export" className="flex items-center text-muted-foreground">
      <RotateCcw className="size-3.5" />
    </span>
  );
}

function HistoryPanel() {
  const { data: history, isLoading } = useHistory();
  const { data: jumpSupported = false } = useHistoryJumpSupported();
  const undo = useCommandStore((s) => s.undo);
  const redo = useCommandStore((s) => s.redo);
  const jumpTo = useCommandStore((s) => s.jumpTo);

  const entries = history?.entries ?? [];
  const currentIndex = history?.currentIndex ?? null;
  const canUndo = currentIndex !== null;
  const canRedo = entries.length > 0 && (currentIndex === null ? true : currentIndex < entries.length - 1);

  return (
    <div className="flex flex-1 flex-col overflow-hidden" aria-label="History">
      <div className="flex items-center gap-1 border-b border-border p-1.5">
        <button
          onClick={() => undo()}
          disabled={!canUndo}
          className="flex flex-1 items-center justify-center gap-1.5 rounded-md py-1.5 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
        >
          <Undo2 className="size-3.5" />
          Undo
        </button>
        <button
          onClick={() => redo()}
          disabled={!canRedo}
          className="flex flex-1 items-center justify-center gap-1.5 rounded-md py-1.5 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
        >
          <Redo2 className="size-3.5" />
          Redo
        </button>
      </div>
      <ol className="flex-1 overflow-y-auto p-1.5">
        {isLoading && <li className="px-2 py-1.5 text-[13px] text-muted-foreground">Loading history…</li>}
        {!isLoading && entries.length === 0 && (
          <li className="px-3 py-6 text-center text-[13px] text-muted-foreground">
            No history yet. Edits you make will appear here.
          </li>
        )}
        {entries.map((entry, index) => {
          const isCurrent = currentIndex === index;
          const isFuture = currentIndex === null ? true : index > currentIndex;
          return (
            <HistoryRow
              key={entry.id}
              label={entry.label}
              isCurrent={isCurrent}
              isFuture={isFuture}
              canJump={jumpSupported}
              onJump={() => jumpTo(index)}
            />
          );
        })}
      </ol>
    </div>
  );
}

function HistoryRow({
  label,
  isCurrent,
  isFuture,
  canJump,
  onJump,
}: {
  label: string;
  isCurrent: boolean;
  isFuture: boolean;
  canJump: boolean;
  onJump: () => void;
}) {
  const content = (
    <>
      <span
        className={cn("size-1.5 shrink-0 rounded-full", isCurrent ? "bg-primary" : "bg-border")}
        aria-hidden="true"
      />
      <span className="flex-1 truncate">{label}</span>
      {isCurrent && <span className="font-mono text-[10px] text-primary">current</span>}
    </>
  );

  const className = cn(
    "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors",
    isCurrent ? "bg-primary/12 text-foreground ring-1 ring-primary/30" : "text-muted-foreground",
    isFuture && !isCurrent && "opacity-50",
  );

  // Jump-to-state is only offered when the backend supports it; otherwise the
  // row is static and users undo/redo stepwise.
  if (!canJump) {
    return <li className={className}>{content}</li>;
  }
  return (
    <li>
      <button
        onClick={onJump}
        title={isFuture ? "Redo to this state" : "Jump to this state"}
        className={cn(className, "hover:bg-secondary/70")}
      >
        {content}
      </button>
    </li>
  );
}
