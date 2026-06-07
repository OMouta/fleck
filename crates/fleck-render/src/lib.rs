//! Skia-backed viewport rendering for Fleck.
//!
//! This crate consumes `fleck-core` document state and viewport geometry. It
//! does not store, mutate, or become the owner of workspace truth.

use fleck_core::geometry::{guide_lines, pixel_grid_for_rect, OverlaySettings, Viewport};
use fleck_core::model::{Axis, CanvasBackground, Layer, Point, Rect, RgbaColor, Workspace};
use skia_safe::surfaces;
use skia_safe::{
    AlphaType, Canvas, Color, ColorType, IPoint, ImageInfo, Paint, PaintStyle, Rect as SkRect,
    Surface,
};

const CHECKERBOARD_CELL_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy)]
pub struct RenderRequest<'a> {
    pub workspace: &'a Workspace,
    pub viewport: Viewport,
    pub overlays: &'a OverlaySettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub pixels_rgba: Vec<u8>,
    pub overlay_summary: OverlaySummary,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OverlaySummary {
    pub checkerboard_drawn: bool,
    pub export_area_count: usize,
    pub guide_count: usize,
    pub selection_count: usize,
    pub transform_handle_count: usize,
    pub pixel_grid_vertical_lines: usize,
    pub pixel_grid_horizontal_lines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RenderError {
    #[error("render target is too large: {width}x{height}")]
    TargetTooLarge { width: u32, height: u32 },
    #[error("could not create Skia raster surface for {width}x{height}")]
    SurfaceCreationFailed { width: u32, height: u32 },
    #[error("could not read pixels from Skia surface")]
    PixelReadFailed,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkiaViewportRenderer;

impl SkiaViewportRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, request: RenderRequest<'_>) -> Result<RenderedFrame, RenderError> {
        let width = dimension_to_pixels(request.viewport.screen_size.width);
        let height = dimension_to_pixels(request.viewport.screen_size.height);
        let mut surface = create_surface(width, height)?;
        let canvas = surface.canvas();
        canvas.clear(canvas_clear_color(&request.workspace.canvas.background));

        let mut summary = OverlaySummary::default();
        if request.overlays.checkerboard {
            draw_checkerboard(canvas, width, height);
            summary.checkerboard_drawn = true;
        }

        for (index, layer) in request
            .workspace
            .layers
            .iter()
            .filter(|layer| layer.visible && layer.opacity > 0.0)
            .enumerate()
        {
            draw_layer_preview(canvas, &request.viewport, layer, index);
        }

        draw_overlays(canvas, request, &mut summary);

        let pixels_rgba = read_surface_pixels(&mut surface, width, height)?;
        Ok(RenderedFrame {
            width,
            height,
            pixels_rgba,
            overlay_summary: summary,
        })
    }
}

pub fn renderer_boundary_summary() -> &'static str {
    "fleck-render renders core-owned document state"
}

fn dimension_to_pixels(value: f32) -> u32 {
    value.ceil().max(1.0) as u32
}

fn create_surface(width: u32, height: u32) -> Result<Surface, RenderError> {
    let width_i32 =
        i32::try_from(width).map_err(|_| RenderError::TargetTooLarge { width, height })?;
    let height_i32 =
        i32::try_from(height).map_err(|_| RenderError::TargetTooLarge { width, height })?;

    surfaces::raster_n32_premul((width_i32, height_i32))
        .ok_or(RenderError::SurfaceCreationFailed { width, height })
}

fn canvas_clear_color(background: &CanvasBackground) -> Color {
    match background {
        CanvasBackground::Transparent => Color::TRANSPARENT,
        CanvasBackground::Solid { color } => color_to_skia(*color),
    }
}

fn draw_checkerboard(canvas: &Canvas, width: u32, height: u32) {
    let mut paint = Paint::default();
    let width = width as f32;
    let height = height as f32;
    let cells_x = (width / CHECKERBOARD_CELL_SIZE).ceil() as u32;
    let cells_y = (height / CHECKERBOARD_CELL_SIZE).ceil() as u32;

    for y in 0..cells_y {
        for x in 0..cells_x {
            let shade = if (x + y) % 2 == 0 { 222 } else { 184 };
            paint.set_color(Color::from_argb(255, shade, shade, shade));
            canvas.draw_rect(
                SkRect::from_xywh(
                    x as f32 * CHECKERBOARD_CELL_SIZE,
                    y as f32 * CHECKERBOARD_CELL_SIZE,
                    CHECKERBOARD_CELL_SIZE,
                    CHECKERBOARD_CELL_SIZE,
                ),
                &paint,
            );
        }
    }
}

fn draw_layer_preview(canvas: &Canvas, viewport: &Viewport, layer: &Layer, index: usize) {
    if let Some(rect) = layer_rect_to_screen(viewport, layer) {
        let mut paint = Paint::default();
        paint.set_anti_alias(false);
        paint.set_color(layer_preview_color(index, layer.opacity));
        canvas.draw_rect(rect, &paint);
    }
}

fn draw_overlays(canvas: &Canvas, request: RenderRequest<'_>, summary: &mut OverlaySummary) {
    if request.overlays.export_areas {
        summary.export_area_count = request.workspace.export_areas.len();
        let mut paint = stroke_paint(Color::from_argb(220, 37, 99, 235), 1.5);
        for export_area in &request.workspace.export_areas {
            if let Some(rect) = rect_to_screen(request.viewport, export_area.bounds) {
                canvas.draw_rect(rect, &paint);
            }
        }
        paint.set_style(PaintStyle::Fill);
    }

    if request.overlays.guides {
        let mut paint = stroke_paint(Color::from_argb(210, 245, 158, 11), 1.0);
        summary.guide_count = request.workspace.guides.len();
        for guide in guide_lines(request.workspace) {
            match guide.axis {
                Axis::Horizontal => {
                    let y = request
                        .viewport
                        .workspace_to_screen(Point {
                            x: 0.0,
                            y: guide.position,
                        })
                        .y;
                    canvas.draw_line((0.0, y), (request.viewport.screen_size.width, y), &paint);
                }
                Axis::Vertical => {
                    let x = request
                        .viewport
                        .workspace_to_screen(Point {
                            x: guide.position,
                            y: 0.0,
                        })
                        .x;
                    canvas.draw_line((x, 0.0), (x, request.viewport.screen_size.height), &paint);
                }
            }
        }
        paint.set_style(PaintStyle::Fill);
    }

    if request.overlays.selections {
        summary.selection_count = request.workspace.selections.len();
        let paint = stroke_paint(Color::from_argb(230, 255, 255, 255), 1.0);
        for selection in &request.workspace.selections {
            if let Some(rect) = rect_to_screen(request.viewport, selection.bounds) {
                canvas.draw_rect(rect, &paint);
            }
        }
    }

    if request.overlays.transform_handles {
        let mut paint = Paint::default();
        paint.set_color(Color::from_argb(245, 255, 255, 255));
        for selection in &request.workspace.selections {
            if let Some(rect) = rect_to_screen(request.viewport, selection.bounds) {
                summary.transform_handle_count += draw_transform_handles(canvas, rect, &paint);
            }
        }
    }

    let pixel_grid = pixel_grid_for_rect(
        request.viewport.visible_workspace_rect(),
        request.viewport.zoom,
        &request.overlays.pixel_grid,
    );
    summary.pixel_grid_vertical_lines = pixel_grid.vertical_lines.len();
    summary.pixel_grid_horizontal_lines = pixel_grid.horizontal_lines.len();
    if !pixel_grid.vertical_lines.is_empty() || !pixel_grid.horizontal_lines.is_empty() {
        let paint = stroke_paint(Color::from_argb(90, 20, 20, 20), 1.0);
        for workspace_x in pixel_grid.vertical_lines {
            let x = request
                .viewport
                .workspace_to_screen(Point {
                    x: workspace_x,
                    y: 0.0,
                })
                .x;
            canvas.draw_line((x, 0.0), (x, request.viewport.screen_size.height), &paint);
        }
        for workspace_y in pixel_grid.horizontal_lines {
            let y = request
                .viewport
                .workspace_to_screen(Point {
                    x: 0.0,
                    y: workspace_y,
                })
                .y;
            canvas.draw_line((0.0, y), (request.viewport.screen_size.width, y), &paint);
        }
    }
}

fn draw_transform_handles(canvas: &Canvas, rect: SkRect, paint: &Paint) -> usize {
    const HANDLE_SIZE: f32 = 6.0;
    let points = [
        (rect.left, rect.top),
        (rect.right, rect.top),
        (rect.left, rect.bottom),
        (rect.right, rect.bottom),
    ];

    for (x, y) in points {
        canvas.draw_rect(
            SkRect::from_xywh(
                x - HANDLE_SIZE / 2.0,
                y - HANDLE_SIZE / 2.0,
                HANDLE_SIZE,
                HANDLE_SIZE,
            ),
            paint,
        );
    }

    points.len()
}

fn layer_rect_to_screen(viewport: &Viewport, layer: &Layer) -> Option<SkRect> {
    let workspace_rect = Rect {
        x: layer.position.x + layer.bounds.x,
        y: layer.position.y + layer.bounds.y,
        width: layer.bounds.width * layer.transform.scale_x,
        height: layer.bounds.height * layer.transform.scale_y,
    };
    rect_to_screen(*viewport, workspace_rect)
}

fn rect_to_screen(viewport: Viewport, rect: Rect) -> Option<SkRect> {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }

    let top_left = viewport.workspace_to_screen(Point {
        x: rect.x,
        y: rect.y,
    });
    Some(SkRect::from_xywh(
        top_left.x,
        top_left.y,
        rect.width * viewport.zoom,
        rect.height * viewport.zoom,
    ))
}

fn stroke_paint(color: Color, width: f32) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    paint.set_color(color);
    paint
}

fn color_to_skia(color: RgbaColor) -> Color {
    Color::from_argb(color.a, color.r, color.g, color.b)
}

fn layer_preview_color(index: usize, opacity: f32) -> Color {
    let palette = [
        (58, 134, 255),
        (6, 214, 160),
        (255, 190, 11),
        (239, 71, 111),
        (131, 56, 236),
    ];
    let (r, g, b) = palette[index % palette.len()];
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color::from_argb(alpha, r, g, b)
}

fn read_surface_pixels(
    surface: &mut Surface,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, RenderError> {
    let mut pixels = vec![0; width as usize * height as usize * 4];
    let image_info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let row_bytes = width as usize * 4;
    let success = surface.image_snapshot().read_pixels(
        &image_info,
        pixels.as_mut_slice(),
        row_bytes,
        IPoint::new(0, 0),
        skia_safe::image::CachingHint::Allow,
    );
    if success {
        Ok(pixels)
    } else {
        Err(RenderError::PixelReadFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fleck_core::geometry::PixelGridSettings;
    use fleck_core::model::{
        BlendMode, ClippingBehavior, ExportArea, ExportBackground, ExportParticipation, Guide,
        ObjectId, Padding, Selection, SelectionKind, Size, Transform, TrimBehavior,
    };

    #[test]
    fn renderer_boundary_names_core_ownership() {
        assert!(renderer_boundary_summary().contains("core-owned document state"));
    }

    #[test]
    fn renders_checkerboard_and_layer_preview_pixels() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 16.0, 16.0, 1.0));
        let frame = render(&workspace, viewport(0.0, 0.0, 1.0, 32.0, 32.0));

        assert_eq!(frame.width, 32);
        assert_eq!(frame.height, 32);
        assert_eq!(frame.pixels_rgba.len(), 32 * 32 * 4);
        assert!(frame.overlay_summary.checkerboard_drawn);
        assert_ne!(pixel_at(&frame, 2, 2), pixel_at(&frame, 20, 2));
        assert_eq!(pixel_at(&frame, 8, 8)[3], 255);
    }

    #[test]
    fn composites_layers_with_alpha() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 16.0, 16.0, 1.0));
        workspace
            .layers
            .push(layer("top", 0.0, 0.0, 16.0, 16.0, 0.5));
        let frame = render_without_overlays(&workspace, viewport(0.0, 0.0, 1.0, 24.0, 24.0));

        let composited = pixel_at(&frame, 8, 8);
        assert!(composited[1] > composited[0]);
        assert_eq!(composited[3], 255);
    }

    #[test]
    fn pan_and_zoom_move_rendered_content() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("tile", 10.0, 10.0, 10.0, 10.0, 1.0));

        let unpanned = render_without_overlays(&workspace, viewport(0.0, 0.0, 1.0, 32.0, 32.0));
        let panned = render_without_overlays(&workspace, viewport(10.0, 10.0, 2.0, 32.0, 32.0));

        assert_eq!(pixel_at(&unpanned, 5, 5), [0, 0, 0, 0]);
        assert_ne!(pixel_at(&unpanned, 12, 12), [0, 0, 0, 0]);
        assert_ne!(pixel_at(&panned, 5, 5), [0, 0, 0, 0]);
        assert_eq!(pixel_at(&panned, 25, 25), [0, 0, 0, 0]);
    }

    #[test]
    fn reports_and_draws_overlay_hooks() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 8.0, 8.0, 1.0));
        workspace.guides.push(Guide {
            id: id("guide-x"),
            axis: Axis::Vertical,
            position: 4.0,
            locked: false,
        });
        workspace.selections.push(Selection {
            id: id("selection"),
            kind: SelectionKind::Rectangular,
            bounds: rect(0.0, 0.0, 8.0, 8.0),
            feather_radius: 0.0,
            source_layer_ids: vec![id("base")],
        });
        workspace.export_areas.push(ExportArea {
            id: id("export"),
            name: "Export".to_owned(),
            bounds: rect(0.0, 0.0, 8.0, 8.0),
            padding: Padding::default(),
            background: ExportBackground::Transparent,
            trim: TrimBehavior::None,
            output_ids: Vec::new(),
            included_layer_ids: Vec::new(),
            excluded_layer_ids: Vec::new(),
            tags: Vec::new(),
            preset_id: None,
        });
        let overlays = OverlaySettings {
            pixel_grid: PixelGridSettings {
                enabled: true,
                min_zoom: 2.0,
            },
            ..OverlaySettings::default()
        };

        let frame = SkiaViewportRenderer::new()
            .render(RenderRequest {
                workspace: &workspace,
                viewport: viewport(0.0, 0.0, 2.0, 24.0, 24.0),
                overlays: &overlays,
            })
            .expect("render succeeds");

        assert_eq!(frame.overlay_summary.guide_count, 1);
        assert_eq!(frame.overlay_summary.selection_count, 1);
        assert_eq!(frame.overlay_summary.export_area_count, 1);
        assert_eq!(frame.overlay_summary.transform_handle_count, 4);
        assert!(frame.overlay_summary.pixel_grid_vertical_lines > 0);
        assert!(frame.overlay_summary.pixel_grid_horizontal_lines > 0);
    }

    #[test]
    fn rendering_does_not_mutate_workspace() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 8.0, 8.0, 1.0));
        let before = workspace.clone();

        let _ = render(&workspace, viewport(0.0, 0.0, 1.0, 16.0, 16.0));

        assert_eq!(workspace, before);
    }

    fn render(workspace: &Workspace, viewport: Viewport) -> RenderedFrame {
        SkiaViewportRenderer::new()
            .render(RenderRequest {
                workspace,
                viewport,
                overlays: &OverlaySettings::default(),
            })
            .expect("render succeeds")
    }

    fn render_without_overlays(workspace: &Workspace, viewport: Viewport) -> RenderedFrame {
        let overlays = OverlaySettings {
            checkerboard: false,
            guides: false,
            pixel_grid: PixelGridSettings {
                enabled: false,
                min_zoom: f32::MAX,
            },
            selections: false,
            transform_handles: false,
            export_areas: false,
        };
        SkiaViewportRenderer::new()
            .render(RenderRequest {
                workspace,
                viewport,
                overlays: &overlays,
            })
            .expect("render succeeds")
    }

    fn pixel_at(frame: &RenderedFrame, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * frame.width + x) * 4) as usize;
        [
            frame.pixels_rgba[index],
            frame.pixels_rgba[index + 1],
            frame.pixels_rgba[index + 2],
            frame.pixels_rgba[index + 3],
        ]
    }

    fn workspace() -> Workspace {
        Workspace::empty(id("workspace"))
    }

    fn layer(id_value: &str, x: f32, y: f32, width: f32, height: f32, opacity: f32) -> Layer {
        Layer {
            id: id(id_value),
            name: id_value.to_owned(),
            visible: true,
            opacity,
            locked: false,
            position: Point { x, y },
            bounds: rect(0.0, 0.0, width, height),
            blend_mode: BlendMode::Normal,
            alpha_channel: true,
            transform: Transform::default(),
            clipping: ClippingBehavior::None,
            mask_layer_id: None,
            group_id: None,
            export_participation: ExportParticipation::Included,
        }
    }

    fn viewport(x: f32, y: f32, zoom: f32, width: f32, height: f32) -> Viewport {
        Viewport::new(Point { x, y }, zoom, Size { width, height }).expect("valid viewport")
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }
}
