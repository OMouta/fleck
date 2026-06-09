import { useEffect, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import {
  AlertTriangle,
  Check,
  ChevronDown,
  Copy,
  Crop,
  FileDown,
  FolderOpen,
  ImageIcon,
  Layers,
  Loader2,
  X,
} from "lucide-react";
import type { Area, ExportResult, ExportResultOutput, Output } from "@/lib/fleck-data";
import { api } from "@/lib/api";
import { useAreas } from "@/lib/queries";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store/ui-store";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

/**
 * Export preview + result dialog (REQ-032/REQ-033). It shows the full export
 * plan for an area — crop, background, padding, and per-output dimensions,
 * format, transparency, estimated size, filename, and destination — with all
 * warnings surfaced *before* anything is exported. Running an export then calls
 * the backend job and previews the produced result, with copy/reveal actions.
 *
 * The dialog targets the currently selected area so it stays in sync with
 * the exports panel and canvas selection.
 */
export function ExportPreviewDialog() {
  const open = useUIStore((s) => s.exportPreviewOpen);
  const setOpen = useUIStore((s) => s.setExportPreviewOpen);
  const selectedId = useUIStore((s) => s.selectedAreaId);
  const { data: areas = [] } = useAreas();

  const area = areas.find((a) => a.id === selectedId) ?? areas[0];

  // Result of the last export run, plus an in-flight flag for the buttons.
  const [result, setResult] = useState<ExportResult | null>(null);
  const [busy, setBusy] = useState<"area" | "all" | null>(null);

  // Clear any prior result whenever the dialog opens or the target area changes.
  useEffect(() => {
    if (open) setResult(null);
  }, [open, area?.id]);

  const runArea = async () => {
    if (!area) return;
    setBusy("area");
    try {
      setResult(await api.exportArea(area.id));
    } finally {
      setBusy(null);
    }
  };

  const runExportAll = async () => {
    setBusy("all");
    try {
      setResult(await api.exportAll());
    } finally {
      setBusy(null);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-background/70 backdrop-blur-sm animate-in-fade" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 flex max-h-[86vh] w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-xl border border-border bg-popover shadow-2xl outline-none animate-in-pop">
          {area ? (
            <>
              <div className="flex items-start gap-3 border-b border-border p-4">
                <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/15 text-primary">
                  <FileDown className="size-5" />
                </span>
                <div className="min-w-0 flex-1">
                  <Dialog.Title className="truncate text-sm font-semibold text-foreground">
                    Export “{area.name}”
                  </Dialog.Title>
                  <Dialog.Description className="mt-1 text-[13px] text-muted-foreground">
                    Review the output plan and warnings before exporting.
                  </Dialog.Description>
                </div>
                <Dialog.Close
                  className="flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                  aria-label="Close"
                >
                  <X className="size-4" />
                </Dialog.Close>
              </div>

              <div className="flex-1 overflow-y-auto p-4">
                {result ? (
                  <ExportResultView result={result} />
                ) : (
                  <ExportPlanView area={area} />
                )}
              </div>

              <div className="flex items-center justify-between gap-2 border-t border-border p-3">
                <button
                  onClick={runExportAll}
                  disabled={busy !== null}
                  className="flex h-8 items-center gap-1.5 rounded-md px-2.5 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:pointer-events-none disabled:opacity-50"
                  title="Export every area (placeholder batch export)"
                >
                  <Layers className="size-3.5" />
                  Export all
                </button>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setOpen(false)}
                    className="h-8 rounded-md px-3 text-[13px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                  >
                    Close
                  </button>
                  <button
                    onClick={runArea}
                    disabled={busy !== null}
                    className="flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
                  >
                    {busy === "area" ? <Loader2 className="size-3.5 animate-spin" /> : <FileDown className="size-3.5" />}
                    {result ? "Export again" : "Area"}
                  </button>
                </div>
              </div>
            </>
          ) : (
            <div className="p-8 text-center">
              <Dialog.Title className="text-sm font-semibold text-foreground">No area selected</Dialog.Title>
              <Dialog.Description className="mt-1 text-[13px] text-muted-foreground">
                Create or select an area to preview its output.
              </Dialog.Description>
            </div>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

/** Pre-export plan: crop/background/padding + per-output settings + warnings. */
function ExportPlanView({ area }: { area: Area }) {
  return (
    <div className="space-y-4">
      {area.warnings.length > 0 && (
        <ul className="space-y-1">
          {area.warnings.map((warning) => (
            <li
              key={warning}
              className="flex items-start gap-2 rounded-md bg-warning/10 px-2.5 py-1.5 text-[12px] text-warning"
            >
              <AlertTriangle className="mt-px size-3.5 shrink-0" />
              {warning}
            </li>
          ))}
        </ul>
      )}

      <section>
        <SectionLabel icon={Crop}>Area</SectionLabel>
        <dl className="grid grid-cols-2 gap-x-4 gap-y-1.5">
          <Spec label="Crop" value={area.dimensions} />
          <Spec label="Position" value={area.position} />
          <Spec label="Padding" value={area.padding} />
          <Spec label="Background" value={area.background} />
        </dl>
      </section>

      <section>
        <SectionLabel icon={FileDown}>Outputs ({area.outputs.length})</SectionLabel>
        {area.outputs.length === 0 ? (
          <p className="rounded-md bg-secondary/40 px-2.5 py-2 text-[12px] text-muted-foreground">
            This area has no outputs. Add one to export it.
          </p>
        ) : (
          <div className="space-y-2">
            {area.outputs.map((output) => (
              <OutputPlanCard key={output.id} output={output} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function OutputPlanCard({ output }: { output: Output }) {
  return (
    <div className="rounded-md border border-border bg-card/60 p-2.5">
      <div className="flex items-center gap-2">
        <FileDown className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="flex-1 truncate font-mono text-[12px] text-foreground">{output.filename}</span>
        <span className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
          {output.format}
        </span>
      </div>
      <dl className="mt-2 grid grid-cols-2 gap-x-4 gap-y-1">
        <Spec label="Dimensions" value={output.dimensions} />
        <Spec label="Scale" value={output.scale} />
        <Spec label="Transparency" value={output.transparency} />
        <Spec label="Est. size" value={output.estimatedSize} />
        {output.quality !== null && <Spec label="Quality" value={`${output.quality}`} />}
        <Spec label="Destination" value={output.destination ?? "Next to workspace"} title={output.destination ?? undefined} />
      </dl>
    </div>
  );
}

/** Post-export result: produced outputs with size, preview, copy + reveal. */
function ExportResultView({ result }: { result: ExportResult }) {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 rounded-md bg-primary/10 px-2.5 py-2 text-[12px] text-foreground">
        <Check className="size-4 shrink-0 text-primary" />
        Exported {result.outputs.length} output{result.outputs.length === 1 ? "" : "s"} from{" "}
        <span className="font-medium">{result.scope}</span>.
      </div>

      {result.warnings.length > 0 && (
        <ul className="space-y-1">
          {result.warnings.map((warning) => (
            <li
              key={warning}
              className="flex items-start gap-2 rounded-md bg-warning/10 px-2.5 py-1.5 text-[12px] text-warning"
            >
              <AlertTriangle className="mt-px size-3.5 shrink-0" />
              {warning}
            </li>
          ))}
        </ul>
      )}

      <div className="space-y-2">
        {result.outputs.map((output) => (
          <ExportResultCard key={output.id} output={output} />
        ))}
      </div>

      {result.failures.length > 0 && (
        <ul className="space-y-1">
          {result.failures.map((failure) => (
            <li
              key={failure.filename}
              className="flex items-start gap-2 rounded-md bg-destructive/10 px-2.5 py-1.5 text-[12px] text-destructive"
            >
              <AlertTriangle className="mt-px size-3.5 shrink-0" />
              <span>
                <span className="font-mono">{failure.filename}</span> — {failure.reason}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function ExportResultCard({ output }: { output: ExportResultOutput }) {
  const copy = (mode: "image" | "base64" | "markdown") => api.copyExportResult(output.id, mode);
  const reveal = () => api.revealExportedFile(output.destination ?? output.filename);

  return (
    <div className="flex items-center gap-3 rounded-md border border-border bg-card/60 p-2.5">
      <div className="flex size-12 shrink-0 items-center justify-center overflow-hidden rounded border border-border bg-background">
        {output.dataUrl ? (
          <img src={output.dataUrl} alt={output.filename} className="size-full object-contain" />
        ) : (
          <ImageIcon className="size-5 text-muted-foreground" />
        )}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate font-mono text-[12px] text-foreground">{output.filename}</p>
        <p className="truncate font-mono text-[10px] text-muted-foreground">
          {output.format} · {output.dimensions} · {output.size}
        </p>
        {output.destination && (
          <p className="truncate font-mono text-[10px] text-muted-foreground" title={output.destination}>
            → {output.destination}
          </p>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button
              className="flex h-7 items-center gap-1 rounded-md border border-border px-2 text-[12px] text-foreground transition-colors hover:bg-secondary"
              aria-label="Copy export result"
            >
              <Copy className="size-3.5" />
              <ChevronDown className="size-3 text-muted-foreground" />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={() => copy("image")}>
              <ImageIcon />
              Copy image
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => copy("base64")}>
              <Copy />
              Copy as Base64
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => copy("markdown")}>
              <Copy />
              Copy as Markdown
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <button
          onClick={reveal}
          className="flex size-7 items-center justify-center rounded-md border border-border text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Reveal in file manager"
          aria-label="Reveal in file manager"
        >
          <FolderOpen className="size-3.5" />
        </button>
      </div>
    </div>
  );
}

function SectionLabel({ icon: Icon, children }: { icon: typeof FileDown; children: React.ReactNode }) {
  return (
    <p className="mb-2 flex items-center gap-1.5 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
      <Icon className="size-3" />
      {children}
    </p>
  );
}

function Spec({ label, value, title }: { label: string; value: string; title?: string }) {
  return (
    <div className="flex flex-col gap-0.5 overflow-hidden">
      <span className="text-[10px] uppercase tracking-wide text-muted-foreground">{label}</span>
      <span className={cn("truncate font-mono text-[12px] text-foreground")} title={title}>
        {value}
      </span>
    </div>
  );
}
