/**
 * Viewport camera math — the frontend mirror of `fleck-core::geometry::Viewport`.
 *
 * The camera (pan/zoom) is presentation state, not document truth, so it lives
 * on the frontend for responsive interaction. These pure helpers keep the
 * transforms identical to the core so screen↔workspace conversions agree.
 */
import type { Point, Rect, Size, Viewport } from "./fleck-data";

export const MIN_ZOOM = 0.02;
export const MAX_ZOOM = 64;
/** Pixel grid only appears at/above this zoom (matches DEFAULT_MIN_PIXEL_GRID_ZOOM). */
export const MIN_PIXEL_GRID_ZOOM = 8;

export const clampZoom = (zoom: number): number => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));

export function workspaceToScreen(vp: Viewport, p: Point): Point {
  return { x: (p.x - vp.origin.x) * vp.zoom, y: (p.y - vp.origin.y) * vp.zoom };
}

export function screenToWorkspace(vp: Viewport, p: Point): Point {
  return { x: p.x / vp.zoom + vp.origin.x, y: p.y / vp.zoom + vp.origin.y };
}

/** Pan by a screen-space delta (e.g. a drag). */
export function panByScreenDelta(vp: Viewport, dx: number, dy: number): Viewport {
  return { ...vp, origin: { x: vp.origin.x - dx / vp.zoom, y: vp.origin.y - dy / vp.zoom } };
}

/** Zoom to `newZoom` while keeping the workspace point under `screen` fixed. */
export function zoomAroundScreenPoint(vp: Viewport, screen: Point, newZoom: number): Viewport {
  const zoom = clampZoom(newZoom);
  const anchor = screenToWorkspace(vp, screen);
  return {
    ...vp,
    zoom,
    origin: { x: anchor.x - screen.x / zoom, y: anchor.y - screen.y / zoom },
  };
}

export function visibleWorkspaceRect(vp: Viewport): Rect {
  return {
    x: vp.origin.x,
    y: vp.origin.y,
    width: vp.screen.width / vp.zoom,
    height: vp.screen.height / vp.zoom,
  };
}

/** Integer workspace coordinates within [start, end] (for the pixel grid). */
export function integerLines(start: number, end: number): number[] {
  const lines: number[] = [];
  for (let i = Math.floor(start); i <= Math.ceil(end); i++) lines.push(i);
  return lines;
}

/** A viewport that fits `rect` (workspace units) into `screen` with padding. */
export function fitRect(rect: Rect, screen: Size, padding = 0.85): Viewport {
  if (rect.width <= 0 || rect.height <= 0 || screen.width <= 0 || screen.height <= 0) {
    return { origin: { x: 0, y: 0 }, zoom: 1, screen };
  }
  const zoom = clampZoom(Math.min(screen.width / rect.width, screen.height / rect.height) * padding);
  const centerX = rect.x + rect.width / 2;
  const centerY = rect.y + rect.height / 2;
  return {
    origin: { x: centerX - screen.width / 2 / zoom, y: centerY - screen.height / 2 / zoom },
    zoom,
    screen,
  };
}
