/**
 * Canvas-2D painter — the frontend stand-in for `fleck-render`.
 *
 * It takes the read-only render model (workspace coordinates), the camera, and
 * overlay settings, then paints a frame. When the native render bridge lands,
 * this is replaced by blitting the backend's RGBA `RenderedFrame`; the host
 * component and its inputs stay the same.
 */
import type { OverlaySettings, Rect, RenderModel, Viewport } from "./fleck-data";
import { integerLines, visibleWorkspaceRect, workspaceToScreen } from "./viewport";

export type Palette = {
  gridDot: string;
  canvasBorder: string;
  layerStroke: string;
  exportArea: string;
  exportLabelText: string;
  guide: string;
  selection: string;
  handleFill: string;
  handleBorder: string;
  pixelGrid: string;
};

const CHECKER_A = "rgba(255, 255, 255, 0.09)";
const CHECKER_B = "rgba(255, 255, 255, 0.03)";
const CHECKER_CELL = 10;

type PaintArgs = {
  ctx: CanvasRenderingContext2D;
  model: RenderModel;
  vp: Viewport;
  overlays: OverlaySettings;
  palette: Palette;
  dpr: number;
};

export function paintScene({ ctx, model, vp, overlays, palette, dpr }: PaintArgs) {
  const { width, height } = vp.screen;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  drawReferenceGrid(ctx, vp, palette.gridDot);

  if (model.canvas.width > 0 && model.canvas.height > 0) {
    const canvasRect = toScreenRect(vp, { x: 0, y: 0, width: model.canvas.width, height: model.canvas.height });
    if (overlays.checkerboard) drawCheckerboard(ctx, canvasRect);

    for (const layer of model.layers) {
      if (!layer.visible) continue;
      const r = toScreenRect(vp, layer.rect);
      ctx.globalAlpha = layer.opacity;
      ctx.fillStyle = layer.color;
      ctx.fillRect(r.x, r.y, r.width, r.height);
    }
    ctx.globalAlpha = 1;

    // Canvas bounds outline on top of content.
    ctx.strokeStyle = palette.canvasBorder;
    ctx.lineWidth = 1;
    ctx.strokeRect(canvasRect.x + 0.5, canvasRect.y + 0.5, canvasRect.width, canvasRect.height);
  }

  if (overlays.exportAreas) drawExportAreas(ctx, model, vp, palette);
  if (overlays.selections) drawSelections(ctx, model, vp, palette);
  if (overlays.transformHandles) drawTransformHandles(ctx, model, vp, palette);
  if (overlays.guides) drawGuides(ctx, model, vp, palette);
  if (overlays.pixelGrid.enabled && vp.zoom >= overlays.pixelGrid.minZoom) {
    drawPixelGrid(ctx, vp, palette.pixelGrid);
  }
}

function toScreenRect(vp: Viewport, rect: Rect): Rect {
  const tl = workspaceToScreen(vp, { x: rect.x, y: rect.y });
  return { x: tl.x, y: tl.y, width: rect.width * vp.zoom, height: rect.height * vp.zoom };
}

/** Choose a workspace grid step whose on-screen spacing reads well. */
function niceStep(zoom: number): number {
  const target = 24; // desired px between dots
  const steps = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000];
  for (const step of steps) {
    if (step * zoom >= target) return step;
  }
  return steps[steps.length - 1];
}

function drawReferenceGrid(ctx: CanvasRenderingContext2D, vp: Viewport, color: string) {
  const step = niceStep(vp.zoom);
  const rect = visibleWorkspaceRect(vp);
  const startX = Math.floor(rect.x / step) * step;
  const startY = Math.floor(rect.y / step) * step;
  ctx.fillStyle = color;
  for (let wx = startX; wx <= rect.x + rect.width; wx += step) {
    for (let wy = startY; wy <= rect.y + rect.height; wy += step) {
      const p = workspaceToScreen(vp, { x: wx, y: wy });
      ctx.beginPath();
      ctx.arc(p.x, p.y, 1, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

function drawCheckerboard(ctx: CanvasRenderingContext2D, rect: Rect) {
  ctx.save();
  ctx.beginPath();
  ctx.rect(rect.x, rect.y, rect.width, rect.height);
  ctx.clip();
  const cols = Math.ceil(rect.width / CHECKER_CELL);
  const rows = Math.ceil(rect.height / CHECKER_CELL);
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      ctx.fillStyle = (row + col) % 2 === 0 ? CHECKER_A : CHECKER_B;
      ctx.fillRect(rect.x + col * CHECKER_CELL, rect.y + row * CHECKER_CELL, CHECKER_CELL, CHECKER_CELL);
    }
  }
  ctx.restore();
}

function drawExportAreas(ctx: CanvasRenderingContext2D, model: RenderModel, vp: Viewport, palette: Palette) {
  ctx.save();
  ctx.strokeStyle = palette.exportArea;
  ctx.lineWidth = 1.5;
  ctx.setLineDash([5, 4]);
  ctx.font = "11px ui-monospace, monospace";
  for (const area of model.exportAreas) {
    const r = toScreenRect(vp, area.rect);
    ctx.strokeRect(r.x, r.y, r.width, r.height);
    ctx.setLineDash([]);
    ctx.fillStyle = palette.exportArea;
    ctx.fillRect(r.x, r.y - 16, ctx.measureText(area.name).width + 10, 15);
    ctx.fillStyle = palette.exportLabelText;
    ctx.fillText(area.name, r.x + 5, r.y - 5);
    ctx.setLineDash([5, 4]);
  }
  ctx.restore();
}

function drawSelections(ctx: CanvasRenderingContext2D, model: RenderModel, vp: Viewport, palette: Palette) {
  ctx.save();
  ctx.strokeStyle = palette.selection;
  ctx.lineWidth = 1;
  ctx.setLineDash([4, 3]);
  for (const sel of model.selections) {
    const r = toScreenRect(vp, sel.rect);
    ctx.strokeRect(r.x + 0.5, r.y + 0.5, r.width, r.height);
  }
  ctx.restore();
}

function drawTransformHandles(ctx: CanvasRenderingContext2D, model: RenderModel, vp: Viewport, palette: Palette) {
  const size = 7;
  for (const sel of model.selections) {
    const r = toScreenRect(vp, sel.rect);
    const corners = [
      [r.x, r.y],
      [r.x + r.width, r.y],
      [r.x, r.y + r.height],
      [r.x + r.width, r.y + r.height],
    ];
    for (const [x, y] of corners) {
      ctx.fillStyle = palette.handleFill;
      ctx.strokeStyle = palette.handleBorder;
      ctx.lineWidth = 1;
      ctx.fillRect(x - size / 2, y - size / 2, size, size);
      ctx.strokeRect(x - size / 2 + 0.5, y - size / 2 + 0.5, size, size);
    }
  }
}

function drawGuides(ctx: CanvasRenderingContext2D, model: RenderModel, vp: Viewport, palette: Palette) {
  ctx.save();
  ctx.strokeStyle = palette.guide;
  ctx.lineWidth = 1;
  for (const guide of model.guides) {
    ctx.beginPath();
    if (guide.axis === "vertical") {
      const x = workspaceToScreen(vp, { x: guide.position, y: 0 }).x + 0.5;
      ctx.moveTo(x, 0);
      ctx.lineTo(x, vp.screen.height);
    } else {
      const y = workspaceToScreen(vp, { x: 0, y: guide.position }).y + 0.5;
      ctx.moveTo(0, y);
      ctx.lineTo(vp.screen.width, y);
    }
    ctx.stroke();
  }
  ctx.restore();
}

function drawPixelGrid(ctx: CanvasRenderingContext2D, vp: Viewport, color: string) {
  const rect = visibleWorkspaceRect(vp);
  ctx.save();
  ctx.strokeStyle = color;
  ctx.lineWidth = 1;
  ctx.globalAlpha = 0.5;
  for (const wx of integerLines(rect.x, rect.x + rect.width)) {
    const x = workspaceToScreen(vp, { x: wx, y: 0 }).x + 0.5;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, vp.screen.height);
    ctx.stroke();
  }
  for (const wy of integerLines(rect.y, rect.y + rect.height)) {
    const y = workspaceToScreen(vp, { x: 0, y: wy }).y + 0.5;
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(vp.screen.width, y);
    ctx.stroke();
  }
  ctx.restore();
}
