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
  Copy,
  Trash2,
} from "lucide-react";
import { api } from "@/lib/api";
import { useRenderModel } from "@/lib/queries";
import { paintScene, type Palette } from "@/lib/render";
import type { Point, Rect } from "@/lib/fleck-data";
import { cn } from "@/lib/utils";
import { openImageFlow, dropImageFlow } from "@/lib/image-import";
import { screenToWorkspace, workspaceToScreen } from "@/lib/viewport";
import { DEFAULT_EXPORT_AREA_SIZE } from "@/lib/export-commands";
import { SELECTION_NUDGE, SELECTION_NUDGE_LARGE } from "@/lib/selection-commands";
import { useUIStore } from "@/store/ui-store";
import { useViewportStore } from "@/store/viewport-store";
import { useCommandStore } from "@/store/command-store";
import { useWorkspaceFilesStore } from "@/store/workspace-files-store";
import { SelectionHUD } from "@/components/fleck/selection-hud";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";

type ExportAreaDrag = {
  id: string;
  pointerId: number;
  startScreen: Point;
  startRect: Rect;
  currentRect: Rect;
};

/**
 * In-flight selection geometry being drawn by the user. Committed on pointer-up
 * (marquee / freehand lasso / wand) or on dblclick / Enter (polygon).
 *
 * Stored in workspace coordinates so screen-space repaints stay consistent
 * across zoom/pan while the drag is alive.
 */
type SelectionDraft =
  | { kind: "rect"; pointerId: number; start: Point; current: Point }
  | { kind: "lasso"; pointerId: number; points: Point[] }
  | { kind: "polygon"; points: Point[]; cursor: Point | null };

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
  const marqueeShape = useUIStore((s) => s.marqueeShape);
  const lassoMode = useUIStore((s) => s.lassoMode);
  const selectedExportAreaId = useUIStore((s) => s.selectedExportAreaId);
  const setSelectedExportAreaId = useUIStore((s) => s.setSelectedExportAreaId);
  const setSideTab = useUIStore((s) => s.setSideTab);
  const activeSelectionId = useUIStore((s) => s.activeSelectionId);
  const setActiveSelectionId = useUIStore((s) => s.setActiveSelectionId);
  const execute = useCommandStore((s) => s.execute);
  const newWorkspace = useWorkspaceFilesStore((s) => s.newWorkspace);
  const [dragOver, setDragOver] = useState(false);
  const [assetPaintVersion, setAssetPaintVersion] = useState(0);
  const [areaDrag, setAreaDrag] = useState<ExportAreaDrag | null>(null);
  const [selectionDraft, setSelectionDraft] = useState<SelectionDraft | null>(null);
  // Workspace point of the last right-click, used to place a new export area there.
  const menuPointRef = useRef<Point>({ x: 0, y: 0 });
  const [menuAreaId, setMenuAreaId] = useState<string | null>(null);

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

  const paintModel =
    model && areaDrag
      ? {
          ...model,
          exportAreas: model.exportAreas.map((area) =>
            area.id === areaDrag.id ? { ...area, rect: areaDrag.currentRect } : area,
          ),
        }
      : model;

  // Paint whenever the camera, overlays, document, assets, drag preview, or size change.
  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || !paintModel || screen.width === 0) return;
    const raf = requestAnimationFrame(() => {
      paintScene({
        ctx,
        model: paintModel,
        vp: { origin, zoom, screen },
        overlays,
        palette: paletteRef.current ?? readPalette(),
        dpr: dprRef.current,
        selectedExportAreaId,
        onAssetsChanged: () => setAssetPaintVersion((v) => v + 1),
      });
    });
    return () => cancelAnimationFrame(raf);
  }, [paintModel, origin, zoom, screen, overlays, selectedExportAreaId, assetPaintVersion]);

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

  /** Topmost export area (in workspace coords) under a screen point, if any. */
  const areaAtScreen = (screenPt: Point): { id: string; rect: Rect } | null => {
    if (!model) return null;
    const w = screenToWorkspace({ origin, zoom, screen }, screenPt);
    for (let i = model.exportAreas.length - 1; i >= 0; i--) {
      const a = model.exportAreas[i];
      if (w.x >= a.rect.x && w.x <= a.rect.x + a.rect.width && w.y >= a.rect.y && w.y <= a.rect.y + a.rect.height) {
        return a;
      }
    }
    return null;
  };

  const selectArea = (id: string) => {
    setSelectedExportAreaId(id);
    setSideTab("exports");
  };

  const startAreaDrag = (hit: { id: string; rect: Rect }, e: React.PointerEvent) => {
    const pt = pointerPos(e);
    setAreaDrag({ id: hit.id, pointerId: e.pointerId, startScreen: pt, startRect: hit.rect, currentRect: hit.rect });
    (e.target as Element).setPointerCapture(e.pointerId);
  };

  /** Create a default-sized export area centred on a screen point. */
  const createAreaAt = (screenPt: Point) => {
    const w = screenToWorkspace({ origin, zoom, screen }, screenPt);
    execute("export_area.create", {
      name: "Export area",
      x: Math.round(w.x - DEFAULT_EXPORT_AREA_SIZE.width / 2),
      y: Math.round(w.y - DEFAULT_EXPORT_AREA_SIZE.height / 2),
      width: DEFAULT_EXPORT_AREA_SIZE.width,
      height: DEFAULT_EXPORT_AREA_SIZE.height,
    });
  };

  const workspaceFromScreen = (pt: Point): Point => screenToWorkspace({ origin, zoom, screen }, pt);

  const startSelectionDraft = (e: React.PointerEvent) => {
    const screenPt = pointerPos(e);
    const wp = workspaceFromScreen(screenPt);
    (e.target as Element).setPointerCapture(e.pointerId);
    if (activeTool === "marquee") {
      setSelectionDraft({ kind: "rect", pointerId: e.pointerId, start: wp, current: wp });
    } else if (activeTool === "lasso" && lassoMode === "freehand") {
      setSelectionDraft({ kind: "lasso", pointerId: e.pointerId, points: [wp] });
    }
  };

  const commitMarqueeDraft = (draft: Extract<SelectionDraft, { kind: "rect" }>) => {
    const x = Math.round(Math.min(draft.start.x, draft.current.x));
    const y = Math.round(Math.min(draft.start.y, draft.current.y));
    const width = Math.max(1, Math.round(Math.abs(draft.current.x - draft.start.x)));
    const height = Math.max(1, Math.round(Math.abs(draft.current.y - draft.start.y)));
    const commandId = marqueeShape === "ellipse" ? "selection.ellipse" : "selection.rect";
    execute(commandId, { x, y, width, height });
  };

  const commitLassoDraft = (draft: Extract<SelectionDraft, { kind: "lasso" }>) => {
    // Need at least a triangle for a valid mask; otherwise treat as a wand-style click.
    if (draft.points.length < 3) {
      const point = draft.points[0];
      const x = Math.round(point.x);
      const y = Math.round(point.y);
      execute("selection.rect", { x, y, width: 1, height: 1 });
      return;
    }
    execute("selection.lasso", { points: draft.points.map((p) => ({ x: p.x, y: p.y })) });
  };

  const commitPolygonDraft = (draft: Extract<SelectionDraft, { kind: "polygon" }>) => {
    if (draft.points.length >= 3) {
      execute("selection.polygon", { points: draft.points.map((p) => ({ x: p.x, y: p.y })) });
    }
    setSelectionDraft(null);
  };

  const onPointerDown = (e: React.PointerEvent) => {
    const wantsPan = e.button === 1 || activeTool === "pan" || spaceRef.current;
    if (wantsPan) {
      e.preventDefault();
      setPanning(true);
      (e.target as Element).setPointerCapture(e.pointerId);
      return;
    }
    if (e.button !== 0) return;
    if (activeTool === "zoom") {
      zoomAt(pointerPos(e), e.altKey ? 1 / 1.6 : 1.6);
      return;
    }
    // Export-area interaction is scoped to the export-area tool so clicks with
    // other tools don't hijack selection: clicking an existing area selects it
    // (synced to the panel + inspector), empty space marks a new region.
    if (activeTool === "export-area") {
      const hit = areaAtScreen(pointerPos(e));
      if (hit) {
        selectArea(hit.id);
        startAreaDrag(hit, e);
      } else {
        createAreaAt(pointerPos(e));
      }
      return;
    }
    if (activeTool === "marquee" || (activeTool === "lasso" && lassoMode === "freehand")) {
      startSelectionDraft(e);
      return;
    }
    if (activeTool === "lasso" && lassoMode === "polygon") {
      const wp = workspaceFromScreen(pointerPos(e));
      setSelectionDraft((prev) =>
        prev?.kind === "polygon"
          ? { ...prev, points: [...prev.points, wp], cursor: wp }
          : { kind: "polygon", points: [wp], cursor: wp },
      );
      return;
    }
    if (activeTool === "wand") {
      const wp = workspaceFromScreen(pointerPos(e));
      execute("selection.magic_wand", {
        x: Math.round(wp.x),
        y: Math.round(wp.y),
        width: 1,
        height: 1,
      });
      return;
    }
  };

  const onContextMenu = (e: React.PointerEvent | React.MouseEvent) => {
    const pt = pointerPos(e as React.PointerEvent);
    menuPointRef.current = pt;
    const hit = areaAtScreen(pt);
    setMenuAreaId(hit?.id ?? null);
    if (hit) setSelectedExportAreaId(hit.id);
  };

  const onPointerMove = (e: React.PointerEvent) => {
    if (areaDrag && e.pointerId === areaDrag.pointerId) {
      const current = screenToWorkspace({ origin, zoom, screen }, pointerPos(e));
      const start = screenToWorkspace({ origin, zoom, screen }, areaDrag.startScreen);
      const dx = current.x - start.x;
      const dy = current.y - start.y;
      setAreaDrag({
        ...areaDrag,
        currentRect: {
          ...areaDrag.startRect,
          x: areaDrag.startRect.x + dx,
          y: areaDrag.startRect.y + dy,
        },
      });
      return;
    }
    if (selectionDraft) {
      const wp = workspaceFromScreen(pointerPos(e));
      if (selectionDraft.kind === "rect" && e.pointerId === selectionDraft.pointerId) {
        setSelectionDraft({ ...selectionDraft, current: wp });
        return;
      }
      if (selectionDraft.kind === "lasso" && e.pointerId === selectionDraft.pointerId) {
        setSelectionDraft({ ...selectionDraft, points: [...selectionDraft.points, wp] });
        return;
      }
      if (selectionDraft.kind === "polygon") {
        setSelectionDraft({ ...selectionDraft, cursor: wp });
        return;
      }
    }
    if (panning) panByScreen(e.movementX, e.movementY);
  };

  const endPointer = (e: React.PointerEvent) => {
    const el = e.target as Element;
    if (areaDrag && e.pointerId === areaDrag.pointerId) {
      const moved =
        Math.round(areaDrag.currentRect.x) !== Math.round(areaDrag.startRect.x) ||
        Math.round(areaDrag.currentRect.y) !== Math.round(areaDrag.startRect.y);
      if (moved) {
        execute("export_area.move", {
          id: areaDrag.id,
          x: Math.round(areaDrag.currentRect.x),
          y: Math.round(areaDrag.currentRect.y),
        });
      }
      setAreaDrag(null);
      if (el.hasPointerCapture?.(e.pointerId)) el.releasePointerCapture(e.pointerId);
      return;
    }
    if (selectionDraft && selectionDraft.kind !== "polygon" && e.pointerId === selectionDraft.pointerId) {
      if (selectionDraft.kind === "rect") commitMarqueeDraft(selectionDraft);
      else if (selectionDraft.kind === "lasso") commitLassoDraft(selectionDraft);
      setSelectionDraft(null);
      if (el.hasPointerCapture?.(e.pointerId)) el.releasePointerCapture(e.pointerId);
      return;
    }
    if (panning) {
      setPanning(false);
      if (el.hasPointerCapture?.(e.pointerId)) el.releasePointerCapture(e.pointerId);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === " ") {
      spaceRef.current = true;
      setSpaceHeld(true);
      e.preventDefault();
      return;
    }

    // Selection keyboard surface: arrows nudge the active selection (Shift =
    // larger nudge), Delete clears it, ⌘C / Ctrl+C copies. Falls through to
    // pan/zoom shortcuts when no selection is active.
    const nudgeDir: [number, number] | null =
      e.key === "ArrowLeft"
        ? [-1, 0]
        : e.key === "ArrowRight"
          ? [1, 0]
          : e.key === "ArrowUp"
            ? [0, -1]
            : e.key === "ArrowDown"
              ? [0, 1]
              : null;
    if (activeSelectionId && nudgeDir) {
      const amount = e.shiftKey ? SELECTION_NUDGE_LARGE : SELECTION_NUDGE;
      execute("selection.move", { dx: nudgeDir[0] * amount, dy: nudgeDir[1] * amount });
      e.preventDefault();
      return;
    }
    if (activeSelectionId && (e.key === "Delete" || e.key === "Backspace")) {
      execute("selection.delete");
      e.preventDefault();
      return;
    }
    if (activeSelectionId && (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "c") {
      execute("selection.copy");
      e.preventDefault();
      return;
    }
    if (selectionDraft?.kind === "polygon") {
      if (e.key === "Enter") {
        commitPolygonDraft(selectionDraft);
        e.preventDefault();
        return;
      }
      if (e.key === "Escape") {
        setSelectionDraft(null);
        e.preventDefault();
        return;
      }
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

  const onDoubleClick = () => {
    if (selectionDraft?.kind === "polygon") commitPolygonDraft(selectionDraft);
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
    if (file) void dropImageFlow(file);
  };

  const cursor = panning
    ? "grabbing"
    : areaDrag
      ? "move"
    : activeTool === "pan" || spaceHeld
      ? "grab"
      : activeTool === "zoom"
        ? "zoom-in"
        : "default";

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
    <div
      ref={containerRef}
      tabIndex={0}
      role="application"
      aria-label="Workspace canvas"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={endPointer}
      onPointerCancel={endPointer}
      onContextMenu={onContextMenu}
      onKeyDown={onKeyDown}
      onKeyUp={onKeyUp}
      onDoubleClick={onDoubleClick}
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

      <SelectionDraftOverlay
        draft={selectionDraft}
        marqueeShape={marqueeShape}
        vp={{ origin, zoom, screen }}
      />

      <SelectionHUD />

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
        <ControlButton label="Zoom to export area" onClick={() => focus("export-area", selectedExportAreaId)}>
          <Frame className="size-4" />
        </ControlButton>
      </div>
    </div>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onSelect={() => createAreaAt(menuPointRef.current)}>
          <Frame />
          New export area here
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => api.exportAll()}>
          <FilePlus2 />
          Export all areas
          <ContextMenuShortcut>⌘⇧E</ContextMenuShortcut>
        </ContextMenuItem>
        {menuAreaId && (
          <>
            <ContextMenuSeparator />
            <ContextMenuItem onSelect={() => api.exportArea(menuAreaId)}>
              <Frame />
              Export this area
              <ContextMenuShortcut>⌘E</ContextMenuShortcut>
            </ContextMenuItem>
            <ContextMenuItem
              onSelect={() => {
                selectArea(menuAreaId);
                focus("export-area", menuAreaId);
              }}
            >
              <Maximize2 />
              Zoom to area
            </ContextMenuItem>
            <ContextMenuItem onSelect={() => execute("export_area.duplicate", { id: menuAreaId })}>
              <Copy />
              Duplicate area
            </ContextMenuItem>
            <ContextMenuSeparator />
            <ContextMenuItem
              variant="destructive"
              onSelect={() => {
                execute("export_area.delete", { id: menuAreaId });
                if (selectedExportAreaId === menuAreaId) setSelectedExportAreaId(null);
              }}
            >
              <Trash2 />
              Delete area
            </ContextMenuItem>
          </>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
}

/**
 * Lightweight overlay that previews the in-flight selection (rect/ellipse for
 * marquee, polyline for lasso/polygon). Workspace coordinates are projected on
 * every render so pan/zoom keep the preview aligned with the canvas paint.
 */
function SelectionDraftOverlay({
  draft,
  marqueeShape,
  vp,
}: {
  draft: SelectionDraft | null;
  marqueeShape: "rect" | "ellipse";
  vp: { origin: Point; zoom: number; screen: { width: number; height: number } };
}) {
  if (!draft) return null;

  if (draft.kind === "rect") {
    const a = workspaceToScreen(vp, draft.start);
    const b = workspaceToScreen(vp, draft.current);
    const left = Math.min(a.x, b.x);
    const top = Math.min(a.y, b.y);
    const width = Math.abs(b.x - a.x);
    const height = Math.abs(b.y - a.y);
    return (
      <div
        aria-hidden="true"
        className={cn(
          "pointer-events-none absolute border border-dashed border-primary bg-primary/10",
          marqueeShape === "ellipse" && "rounded-[50%]",
        )}
        style={{ left, top, width, height }}
      />
    );
  }

  const points = draft.kind === "lasso" ? draft.points : draft.points;
  const screenPts = points.map((p) => workspaceToScreen(vp, p));
  const polyline = screenPts.map((p) => `${p.x},${p.y}`).join(" ");
  const cursor = draft.kind === "polygon" && draft.cursor ? workspaceToScreen(vp, draft.cursor) : null;
  return (
    <svg
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 h-full w-full"
      style={{ overflow: "visible" }}
    >
      <polyline points={polyline} fill="rgba(124,156,255,0.10)" stroke="hsl(var(--primary))" strokeDasharray="4 3" strokeWidth={1} />
      {cursor && screenPts.length > 0 && (
        <line
          x1={screenPts[screenPts.length - 1].x}
          y1={screenPts[screenPts.length - 1].y}
          x2={cursor.x}
          y2={cursor.y}
          stroke="hsl(var(--primary))"
          strokeDasharray="2 3"
          strokeWidth={1}
        />
      )}
      {draft.kind === "polygon" &&
        screenPts.map((p, i) => <circle key={i} cx={p.x} cy={p.y} r={3} fill="hsl(var(--primary))" />)}
    </svg>
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
