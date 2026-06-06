import {
  Eye,
  EyeOff,
  Lock,
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
  History as HistoryIcon,
  Undo2,
  Redo2,
} from "lucide-react";
import type { ExportArea, HistoryEntry, Layer } from "@/lib/fleck-data";
import { api } from "@/lib/api";
import {
  useExportAreas,
  useHistory,
  useLayers,
  useToggleLayerLocked,
  useToggleLayerVisibility,
} from "@/lib/queries";
import { cn } from "@/lib/utils";
import { useUIStore, type SideTab } from "@/store/ui-store";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

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

function LayersPanel() {
  const { data: layers = [], isLoading } = useLayers();
  const selected = useUIStore((s) => s.selectedLayerId);
  const onSelect = useUIStore((s) => s.setSelectedLayerId);
  const toggleVisible = useToggleLayerVisibility();
  const toggleLocked = useToggleLayerLocked();

  const selectedLayer = layers.find((l) => l.id === selected) ?? layers[0];

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Layer stack</span>
        <button
          onClick={() => api.runCommand("add-layer")}
          className="flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Add layer"
          aria-label="Add layer"
        >
          <Plus className="size-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-1.5 pb-2">
        {isLoading && <p className="px-3 py-4 text-[13px] text-muted-foreground">Loading layers…</p>}
        {!isLoading && layers.length === 0 && (
          <p className="px-3 py-6 text-center text-[13px] text-muted-foreground">
            No layers yet. Add a layer or open an image.
          </p>
        )}
        {layers.map((layer) => {
          const Icon = KIND_ICON[layer.kind];
          const isSelected = selected === layer.id;
          return (
            <div
              key={layer.id}
              className={cn(
                "group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors",
                isSelected ? "bg-primary/12 ring-1 ring-primary/30" : "hover:bg-secondary/70",
                !layer.visible && "opacity-50",
              )}
            >
              <button
                onClick={() => toggleVisible.mutate({ id: layer.id, visible: !layer.visible })}
                className="flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:text-foreground"
                aria-label={layer.visible ? `Hide ${layer.name}` : `Show ${layer.name}`}
                aria-pressed={layer.visible}
              >
                {layer.visible ? <Eye className="size-3.5" /> : <EyeOff className="size-3.5" />}
              </button>

              <button
                onClick={() => onSelect(layer.id)}
                className="flex flex-1 items-center gap-2 overflow-hidden text-left"
                aria-pressed={isSelected}
              >
                <Icon className={cn("size-4 shrink-0", isSelected ? "text-primary" : "text-muted-foreground")} />
                <span className="flex-1 truncate text-[13px] text-foreground">{layer.name}</span>
                {layer.opacity < 100 && (
                  <span className="font-mono text-[10px] text-muted-foreground">{layer.opacity}%</span>
                )}
              </button>

              <button
                onClick={() => toggleLocked.mutate({ id: layer.id, locked: !layer.locked })}
                className={cn(
                  "flex size-5 shrink-0 items-center justify-center rounded hover:text-foreground",
                  layer.locked ? "text-muted-foreground" : "text-transparent group-hover:text-muted-foreground",
                )}
                aria-label={layer.locked ? `Unlock ${layer.name}` : `Lock ${layer.name}`}
                aria-pressed={layer.locked}
              >
                <Lock className="size-3" />
              </button>
            </div>
          );
        })}
      </div>

      {selectedLayer ? (
        <SelectedLayerInspector layer={selectedLayer} />
      ) : (
        <div className="border-t border-border p-3" aria-label="Inspector">
          <p className="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
          <p className="text-[13px] text-muted-foreground">No selection.</p>
        </div>
      )}
    </div>
  );
}

function SelectedLayerInspector({ layer }: { layer: Layer }) {
  return (
    <div className="border-t border-border p-3" aria-label="Inspector">
      <p className="mb-2.5 text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspector</p>
      <div className="space-y-2.5">
        <Field label="Name">
          <span className="truncate text-[13px] text-foreground">{layer.name}</span>
        </Field>
        <Field label="Opacity">
          <div className="flex items-center gap-2">
            <div className="h-1 flex-1 overflow-hidden rounded-full bg-secondary">
              <div className="h-full rounded-full bg-primary" style={{ width: `${layer.opacity}%` }} />
            </div>
            <span className="w-9 text-right font-mono text-[11px] text-foreground">{layer.opacity}%</span>
          </div>
        </Field>
        <Field label="Blend">
          <span className="rounded bg-secondary px-1.5 py-0.5 text-[11px] text-foreground">{layer.blend}</span>
        </Field>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
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
  const { data: entries = [], isLoading } = useHistory();

  return (
    <div className="flex flex-1 flex-col overflow-hidden" aria-label="History">
      <div className="flex items-center gap-1 border-b border-border p-1.5">
        <button
          onClick={() => api.undo()}
          className="flex flex-1 items-center justify-center gap-1.5 rounded-md py-1.5 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
        >
          <Undo2 className="size-3.5" />
          Undo
        </button>
        <button
          onClick={() => api.redo()}
          className="flex flex-1 items-center justify-center gap-1.5 rounded-md py-1.5 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
        >
          <Redo2 className="size-3.5" />
          Redo
        </button>
      </div>
      <ol className="flex-1 overflow-y-auto p-1.5">
        {isLoading && <li className="px-2 py-1.5 text-[13px] text-muted-foreground">Loading history…</li>}
        {entries.map((entry) => (
          <HistoryRow key={entry.id} entry={entry} />
        ))}
      </ol>
    </div>
  );
}

function HistoryRow({ entry }: { entry: HistoryEntry }) {
  return (
    <li
      className={cn(
        "flex items-center gap-2 rounded-md px-2 py-1.5 text-[13px]",
        entry.current ? "bg-primary/12 text-foreground ring-1 ring-primary/30" : "text-muted-foreground",
      )}
    >
      <span
        className={cn("size-1.5 shrink-0 rounded-full", entry.current ? "bg-primary" : "bg-border")}
        aria-hidden="true"
      />
      <span className="flex-1 truncate">{entry.label}</span>
      {entry.current && <span className="font-mono text-[10px] text-primary">current</span>}
    </li>
  );
}
