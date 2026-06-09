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
  Link2,
  Box,
  RefreshCw,
  ExternalLink,
  Replace,
  ClipboardPaste,
  History as HistoryIcon,
  Undo2,
  Redo2,
  Frame,
  Maximize2,
  Unlink,
} from "lucide-react";
import type { Area, ImageObject, ImageSourceState, Layer, Output } from "@/lib/fleck-data";
import { api } from "@/lib/api";
import { useAreas, useHistory, useHistoryJumpSupported, useImageObjects, useLayers } from "@/lib/queries";
import { BLEND_MODES } from "@/lib/layer-commands";
import { SOURCE_STATE_LABEL } from "@/lib/image-commands";
import {
  EXPORT_BACKGROUNDS,
  newExportId,
  OUTPUT_FORMATS,
  SCALE_PRESETS,
  formatParam,
  isLossyFormat,
} from "@/lib/area-commands";
import { openImageFlow, pasteImageFlow, replaceImageFlow, revealImageSourceFlow } from "@/lib/image-import";
import { cn } from "@/lib/utils";
import { useUIStore, type SideTab } from "@/store/ui-store";
import { useViewportStore } from "@/store/viewport-store";
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
  const { data: imageObjects = [] } = useImageObjects();
  const { data: areas = [] } = useAreas();

  return (
    <Tabs value={tab} onValueChange={(v) => setTab(v as SideTab)} asChild>
      <aside className="flex w-72 shrink-0 flex-col border-l border-border bg-sidebar" aria-label="Editor panels">
        <TabsList className="border-b border-border p-1.5">
          <PanelTab value="layers" label="Layers" icon={Layers} count={layers.length} active={tab === "layers"} />
          <PanelTab value="images" label="Images" icon={ImageIcon} count={imageObjects.length} active={tab === "images"} />
          <PanelTab value="exports" label="Exports" icon={FileDown} count={areas.length} active={tab === "exports"} />
          <PanelTab value="history" label="History" icon={HistoryIcon} active={tab === "history"} />
        </TabsList>

        <TabsContent value="layers">
          <LayersPanel />
        </TabsContent>
        <TabsContent value="images">
          <ImagesPanel />
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

/** Icon-only panel tab (labels live in the tooltip/aria so four tabs fit the rail). */
function PanelTab({
  value,
  label,
  icon: Icon,
  count,
  active,
}: {
  value: SideTab;
  label: string;
  icon: LucideIcon;
  count?: number;
  active: boolean;
}) {
  return (
    <TabsTrigger value={value} title={label} aria-label={label}>
      <Icon className="size-4" />
      {count !== undefined && (
        <span
          className={cn(
            "rounded px-1 font-mono text-[10px]",
            active ? "bg-background text-muted-foreground" : "text-muted-foreground",
          )}
        >
          {count}
        </span>
      )}
    </TabsTrigger>
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

// --- Images panel ------------------------------------------------------------

/** Image-object actions an action surface can request; mapped to core flows below. */
type ImageAction = "duplicate" | "replace" | "rasterize" | "reveal";

/** Per source-state presentation: icon + accent color (label from SOURCE_STATE_LABEL). */
const SOURCE_STATE_META: Record<ImageSourceState, { icon: LucideIcon; className: string }> = {
  linked: { icon: Link2, className: "text-primary" },
  embedded: { icon: Box, className: "text-muted-foreground" },
  missing: { icon: AlertTriangle, className: "text-destructive" },
  replaced: { icon: RefreshCw, className: "text-warning" },
};

function ImagesPanel() {
  const { data: objects = [], isLoading } = useImageObjects();
  const selected = useUIStore((s) => s.selectedImageObjectId);
  const onSelect = useUIStore((s) => s.setSelectedImageObjectId);
  const execute = useCommandStore((s) => s.execute);

  const selectedObject = objects.find((o) => o.id === selected) ?? objects[0];

  // Import flows go through native acquisition + the undoable image command engine
  // (see image-import). Pure mutations execute their core command directly.
  const handleAction = (action: ImageAction, object: ImageObject) => {
    switch (action) {
      case "duplicate":
        execute("image.duplicate_object", { object_id: object.id });
        break;
      case "rasterize":
        execute("image.rasterize_object", { object_id: object.id });
        break;
      case "replace":
        replaceImageFlow(object.id);
        break;
      case "reveal":
        revealImageSourceFlow(object.id);
        break;
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Placed images</span>
        <div className="flex items-center gap-0.5">
          <HeaderButton onClick={() => openImageFlow()} icon={Plus} label="Open image" />
          <HeaderButton onClick={() => pasteImageFlow()} icon={ClipboardPaste} label="Paste image" />
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-1.5 pb-2">
        {isLoading && <p className="px-3 py-4 text-[13px] text-muted-foreground">Loading images…</p>}
        {!isLoading && objects.length === 0 && (
          <p className="px-3 py-6 text-center text-[13px] text-muted-foreground">
            No placed images. Open, paste, or drag an image into the workspace.
          </p>
        )}
        {objects.map((object) => (
          <ImageRow
            key={object.id}
            object={object}
            selected={selectedObject?.id === object.id}
            onSelect={() => onSelect(object.id)}
            onAction={(a) => handleAction(a, object)}
          />
        ))}
      </div>

      {selectedObject ? (
        <ImageObjectInspector object={selectedObject} onAction={(a) => handleAction(a, selectedObject)} />
      ) : (
        <div className="border-t border-border p-3" aria-label="Inspector">
          <p className="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
          <p className="text-[13px] text-muted-foreground">No selection.</p>
        </div>
      )}
    </div>
  );
}

function HeaderButton({ onClick, icon: Icon, label }: { onClick: () => void; icon: LucideIcon; label: string }) {
  return (
    <button
      onClick={onClick}
      title={label}
      aria-label={label}
      className="flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
    >
      <Icon className="size-4" />
    </button>
  );
}

function ImageRow({
  object,
  selected,
  onSelect,
  onAction,
}: {
  object: ImageObject;
  selected: boolean;
  onSelect: () => void;
  onAction: (action: ImageAction) => void;
}) {
  const isLinked = object.sourceState === "linked";
  const isRasterized = object.rasterizedLayerId !== null;

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <button
          onClick={onSelect}
          aria-pressed={selected}
          aria-current={selected}
          className={cn(
            "group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors",
            selected ? "bg-primary/12 ring-1 ring-primary/30" : "hover:bg-secondary/70",
          )}
        >
          <SourceStateIcon state={object.sourceState} />
          <span className="flex-1 truncate text-[13px] text-foreground">{object.name}</span>
          <span className="sr-only">{SOURCE_STATE_LABEL[object.sourceState]}</span>
          {isRasterized && (
            <span className="font-mono text-[9px] uppercase tracking-wide text-muted-foreground">rast</span>
          )}
          {object.dimensions && (
            <span className="font-mono text-[10px] text-muted-foreground">{object.dimensions}</span>
          )}
        </button>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onSelect={() => onAction("duplicate")}>
          <Copy />
          Duplicate
          <ContextMenuShortcut>⌘J</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => onAction("replace")}>
          <Replace />
          Replace source…
        </ContextMenuItem>
        <ContextMenuItem disabled={isRasterized} onSelect={() => onAction("rasterize")}>
          <Layers />
          Rasterize
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem disabled={!isLinked} onSelect={() => onAction("reveal")}>
          <ExternalLink />
          Reveal source
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}

function SourceStateIcon({ state }: { state: ImageSourceState }) {
  const { icon: Icon, className } = SOURCE_STATE_META[state];
  return <Icon className={cn("size-4 shrink-0", className)} />;
}

function SourceStateBadge({ state }: { state: ImageSourceState }) {
  const { icon: Icon, className } = SOURCE_STATE_META[state];
  return (
    <span className={cn("flex items-center gap-1 text-[10px] font-medium uppercase tracking-wide", className)}>
      <Icon className="size-3" />
      {SOURCE_STATE_LABEL[state]}
    </span>
  );
}

function ImageObjectInspector({
  object,
  onAction,
}: {
  object: ImageObject;
  onAction: (action: ImageAction) => void;
}) {
  const isLinked = object.sourceState === "linked";
  const isRasterized = object.rasterizedLayerId !== null;

  return (
    <div className="max-h-[58%] shrink-0 overflow-y-auto border-t border-border p-3" aria-label="Inspector">
      <div className="mb-2.5 flex items-center justify-between gap-2">
        <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
        <SourceStateBadge state={object.sourceState} />
      </div>

      <div className="space-y-2.5">
        <Field label="Name">
          <span className="truncate text-[13px] text-foreground">{object.name}</span>
        </Field>
        <Field label="Source">
          <span className="block truncate text-[12px] text-foreground" title={object.sourcePath ?? object.sourceName}>
            {object.sourceName}
          </span>
        </Field>
        <Field label="Format">
          <Readonly>{[object.format, object.dimensions].filter(Boolean).join(" · ") || "—"}</Readonly>
        </Field>
      </div>

      {object.sourceState === "missing" && (
        <p className="mt-2.5 flex items-start gap-1.5 rounded-md bg-destructive/10 px-2 py-1.5 text-[11px] text-destructive">
          <AlertTriangle className="mt-px size-3 shrink-0" />
          Linked file is missing. Replace the source to relink it.
        </p>
      )}

      {/* Transform is shown read-only: the core has no image-object transform
          command yet (see DEC-FE-006-image-transform-edit). */}
      <p className="mb-1.5 mt-3 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">Transform</p>
      <div className="space-y-2.5">
        <Field label="Position">
          <Readonly>
            {round(object.position.x)}, {round(object.position.y)}
          </Readonly>
        </Field>
        <Field label="Scale">
          <Readonly>
            {round(object.scale.width)} × {round(object.scale.height)}
          </Readonly>
        </Field>
        <Field label="Rotation">
          <Readonly>{round(object.rotationDegrees)}°</Readonly>
        </Field>
        <Field label="Opacity">
          <Readonly>{object.opacity}%</Readonly>
        </Field>
        <Field label="Crop">
          <Readonly>{object.crop ? `${round(object.crop.width)} × ${round(object.crop.height)}` : "None"}</Readonly>
        </Field>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-1.5 border-t border-border pt-3">
        <InspectorButton onClick={() => onAction("replace")} icon={Replace} label="Replace" />
        <InspectorButton onClick={() => onAction("reveal")} icon={ExternalLink} label="Reveal" disabled={!isLinked} />
        <InspectorButton onClick={() => onAction("rasterize")} icon={Layers} label="Rasterize" disabled={isRasterized} />
        <InspectorButton onClick={() => onAction("duplicate")} icon={Copy} label="Duplicate" />
      </div>
    </div>
  );
}

/** Static, non-editable inspector value. */
function Readonly({ children }: { children: ReactNode }) {
  return <span className="font-mono text-[11px] text-foreground">{children}</span>;
}

function round(n: number): number {
  return Math.round(n * 10) / 10;
}

// --- Exports panel -----------------------------------------------------------

/** Export-area actions an action surface can request; mapped to core flows below. */
type AreaAction = "rename" | "duplicate" | "delete" | "export" | "zoom" | "add-output" | "preview";

/** Output actions an action surface can request. */
type OutputAction = "duplicate" | "detach" | "remove";

function ExportsPanel() {
  const { data: areas = [], isLoading } = useAreas();
  const selectedId = useUIStore((s) => s.selectedAreaId);
  const setSelected = useUIStore((s) => s.setSelectedAreaId);
  const setExportPreviewOpen = useUIStore((s) => s.setExportPreviewOpen);
  const execute = useCommandStore((s) => s.execute);
  const focus = useViewportStore((s) => s.focus);

  const [renamingId, setRenamingId] = useState<string | null>(null);

  const selectedArea = areas.find((a) => a.id === selectedId) ?? areas[0];

  // Every export edit routes through the command engine so it is undoable and
  // recorded in history (see area-commands + command-store).
  const handleAreaAction = (action: AreaAction, area: Area) => {
    switch (action) {
      case "rename":
        setRenamingId(area.id);
        break;
      case "duplicate":
        execute("area.duplicate", { id: area.id });
        break;
      case "delete":
        execute("area.delete", { id: area.id });
        if (selectedId === area.id) setSelected(null);
        break;
      case "export":
        api.exportArea(area.id);
        break;
      case "preview":
        setSelected(area.id);
        setExportPreviewOpen(true);
        break;
      case "zoom":
        setSelected(area.id);
        focus("area", area.id);
        break;
      case "add-output":
        addOutput(area.id);
        break;
    }
  };

  // Adding an output is a create + attach: the core `output.add` only registers
  // the output on the workspace, so we attach it to the area as a second step.
  const addOutput = async (areaId: string) => {
    const id = newExportId("output");
    await execute("output.add", { id, filename: "export.png", format: "png" });
    await execute("area.attach_output", { area_id: areaId, output_id: id });
  };

  const handleOutputAction = async (action: OutputAction, area: Area, output: Output) => {
    switch (action) {
      case "duplicate": {
        // Duplicate registers a detached copy; attach it to the same area.
        const newId = newExportId("output");
        await execute("output.duplicate", { id: output.id, new_id: newId });
        await execute("area.attach_output", { area_id: area.id, output_id: newId });
        break;
      }
      case "detach":
        execute("area.detach_output", { area_id: area.id, output_id: output.id });
        break;
      case "remove":
        execute("output.remove", { id: output.id });
        break;
    }
  };

  const commitRename = (area: Area, name: string) => {
    setRenamingId(null);
    if (name && name !== area.name) execute("area.rename", { id: area.id, name });
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Areas</span>
        <button
          onClick={() => execute("area.create", { name: "Area" })}
          className="flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Add area"
          aria-label="Add area"
        >
          <Plus className="size-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-1.5 pb-2">
        {isLoading && <p className="px-3 py-4 text-[13px] text-muted-foreground">Loading areas…</p>}
        {!isLoading && areas.length === 0 && (
          <p className="px-3 py-6 text-center text-[13px] text-muted-foreground">
            No areas yet. Use the area tool or right-click the canvas to mark a region.
          </p>
        )}
        {areas.map((area) => (
          <AreaRow
            key={area.id}
            area={area}
            selected={selectedArea?.id === area.id}
            renaming={renamingId === area.id}
            onSelect={() => setSelected(area.id)}
            onAction={(a) => handleAreaAction(a, area)}
            onCommitRename={(name) => commitRename(area, name)}
            onCancelRename={() => setRenamingId(null)}
          />
        ))}
      </div>

      {selectedArea ? (
        <ExportInspector
          area={selectedArea}
          onAreaAction={(a) => handleAreaAction(a, selectedArea)}
          onOutputAction={(a, output) => handleOutputAction(a, selectedArea, output)}
          onCommitRename={(name) => commitRename(selectedArea, name)}
          onMoveArea={(x, y) => execute("area.move", { id: selectedArea.id, x, y })}
          onResizeArea={(width, height) => execute("area.resize", { id: selectedArea.id, width, height })}
          onSetAreaPadding={(padding) => execute("area.set_padding", { id: selectedArea.id, ...padding })}
          onSetAreaBackground={(background) =>
            execute("area.set_background", { id: selectedArea.id, background })
          }
          onSetOutputFormat={(output, format) => execute("output.update", { id: output.id, format })}
          onSetOutputScale={(output, scale) => execute("output.update", { id: output.id, scale })}
          onSetOutputQuality={(output, quality) => execute("output.update", { id: output.id, quality })}
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

function AreaRow({
  area,
  selected,
  renaming,
  onSelect,
  onAction,
  onCommitRename,
  onCancelRename,
}: {
  area: Area;
  selected: boolean;
  renaming: boolean;
  onSelect: () => void;
  onAction: (action: AreaAction) => void;
  onCommitRename: (name: string) => void;
  onCancelRename: () => void;
}) {
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          className={cn(
            "group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors",
            selected ? "bg-primary/12 ring-1 ring-primary/30" : "hover:bg-secondary/70",
          )}
        >
          <Frame className={cn("size-4 shrink-0", selected ? "text-primary" : "text-muted-foreground")} />
          {renaming ? (
            <NameInput
              initial={area.name}
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
              <span className="flex-1 truncate font-mono text-[13px] text-foreground">{area.name}</span>
              <StatusDot status={area.status} />
              <span className="font-mono text-[10px] text-muted-foreground">{area.dimensions}</span>
            </button>
          )}
        </div>
      </ContextMenuTrigger>
      <AreaMenu onAction={onAction} />
    </ContextMenu>
  );
}

/** Right-click action list shared by area rows and the canvas. */
function AreaMenu({ onAction }: { onAction: (action: AreaAction) => void }) {
  return (
    <ContextMenuContent>
      <ContextMenuItem onSelect={() => onAction("preview")}>
        <Eye />
        Preview export…
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("export")}>
        <FileDown />
        Area
        <ContextMenuShortcut>⌘E</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("zoom")}>
        <Maximize2 />
        Zoom to area
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onSelect={() => onAction("rename")}>
        <Pencil />
        Rename
        <ContextMenuShortcut>F2</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("duplicate")}>
        <Copy />
        Duplicate
        <ContextMenuShortcut>⌘D</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => onAction("add-output")}>
        <Plus />
        Add output
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" onSelect={() => onAction("delete")}>
        <Trash2 />
        Delete
      </ContextMenuItem>
    </ContextMenuContent>
  );
}

function ExportInspector({
  area,
  onAreaAction,
  onOutputAction,
  onCommitRename,
  onMoveArea,
  onResizeArea,
  onSetAreaPadding,
  onSetAreaBackground,
  onSetOutputFormat,
  onSetOutputScale,
  onSetOutputQuality,
}: {
  area: Area;
  onAreaAction: (action: AreaAction) => void;
  onOutputAction: (action: OutputAction, output: Output) => void;
  onCommitRename: (name: string) => void;
  onMoveArea: (x: number, y: number) => void;
  onResizeArea: (width: number, height: number) => void;
  onSetAreaPadding: (padding: Area["paddingPx"]) => void;
  onSetAreaBackground: (background: string) => void;
  onSetOutputFormat: (output: Output, format: string) => void;
  onSetOutputScale: (output: Output, scale: number) => void;
  onSetOutputQuality: (output: Output, quality: number) => void;
}) {
  return (
    <div className="max-h-[60%] shrink-0 overflow-y-auto border-t border-border p-3" aria-label="Inspector">
      <div className="mb-2.5 flex items-center justify-between gap-2">
        <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
        {area.status === "warning" && (
          <span className="flex items-center gap-1 text-[10px] font-medium uppercase tracking-wide text-warning">
            <AlertTriangle className="size-3" />
            {area.warnings.length} warning{area.warnings.length === 1 ? "" : "s"}
          </span>
        )}
      </div>

      <div className="space-y-2.5">
        <Field label="Name">
          <NameInput
            key={`${area.id}:${area.name}`}
            initial={area.name}
            onCommit={onCommitRename}
            className="h-7 w-full rounded border border-border bg-background px-1.5 text-[13px] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
        </Field>
        <Field label="Size">
          <NumericPair
            x={area.bounds.width}
            y={area.bounds.height}
            xLabel="Width"
            yLabel="Height"
            min={1}
            onCommit={(width, height) => onResizeArea(width, height)}
          />
        </Field>
        <Field label="Position">
          <NumericPair
            x={area.bounds.x}
            y={area.bounds.y}
            xLabel="X"
            yLabel="Y"
            onCommit={(x, y) => onMoveArea(x, y)}
          />
        </Field>
        <Field label="Padding">
          <PaddingControl padding={area.paddingPx} onCommit={onSetAreaPadding} />
        </Field>
        <Field label="Background">
          <BackgroundMenu value={area.backgroundParam} onSelect={onSetAreaBackground} />
        </Field>
      </div>

      {/* Warnings come straight from core export preview metadata (REQ-039). */}
      {area.warnings.length > 0 && (
        <ul className="mt-2.5 space-y-1">
          {area.warnings.map((warning) => (
            <li
              key={warning}
              className="flex items-start gap-1.5 rounded-md bg-warning/10 px-2 py-1.5 text-[11px] text-warning"
            >
              <AlertTriangle className="mt-px size-3 shrink-0" />
              {warning}
            </li>
          ))}
        </ul>
      )}

      <div className="mb-1.5 mt-3 flex items-center justify-between">
        <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
          Outputs ({area.outputs.length})
        </p>
        <button
          onClick={() => onAreaAction("add-output")}
          className="flex size-5 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Add output"
          aria-label="Add output"
        >
          <Plus className="size-3.5" />
        </button>
      </div>
      {area.outputs.length === 0 ? (
        <p className="rounded-md bg-secondary/40 px-2 py-1.5 text-[11px] text-muted-foreground">
          No outputs. Add one to export this area.
        </p>
      ) : (
        <div className="space-y-1.5">
          {area.outputs.map((output) => (
            <OutputCard
              key={output.id}
              output={output}
              onAction={(a) => onOutputAction(a, output)}
              onSetFormat={(format) => onSetOutputFormat(output, format)}
              onSetScale={(scale) => onSetOutputScale(output, scale)}
              onSetQuality={(quality) => onSetOutputQuality(output, quality)}
            />
          ))}
        </div>
      )}

      <div className="mt-3 border-t border-border pt-3">
        <button
          onClick={() => onAreaAction("preview")}
          className="flex h-8 w-full items-center justify-center gap-1.5 rounded-md bg-primary text-[12px] font-medium text-primary-foreground transition-transform active:scale-[0.99]"
        >
          <Eye className="size-3.5" />
          Preview export…
        </button>
        <div className="mt-1.5 grid grid-cols-2 gap-1.5">
          <InspectorButton onClick={() => onAreaAction("export")} icon={FileDown} label="Export" />
          <InspectorButton onClick={() => onAreaAction("zoom")} icon={Maximize2} label="Zoom to" />
          <InspectorButton onClick={() => onAreaAction("duplicate")} icon={Copy} label="Duplicate" />
          <InspectorButton onClick={() => onAreaAction("delete")} icon={Trash2} label="Delete" destructive />
        </div>
      </div>
    </div>
  );
}

function NumericPair({
  x,
  y,
  xLabel,
  yLabel,
  min,
  onCommit,
}: {
  x: number;
  y: number;
  xLabel: string;
  yLabel: string;
  min?: number;
  onCommit: (x: number, y: number) => void;
}) {
  const [draftX, setDraftX] = useState(String(round(x)));
  const [draftY, setDraftY] = useState(String(round(y)));

  useEffect(() => {
    setDraftX(String(round(x)));
    setDraftY(String(round(y)));
  }, [x, y]);

  const commit = () => {
    const nextX = clampNumeric(draftX, x, min);
    const nextY = clampNumeric(draftY, y, min);
    if (nextX !== round(x) || nextY !== round(y)) onCommit(nextX, nextY);
    setDraftX(String(nextX));
    setDraftY(String(nextY));
  };

  return (
    <div className="grid grid-cols-2 gap-1.5">
      <InspectorNumber value={draftX} label={xLabel} min={min} onChange={setDraftX} onCommit={commit} />
      <InspectorNumber value={draftY} label={yLabel} min={min} onChange={setDraftY} onCommit={commit} />
    </div>
  );
}

function PaddingControl({
  padding,
  onCommit,
}: {
  padding: Area["paddingPx"];
  onCommit: (padding: Area["paddingPx"]) => void;
}) {
  const [draft, setDraft] = useState({
    top: String(round(padding.top)),
    right: String(round(padding.right)),
    bottom: String(round(padding.bottom)),
    left: String(round(padding.left)),
  });

  useEffect(() => {
    setDraft({
      top: String(round(padding.top)),
      right: String(round(padding.right)),
      bottom: String(round(padding.bottom)),
      left: String(round(padding.left)),
    });
  }, [padding.top, padding.right, padding.bottom, padding.left]);

  const commit = () => {
    const next = {
      top: clampNumeric(draft.top, padding.top, 0),
      right: clampNumeric(draft.right, padding.right, 0),
      bottom: clampNumeric(draft.bottom, padding.bottom, 0),
      left: clampNumeric(draft.left, padding.left, 0),
    };
    if (
      next.top !== round(padding.top) ||
      next.right !== round(padding.right) ||
      next.bottom !== round(padding.bottom) ||
      next.left !== round(padding.left)
    ) {
      onCommit(next);
    }
    setDraft({
      top: String(next.top),
      right: String(next.right),
      bottom: String(next.bottom),
      left: String(next.left),
    });
  };

  return (
    <div className="grid grid-cols-4 gap-1">
      {(["top", "right", "bottom", "left"] as const).map((side) => (
        <InspectorNumber
          key={side}
          value={draft[side]}
          label={side.slice(0, 1).toUpperCase()}
          min={0}
          onChange={(value) => setDraft((prev) => ({ ...prev, [side]: value }))}
          onCommit={commit}
        />
      ))}
    </div>
  );
}

function InspectorNumber({
  value,
  label,
  min,
  onChange,
  onCommit,
}: {
  value: string;
  label: string;
  min?: number;
  onChange: (value: string) => void;
  onCommit: () => void;
}) {
  return (
    <label className="flex h-7 items-center gap-1 rounded border border-border bg-background px-1 text-[11px] text-muted-foreground focus-within:ring-2 focus-within:ring-ring">
      <span className="shrink-0">{label}</span>
      <input
        type="number"
        min={min}
        value={value}
        aria-label={label}
        onChange={(e) => onChange(e.target.value)}
        onBlur={onCommit}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter") {
            e.preventDefault();
            onCommit();
          }
        }}
        className="min-w-0 flex-1 bg-transparent text-right font-mono text-[11px] text-foreground outline-none"
      />
    </label>
  );
}

function BackgroundMenu({ value, onSelect }: { value: string; onSelect: (value: string) => void }) {
  const current = EXPORT_BACKGROUNDS.find((b) => b.value === value) ?? EXPORT_BACKGROUNDS[0];
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          className="flex h-7 w-full items-center justify-between gap-1 rounded border border-border bg-background px-1.5 text-[11px] text-foreground transition-colors hover:bg-secondary"
          aria-label="Area background"
        >
          {current.label}
          <ChevronDown className="size-3 text-muted-foreground" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="min-w-40">
        {EXPORT_BACKGROUNDS.map((background) => (
          <DropdownMenuItem key={background.value} onSelect={() => onSelect(background.value)}>
            {background.label}
            {background.value === value && <Check className="ml-auto size-3.5 text-primary" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function clampNumeric(value: string, fallback: number, min?: number): number {
  const parsed = Math.round(Number(value));
  if (!Number.isFinite(parsed)) return round(fallback);
  return min == null ? parsed : Math.max(min, parsed);
}

function OutputCard({
  output,
  onAction,
  onSetFormat,
  onSetScale,
  onSetQuality,
}: {
  output: Output;
  onAction: (action: OutputAction) => void;
  onSetFormat: (format: string) => void;
  onSetScale: (scale: number) => void;
  onSetQuality: (quality: number) => void;
}) {
  const lossy = isLossyFormat(output.format);
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div className="rounded-md border border-border bg-card/60 p-2">
          <div className="flex items-center gap-2">
            <FileDown className="size-3.5 shrink-0 text-muted-foreground" />
            <span className="flex-1 truncate font-mono text-[12px] text-foreground" title={output.destination ?? output.filename}>
              {output.filename}
            </span>
            <span className="font-mono text-[10px] text-muted-foreground">{output.dimensions}</span>
          </div>
          <div className="mt-1.5 flex items-center gap-1.5">
            <SelectMenu
              value={output.format}
              label={`Format: ${output.format}`}
              options={OUTPUT_FORMATS.map((f) => ({ value: f.label, label: f.label }))}
              onSelect={(label) => onSetFormat(formatParam(label))}
            />
            <SelectMenu
              value={output.scale}
              label={`Scale: ${output.scale}`}
              options={SCALE_PRESETS.map((s) => ({ value: s.label, label: s.label }))}
              onSelect={(label) => onSetScale(SCALE_PRESETS.find((s) => s.label === label)?.value ?? 1)}
            />
            {lossy && <QualityInput value={output.quality ?? 80} onCommit={onSetQuality} />}
          </div>
          {output.destination && (
            <p className="mt-1 truncate font-mono text-[10px] text-muted-foreground" title={output.destination}>
              → {output.destination}
            </p>
          )}
        </div>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onSelect={() => onAction("duplicate")}>
          <Copy />
          Duplicate
          <ContextMenuShortcut>⌘D</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => onAction("detach")}>
          <Unlink />
          Detach from area
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem variant="destructive" onSelect={() => onAction("remove")}>
          <Trash2 />
          Remove
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}

/** Small labelled dropdown used for output format/scale selection. */
function SelectMenu({
  value,
  label,
  options,
  onSelect,
}: {
  value: string;
  label: string;
  options: { value: string; label: string }[];
  onSelect: (value: string) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          className="flex items-center gap-1 rounded bg-secondary px-1.5 py-0.5 text-[10px] text-foreground transition-colors hover:bg-secondary/70"
          aria-label={label}
        >
          {value}
          <ChevronDown className="size-3 text-muted-foreground" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="max-h-64 min-w-28 overflow-y-auto">
        {options.map((option) => (
          <DropdownMenuItem key={option.value} onSelect={() => onSelect(option.value)}>
            {option.label}
            {option.value === value && <Check className="ml-auto size-3.5 text-primary" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/** Quality field for lossy formats; commits a single undoable step on blur/Enter. */
function QualityInput({ value, onCommit }: { value: number; onCommit: (quality: number) => void }) {
  const [draft, setDraft] = useState(String(value));

  useEffect(() => {
    setDraft(String(value));
  }, [value]);

  const commit = () => {
    const next = Math.max(1, Math.min(100, Math.round(Number(draft) || value)));
    if (next !== value) onCommit(next);
    setDraft(String(next));
  };

  return (
    <span className="flex items-center gap-1 rounded bg-secondary px-1.5 py-0.5 text-[10px] text-muted-foreground">
      Q
      <input
        type="number"
        min={1}
        max={100}
        value={draft}
        aria-label="Output quality"
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter") {
            e.preventDefault();
            commit();
          }
        }}
        className="w-9 bg-transparent text-right font-mono text-[10px] text-foreground outline-none"
      />
    </span>
  );
}

function StatusDot({ status }: { status: Area["status"] }) {
  if (status === "ready")
    return (
      <span title="Ready to export" className="flex items-center text-primary">
        <Check className="size-3.5" />
      </span>
    );
  return (
    <span title="Has warnings" className="flex items-center text-warning">
      <AlertTriangle className="size-3.5" />
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
