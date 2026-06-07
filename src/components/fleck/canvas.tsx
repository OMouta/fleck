import { useEffect, useRef, useState } from "react";
import {
  Maximize2,
  FolderOpen,
  FilePlus2,
  Frame,
  Grid2x2,
  Grid3x3,
  Ruler,
  SquareDashed,
  Move,
} from "lucide-react";
import { api } from "@/lib/api";
import { useRenderModel } from "@/lib/queries";
import { paintScene, type Palette } from "@/lib/render";
import type { Point } from "@/lib/fleck-data";
import { cn } from "@/lib/utils";
import { openImageFlow, dropImageFlow } from "@/lib/image-import";
import { useUIStore } from "@/store/ui-store";
import { useViewportStore } from "@/store/viewport-store";
import { useWorkspaceFilesStore } from "@/store/workspace-files-store";

function readPalette(): Palette {
  const s = getComputedStyle(document.documentElement);
  const v = (name: string, fallback: string) => s.getPropertyValue(name).trim() || fallback;
  return {
    gridDot: "rgba(255, 255, 255, 0.07)",
    canvasBorder: v("--border", "#333"),
    layerStroke: v("--border", "#333"),
    exportArea: v("--primary", "#ddd"),
    exportLabelText: v("--primary-foreground", "#111"),
    guide: v("--warning", "#e0a000"),
    selection: "rgba(255, 255, 255, 0.9)",
    handleFill: v("--background", "#111"),
    handleBorder: v("--primary", "#ddd"),
    pixelGrid: "rgba(255, 255, 255, 0.16)",
  };
}

export function Canvas() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const paletteRef = useRef<Palette | null>(null);
  const dprRef = useRef(1);
  const spaceRef = useRef(false);

  const activeTool = useUIStore((s) => s.activeTool);
  const newWorkspace = useWorkspaceFilesStore((s) => s.newWorkspace);
  const [dragOver, setDragOver] = useState(false);

  const origin = useViewportStore((s) => s.origin);
  const zoom = useViewportStore((s) => s.zoom);
  const screen = useViewportStore((s) => s.screen);
  const overlays = useViewportStore((s) => s.overlays);
  const panning = useViewportStore((s) => s.panning);
  const setScreen = useViewportStore((s) => s.setScreen);
  const panByScreen = useViewportStore((s) => s.panByScreen);
  const zoomAt = useViewportStore((s) => s.zoomAt);
  const zoomCentered = useViewportStore((s) => s.zoomCentered);
  const setPanning = useViewportStore((s) => s.setPanning);
  const focus = useViewportStore((s) => s.focus);
  const toggleOverlay = useViewportStore((s) => s.toggleOverlay);
  const togglePixelGrid = useViewportStore((s) => s.togglePixelGrid);

  const { data: model } = useRenderModel();
  const isEmpty = !model || (model.canvas.width <= 0 && model.layers.length === 0);

  const [spaceHeld, setSpaceHeld] = useState(false);

  if (!paletteRef.current && typeof window !== "undefined") {
    paletteRef.current = readPalette();
  }

  // Keep the canvas sized to its container (and the viewport's screen size in sync).
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const measure = () => {
      const w = container.clientWidth;
      const h = container.clientHeight;
      dprRef.current = window.devicePixelRatio || 1;
      const canvas = canvasRef.current;
      if (canvas) {
        canvas.width = Math.round(w * dprRef.current);
        canvas.height = Math.round(h * dprRef.current);
      }
      setScreen({ width: w, height: h });
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(container);
    return () => ro.disconnect();
  }, [setScreen]);

  // Paint whenever the camera, overlays, document, or size change.
  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || !model || screen.width === 0) return;
    const raf = requestAnimationFrame(() => {
      paintScene({
        ctx,
        model,
        vp: { origin, zoom, screen },
        overlays,
        palette: paletteRef.current ?? readPalette(),
        dpr: dprRef.current,
      });
    });
    return () => cancelAnimationFrame(raf);
  }, [model, origin, zoom, screen, overlays]);

  // Focus the canvas on mount so it's the default editor focus.
  useEffect(() => {
    containerRef.current?.focus({ preventScroll: true });
  }, []);

  // Wheel: pan with trackpad/scroll, zoom with ctrl/cmd (and trackpad pinch).
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const rect = canvas.getBoundingClientRect();
      const pt: Point = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      if (e.ctrlKey || e.metaKey) {
        zoomAt(pt, Math.exp(-e.deltaY * 0.0015));
      } else {
        panByScreen(-e.deltaX, -e.deltaY);
      }
    };
    canvas.addEventListener("wheel", onWheel, { passive: false });
    return () => canvas.removeEventListener("wheel", onWheel);
  }, [zoomAt, panByScreen]);

  const pointerPos = (e: React.PointerEvent): Point => {
    const rect = canvasRef.current!.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  };

  const onPointerDown = (e: React.PointerEvent) => {
    const wantsPan = e.button === 1 || activeTool === "pan" || spaceRef.current;
    if (wantsPan) {
      e.preventDefault();
      setPanning(true);
      (e.target as Element).setPointerCapture(e.pointerId);
      return;
    }
    if (e.button === 0 && activeTool === "zoom") {
      zoomAt(pointerPos(e), e.altKey ? 1 / 1.6 : 1.6);
    }
  };

  const onPointerMove = (e: React.PointerEvent) => {
    if (panning) panByScreen(e.movementX, e.movementY);
  };

  const endPan = (e: React.PointerEvent) => {
    if (!panning) return;
    setPanning(false);
    const el = e.target as Element;
    if (el.hasPointerCapture?.(e.pointerId)) el.releasePointerCapture(e.pointerId);
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === " ") {
      spaceRef.current = true;
      setSpaceHeld(true);
      e.preventDefault();
      return;
    }
    const step = 60;
    switch (e.key) {
      case "+":
      case "=":
        zoomCentered(1.15);
        e.preventDefault();
        break;
      case "-":
      case "_":
        zoomCentered(1 / 1.15);
        e.preventDefault();
        break;
      case "0":
        focus("actual");
        e.preventDefault();
        break;
      case "1":
        focus("fit");
        e.preventDefault();
        break;
      case "ArrowLeft":
        panByScreen(step, 0);
        e.preventDefault();
        break;
      case "ArrowRight":
        panByScreen(-step, 0);
        e.preventDefault();
        break;
      case "ArrowUp":
        panByScreen(0, step);
        e.preventDefault();
        break;
      case "ArrowDown":
        panByScreen(0, -step);
        e.preventDefault();
        break;
    }
  };

  const onKeyUp = (e: React.KeyboardEvent) => {
    if (e.key === " ") {
      spaceRef.current = false;
      setSpaceHeld(false);
    }
  };

  // Drag image files into the workspace to import them (REQ-050).
  const onDragOverFiles = (e: React.DragEvent) => {
    if (!e.dataTransfer.types.includes("Files")) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "copy";
    if (!dragOver) setDragOver(true);
  };
  const onDragLeaveFiles = (e: React.DragEvent) => {
    if (e.currentTarget === e.target) setDragOver(false);
  };
  const onDropFiles = (e: React.DragEvent) => {
    if (!e.dataTransfer.types.includes("Files")) return;
    e.preventDefault();
    setDragOver(false);
    const file = Array.from(e.dataTransfer.files).find((f) => f.type.startsWith("image/"));
    if (file) dropImageFlow(file.name);
  };

  const cursor = panning
    ? "grabbing"
    : activeTool === "pan" || spaceHeld
      ? "grab"
      : activeTool === "zoom"
        ? "zoom-in"
        : "default";

  return (
    <div
      ref={containerRef}
      tabIndex={0}
      role="application"
      aria-label="Workspace canvas"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={endPan}
      onPointerCancel={endPan}
      onKeyDown={onKeyDown}
      onKeyUp={onKeyUp}
      onDragOver={onDragOverFiles}
      onDragLeave={onDragLeaveFiles}
      onDrop={onDropFiles}
      className="relative flex-1 overflow-hidden bg-background outline-none"
      style={{ cursor }}
    >
      <canvas ref={canvasRef} className="absolute inset-0 h-full w-full" />

      {dragOver && (
        <div className="pointer-events-none absolute inset-2 z-20 flex items-center justify-center rounded-xl border-2 border-dashed border-primary bg-primary/5">
          <span className="rounded-md bg-primary px-3 py-1.5 text-[13px] font-medium text-primary-foreground">
            Drop image to import
          </span>
        </div>
      )}

      {isEmpty && <EmptyState onOpenImage={() => openImageFlow()} onNewWorkspace={() => newWorkspace()} />}

      {/* Active tool indicator (top-left) */}
      <div className="pointer-events-none absolute left-3 top-3 flex items-center gap-2 rounded-md border border-border bg-card/80 px-2.5 py-1.5 text-xs backdrop-blur-sm">
        <span className="size-1.5 rounded-full bg-primary" />
        <span className="font-medium capitalize text-foreground">{activeTool.replace("-", " ")}</span>
        <span className="text-muted-foreground">tool active</span>
      </div>

      {/* Overlay toggles (top-right) */}
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="absolute right-3 top-3 flex items-center gap-0.5 rounded-lg border border-border bg-card/90 p-1 shadow-lg backdrop-blur-sm"
      >
        <OverlayToggle active={overlays.checkerboard} label="Transparency checkerboard" onClick={() => toggleOverlay("checkerboard")}>
          <Grid2x2 className="size-4" />
        </OverlayToggle>
        <OverlayToggle active={overlays.guides} label="Guides" onClick={() => toggleOverlay("guides")}>
          <Ruler className="size-4" />
        </OverlayToggle>
        <OverlayToggle active={overlays.pixelGrid.enabled} label="Pixel grid" onClick={togglePixelGrid}>
          <Grid3x3 className="size-4" />
        </OverlayToggle>
        <OverlayToggle active={overlays.selections} label="Selections" onClick={() => toggleOverlay("selections")}>
          <SquareDashed className="size-4" />
        </OverlayToggle>
        <OverlayToggle active={overlays.transformHandles} label="Transform handles" onClick={() => toggleOverlay("transformHandles")}>
          <Move className="size-4" />
        </OverlayToggle>
        <OverlayToggle active={overlays.exportAreas} label="Export areas" onClick={() => toggleOverlay("exportAreas")}>
          <Frame className="size-4" />
        </OverlayToggle>
      </div>

      {/* Zoom controls (bottom-center) */}
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="absolute bottom-3 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-lg border border-border bg-card/90 p-1 shadow-lg backdrop-blur-sm"
      >
        <ControlButton label="Zoom out" onClick={() => zoomCentered(1 / 1.2)}>
          <span className="text-lg leading-none">−</span>
        </ControlButton>
        <button
          onClick={() => focus("actual")}
          className="min-w-[52px] rounded-md px-1.5 py-1 font-mono text-xs text-foreground transition-colors hover:bg-secondary"
          title="Reset to 100%"
        >
          {Math.round(zoom * 100)}%
        </button>
        <ControlButton label="Zoom in" onClick={() => zoomCentered(1.2)}>
          <span className="text-lg leading-none">+</span>
        </ControlButton>

        <div className="mx-1 h-5 w-px bg-border" />

        <ControlButton label="Zoom to fit" onClick={() => focus("fit")}>
          <Maximize2 className="size-4" />
        </ControlButton>
        <button
          onClick={() => focus("pixel-perfect")}
          className="rounded-md px-1.5 py-1 font-mono text-[11px] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          title="Pixel-perfect (integer zoom)"
        >
          1:1
        </button>
        <ControlButton label="Zoom to selection" onClick={() => focus("selection")}>
          <SquareDashed className="size-4" />
        </ControlButton>
        <ControlButton label="Zoom to export area" onClick={() => focus("export-area")}>
          <Frame className="size-4" />
        </ControlButton>
      </div>
    </div>
  );
}

function EmptyState({ onOpenImage, onNewWorkspace }: { onOpenImage: () => void; onNewWorkspace: () => void }) {
  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center p-6">
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="pointer-events-auto flex max-w-sm flex-col items-center gap-5 text-center"
      >
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-foreground">Untitled workspace</h1>
          <p className="mt-1 text-[13px] text-muted-foreground">
            Open an image or start a new workspace to begin. Scroll to pan, ⌘/Ctrl-scroll to zoom.
          </p>
        </div>
        <div className="flex flex-wrap items-center justify-center gap-2">
          <button
            onClick={onOpenImage}
            className="flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[13px] font-medium text-primary-foreground transition-colors hover:bg-primary/90"
          >
            <FolderOpen className="size-4" />
            Open image
          </button>
          <button
            onClick={onNewWorkspace}
            className="flex h-8 items-center gap-1.5 rounded-md border border-border px-3 text-[13px] text-foreground transition-colors hover:bg-secondary"
          >
            <FilePlus2 className="size-4" />
            New workspace
          </button>
        </div>
      </div>
    </div>
  );
}

function OverlayToggle({
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

function ControlButton({
  children,
  onClick,
  label,
}: {
  children: React.ReactNode;
  onClick: () => void;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      aria-label={label}
      title={label}
      className="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
    >
      {children}
    </button>
  );
}
