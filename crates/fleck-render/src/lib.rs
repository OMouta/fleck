//! Skia-backed viewport rendering for Fleck.
//!
//! This crate consumes `fleck-core` document state and viewport geometry. It
//! does not store, mutate, or become the owner of workspace truth.

use fleck_core::geometry::{guide_lines, pixel_grid_for_rect, OverlaySettings, Viewport};
use fleck_core::model::{
    Axis, BlendMode, CanvasBackground, Layer, Point, Rect, RgbaColor, Workspace,
};
use fleck_core::model::{
    ExportArea, ExportBackground, ExportParticipation, MetadataBehavior, ObjectId,
    OutputDefinition, OutputFormat, Padding, Size, TransparencyBehavior, TrimBehavior,
};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder};
use skia_safe::surfaces;
use skia_safe::{
    AlphaType, BlendMode as SkBlendMode, Canvas, Color, ColorType, IPoint, ImageInfo, Paint,
    PaintStyle, Rect as SkRect, Surface,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedExport {
    pub filename: String,
    pub destination: Option<String>,
    pub format: OutputFormat,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultExportOptions {
    pub filename: String,
    pub folder: Option<String>,
    pub format: OutputFormat,
    pub scale: f32,
    pub quality: Option<u8>,
    pub background: ExportBackground,
    pub transparency: TransparencyBehavior,
    pub metadata: MetadataBehavior,
}

impl Default for DefaultExportOptions {
    fn default() -> Self {
        Self {
            filename: "export.png".to_owned(),
            folder: None,
            format: OutputFormat::Png,
            scale: 1.0,
            quality: None,
            background: ExportBackground::Transparent,
            transparency: TransparencyBehavior::Preserve,
            metadata: MetadataBehavior::Strip,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportPipelineError {
    #[error("export area `{id}` was not found")]
    AreaNotFound { id: ObjectId },
    #[error("selection `{id}` was not found")]
    SelectionNotFound { id: ObjectId },
    #[error("output `{id}` was not found")]
    OutputNotFound { id: ObjectId },
    #[error("export area `{id}` does not have outputs")]
    AreaHasNoOutputs { id: ObjectId },
    #[error("no visible exportable content is available")]
    NoExportableContent,
    #[error("output format `{format:?}` is not supported by the basic export pipeline")]
    UnsupportedFormat { format: OutputFormat },
    #[error("export scale must be positive")]
    InvalidScale,
    #[error("render failed")]
    Render(#[from] RenderError),
    #[error("image encoding failed")]
    Encoding(#[from] image::ImageError),
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

    pub fn export_workspace_default(
        &self,
        workspace: &Workspace,
        options: &DefaultExportOptions,
    ) -> Result<EncodedExport, ExportPipelineError> {
        let bounds = default_export_bounds(workspace)?;
        let output = OutputDefinition {
            id: ObjectId::new("default-output").expect("generated id is valid"),
            filename: options.filename.clone(),
            folder: options.folder.clone(),
            format: options.format,
            width: None,
            height: None,
            scale: options.scale,
            quality: options.quality,
            compression: fleck_core::model::CompressionSettings::default(),
            background: options.background.clone(),
            transparency: options.transparency,
            metadata: options.metadata,
        };
        self.export_bounds(workspace, bounds, None, &output, TrimBehavior::None)
    }

    pub fn export_selection(
        &self,
        workspace: &Workspace,
        selection_id: &ObjectId,
        output: &OutputDefinition,
    ) -> Result<EncodedExport, ExportPipelineError> {
        let selection = workspace
            .selections
            .iter()
            .find(|selection| selection.id == *selection_id)
            .ok_or_else(|| ExportPipelineError::SelectionNotFound {
                id: selection_id.clone(),
            })?;
        self.export_bounds(
            workspace,
            selection.bounds,
            None,
            output,
            TrimBehavior::None,
        )
    }

    pub fn export_area(
        &self,
        workspace: &Workspace,
        area_id: &ObjectId,
    ) -> Result<Vec<EncodedExport>, ExportPipelineError> {
        let area = workspace
            .export_areas
            .iter()
            .find(|area| area.id == *area_id)
            .ok_or_else(|| ExportPipelineError::AreaNotFound {
                id: area_id.clone(),
            })?;
        if area.output_ids.is_empty() {
            return Err(ExportPipelineError::AreaHasNoOutputs {
                id: area_id.clone(),
            });
        }

        area.output_ids
            .iter()
            .map(|output_id| {
                let output = output_by_id(workspace, output_id)?;
                self.export_area_output(workspace, area, output)
            })
            .collect()
    }

    pub fn export_all(
        &self,
        workspace: &Workspace,
        default_options: &DefaultExportOptions,
    ) -> Result<Vec<EncodedExport>, ExportPipelineError> {
        if workspace.export_areas.is_empty() {
            return Ok(vec![
                self.export_workspace_default(workspace, default_options)?
            ]);
        }

        workspace
            .export_areas
            .iter()
            .flat_map(|area| {
                area.output_ids
                    .iter()
                    .map(move |output_id| (area, output_id))
            })
            .map(|(area, output_id)| {
                let output = output_by_id(workspace, output_id)?;
                self.export_area_output(workspace, area, output)
            })
            .collect()
    }

    fn export_area_output(
        &self,
        workspace: &Workspace,
        area: &ExportArea,
        output: &OutputDefinition,
    ) -> Result<EncodedExport, ExportPipelineError> {
        let bounds = padded_bounds(area.bounds, area.padding);
        let mut filtered = workspace.clone();
        filtered.layers = workspace
            .layers
            .iter()
            .filter(|layer| layer_participates_in_area(layer, area))
            .cloned()
            .collect();
        self.export_bounds(&filtered, bounds, Some(area), output, area.trim)
    }

    fn export_bounds(
        &self,
        workspace: &Workspace,
        bounds: Rect,
        area: Option<&ExportArea>,
        output: &OutputDefinition,
        trim: TrimBehavior,
    ) -> Result<EncodedExport, ExportPipelineError> {
        if output.scale <= 0.0 {
            return Err(ExportPipelineError::InvalidScale);
        }
        require_basic_format(output.format)?;
        let width = output
            .width
            .unwrap_or_else(|| scaled_dimension(bounds.width, output.scale));
        let height = output
            .height
            .unwrap_or_else(|| scaled_dimension(bounds.height, output.scale));
        let zoom_x = width as f32 / bounds.width.max(1.0);
        let zoom_y = height as f32 / bounds.height.max(1.0);
        let zoom = zoom_x.min(zoom_y).max(f32::EPSILON);
        let background = effective_background(area, output);
        let render_background = if trim == TrimBehavior::TransparentPixels {
            ExportBackground::Transparent
        } else {
            background.clone()
        };
        let mut render_workspace = workspace.clone();
        render_workspace.canvas.background =
            canvas_background_for_export(&render_background, output.transparency);
        let frame = self.render(RenderRequest {
            workspace: &render_workspace,
            viewport: Viewport::new(
                Point {
                    x: bounds.x,
                    y: bounds.y,
                },
                zoom,
                Size {
                    width: width as f32,
                    height: height as f32,
                },
            )
            .expect("positive export viewport"),
            overlays: &export_overlay_settings(),
        })?;
        let (pixels_rgba, width, height) = match trim {
            TrimBehavior::None => (frame.pixels_rgba, frame.width, frame.height),
            TrimBehavior::TransparentPixels => {
                trim_transparent_pixels(frame.pixels_rgba, frame.width, frame.height)
            }
        };
        let pixels = prepare_pixels_for_output(
            pixels_rgba,
            width,
            height,
            &background,
            output.transparency,
            output.format,
        );
        let bytes = encode_pixels(&pixels, width, height, output)?;
        Ok(EncodedExport {
            filename: output.filename.clone(),
            destination: output
                .folder
                .as_ref()
                .map(|folder| format!("{folder}/{}", output.filename)),
            format: output.format,
            width,
            height,
            bytes,
        })
    }
}

fn default_export_bounds(workspace: &Workspace) -> Result<Rect, ExportPipelineError> {
    workspace
        .layers
        .iter()
        .filter(|layer| {
            layer.visible
                && layer.opacity > 0.0
                && layer.export_participation != ExportParticipation::Excluded
        })
        .map(layer_workspace_rect)
        .reduce(union_rect)
        .ok_or(ExportPipelineError::NoExportableContent)
}

fn output_by_id<'a>(
    workspace: &'a Workspace,
    output_id: &ObjectId,
) -> Result<&'a OutputDefinition, ExportPipelineError> {
    workspace
        .outputs
        .iter()
        .find(|output| output.id == *output_id)
        .ok_or_else(|| ExportPipelineError::OutputNotFound {
            id: output_id.clone(),
        })
}

fn padded_bounds(bounds: Rect, padding: Padding) -> Rect {
    Rect {
        x: bounds.x - padding.left,
        y: bounds.y - padding.top,
        width: (bounds.width + padding.left + padding.right).max(1.0),
        height: (bounds.height + padding.top + padding.bottom).max(1.0),
    }
}

fn layer_participates_in_area(layer: &Layer, area: &ExportArea) -> bool {
    if !layer.visible
        || layer.opacity <= 0.0
        || layer.export_participation == ExportParticipation::Excluded
    {
        return false;
    }
    if area.excluded_layer_ids.contains(&layer.id) {
        return false;
    }
    area.included_layer_ids.is_empty() || area.included_layer_ids.contains(&layer.id)
}

fn layer_workspace_rect(layer: &Layer) -> Rect {
    Rect {
        x: layer.position.x + layer.bounds.x,
        y: layer.position.y + layer.bounds.y,
        width: layer.bounds.width * layer.transform.scale_x,
        height: layer.bounds.height * layer.transform.scale_y,
    }
}

fn union_rect(a: Rect, b: Rect) -> Rect {
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = (a.x + a.width).max(b.x + b.width);
    let bottom = (a.y + a.height).max(b.y + b.height);
    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn scaled_dimension(value: f32, scale: f32) -> u32 {
    (value * scale).round().max(1.0) as u32
}

fn effective_background(area: Option<&ExportArea>, output: &OutputDefinition) -> ExportBackground {
    match (&area.map(|area| &area.background), &output.background) {
        (Some(ExportBackground::Solid { color }), ExportBackground::Transparent) => {
            ExportBackground::Solid { color: *color }
        }
        (Some(ExportBackground::CheckerboardPreview), ExportBackground::Transparent) => {
            ExportBackground::CheckerboardPreview
        }
        _ => output.background.clone(),
    }
}

fn canvas_background_for_export(
    background: &ExportBackground,
    transparency: TransparencyBehavior,
) -> CanvasBackground {
    match (background, transparency) {
        (ExportBackground::Solid { color }, _) => CanvasBackground::Solid { color: *color },
        (ExportBackground::CheckerboardPreview, _) => CanvasBackground::Transparent,
        (ExportBackground::Transparent, TransparencyBehavior::Flatten) => CanvasBackground::Solid {
            color: RgbaColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        },
        (ExportBackground::Transparent, TransparencyBehavior::Preserve) => {
            CanvasBackground::Transparent
        }
    }
}

fn export_overlay_settings() -> OverlaySettings {
    OverlaySettings {
        checkerboard: false,
        guides: false,
        export_areas: false,
        selections: false,
        transform_handles: false,
        pixel_grid: fleck_core::geometry::PixelGridSettings {
            enabled: false,
            min_zoom: f32::MAX,
        },
    }
}

fn trim_transparent_pixels(pixels: Vec<u8>, width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let mut left = width;
    let mut top = height;
    let mut right = 0;
    let mut bottom = 0;

    for y in 0..height {
        for x in 0..width {
            let alpha = pixels[((y * width + x) * 4 + 3) as usize];
            if alpha > 0 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
    }

    if right <= left || bottom <= top {
        return (vec![0, 0, 0, 0], 1, 1);
    }

    let trimmed_width = right - left;
    let trimmed_height = bottom - top;
    let mut trimmed = Vec::with_capacity((trimmed_width * trimmed_height * 4) as usize);
    for y in top..bottom {
        let start = ((y * width + left) * 4) as usize;
        let end = start + (trimmed_width * 4) as usize;
        trimmed.extend_from_slice(&pixels[start..end]);
    }
    (trimmed, trimmed_width, trimmed_height)
}

fn prepare_pixels_for_output(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    background: &ExportBackground,
    transparency: TransparencyBehavior,
    format: OutputFormat,
) -> Vec<u8> {
    let must_flatten =
        transparency == TransparencyBehavior::Flatten || format == OutputFormat::Jpeg;
    match (background, must_flatten) {
        (ExportBackground::Transparent, false) => pixels,
        _ => flatten_pixels(
            &pixels,
            width,
            height,
            background_color_or_default(background),
            matches!(background, ExportBackground::CheckerboardPreview),
            format == OutputFormat::Jpeg,
        ),
    }
}

fn background_color_or_default(background: &ExportBackground) -> RgbaColor {
    match background {
        ExportBackground::Solid { color } => *color,
        ExportBackground::Transparent | ExportBackground::CheckerboardPreview => RgbaColor {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
    }
}

fn flatten_pixels(
    pixels: &[u8],
    width: u32,
    height: u32,
    background: RgbaColor,
    checkerboard: bool,
    rgb_only: bool,
) -> Vec<u8> {
    let bytes_per_pixel = if rgb_only { 3 } else { 4 };
    let mut flattened = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            let bg = if checkerboard {
                checkerboard_color(x, y)
            } else {
                background
            };
            let alpha = pixels[index + 3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            let r = (pixels[index] as f32 * alpha + bg.r as f32 * inv_alpha).round() as u8;
            let g = (pixels[index + 1] as f32 * alpha + bg.g as f32 * inv_alpha).round() as u8;
            let b = (pixels[index + 2] as f32 * alpha + bg.b as f32 * inv_alpha).round() as u8;
            flattened.extend_from_slice(&[r, g, b]);
            if !rgb_only {
                flattened.push(255);
            }
        }
    }
    flattened
}

fn checkerboard_color(x: u32, y: u32) -> RgbaColor {
    let shade =
        if ((x / CHECKERBOARD_CELL_SIZE as u32) + (y / CHECKERBOARD_CELL_SIZE as u32)) % 2 == 0 {
            222
        } else {
            184
        };
    RgbaColor {
        r: shade,
        g: shade,
        b: shade,
        a: 255,
    }
}

fn require_basic_format(format: OutputFormat) -> Result<(), ExportPipelineError> {
    match format {
        OutputFormat::Png | OutputFormat::Jpeg | OutputFormat::WebP => Ok(()),
        _ => Err(ExportPipelineError::UnsupportedFormat { format }),
    }
}

fn encode_pixels(
    pixels: &[u8],
    width: u32,
    height: u32,
    output: &OutputDefinition,
) -> Result<Vec<u8>, ExportPipelineError> {
    let mut bytes = Vec::new();
    match output.format {
        OutputFormat::Png => {
            let compression = if output.compression.optimize {
                CompressionType::Best
            } else {
                CompressionType::Fast
            };
            PngEncoder::new_with_quality(&mut bytes, compression, FilterType::Adaptive)
                .write_image(pixels, width, height, ExtendedColorType::Rgba8)?;
        }
        OutputFormat::Jpeg => {
            JpegEncoder::new_with_quality(&mut bytes, output.quality.unwrap_or(85).clamp(1, 100))
                .encode(pixels, width, height, ExtendedColorType::Rgb8)?;
        }
        OutputFormat::WebP => {
            WebPEncoder::new_lossless(&mut bytes).write_image(
                pixels,
                width,
                height,
                ExtendedColorType::Rgba8,
            )?;
        }
        _ => {
            return Err(ExportPipelineError::UnsupportedFormat {
                format: output.format,
            })
        }
    }
    Ok(bytes)
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
        paint.set_blend_mode(blend_mode_to_skia(layer.blend_mode));
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

fn blend_mode_to_skia(blend_mode: BlendMode) -> SkBlendMode {
    match blend_mode {
        BlendMode::Normal => SkBlendMode::SrcOver,
        BlendMode::Multiply => SkBlendMode::Multiply,
        BlendMode::Screen => SkBlendMode::Screen,
        BlendMode::Overlay => SkBlendMode::Overlay,
        BlendMode::Darken => SkBlendMode::Darken,
        BlendMode::Lighten => SkBlendMode::Lighten,
        BlendMode::ColorDodge => SkBlendMode::ColorDodge,
        BlendMode::ColorBurn => SkBlendMode::ColorBurn,
        BlendMode::HardLight => SkBlendMode::HardLight,
        BlendMode::SoftLight => SkBlendMode::SoftLight,
        BlendMode::Difference => SkBlendMode::Difference,
        BlendMode::Exclusion => SkBlendMode::Exclusion,
        BlendMode::Hue => SkBlendMode::Hue,
        BlendMode::Saturation => SkBlendMode::Saturation,
        BlendMode::Color => SkBlendMode::Color,
        BlendMode::Luminosity => SkBlendMode::Luminosity,
    }
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
    fn blend_modes_change_deterministic_output() {
        let mut normal = workspace();
        normal.layers.push(layer("base", 0.0, 0.0, 16.0, 16.0, 1.0));
        normal.layers.push(layer("top", 0.0, 0.0, 16.0, 16.0, 1.0));

        let mut multiply = normal.clone();
        multiply.layers[1].blend_mode = BlendMode::Multiply;

        let viewport = viewport(0.0, 0.0, 1.0, 24.0, 24.0);
        let normal_frame = render_without_overlays(&normal, viewport);
        let multiply_frame = render_without_overlays(&multiply, viewport);

        assert_ne!(
            pixel_at(&normal_frame, 8, 8),
            pixel_at(&multiply_frame, 8, 8)
        );
    }

    #[test]
    fn hidden_layers_do_not_render() {
        let mut workspace = workspace();
        let mut hidden = layer("hidden", 0.0, 0.0, 16.0, 16.0, 1.0);
        hidden.visible = false;
        workspace.layers.push(hidden);

        let frame = render_without_overlays(&workspace, viewport(0.0, 0.0, 1.0, 24.0, 24.0));

        assert_eq!(pixel_at(&frame, 8, 8), [0, 0, 0, 0]);
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
            mask: None,
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

    #[test]
    fn default_export_uses_visible_layer_bounds_without_export_areas() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 10.0, 20.0, 16.0, 8.0, 1.0));

        let export = SkiaViewportRenderer::new()
            .export_workspace_default(
                &workspace,
                &DefaultExportOptions {
                    filename: "default.png".to_owned(),
                    ..DefaultExportOptions::default()
                },
            )
            .expect("default export");

        assert_eq!(export.filename, "default.png");
        assert_eq!(export.format, OutputFormat::Png);
        assert_eq!((export.width, export.height), (16, 8));
        assert!(export.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn export_area_outputs_apply_padding_scale_and_layer_rules() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 16.0, 16.0, 1.0));
        workspace
            .layers
            .push(layer("excluded", 0.0, 0.0, 16.0, 16.0, 1.0));
        workspace
            .outputs
            .push(output("png", OutputFormat::Png, 2.0));
        workspace.export_areas.push(ExportArea {
            id: id("area"),
            name: "Area".to_owned(),
            bounds: rect(0.0, 0.0, 10.0, 10.0),
            padding: Padding {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0,
            },
            background: ExportBackground::Transparent,
            trim: TrimBehavior::None,
            output_ids: vec![id("png")],
            included_layer_ids: vec![id("base")],
            excluded_layer_ids: vec![id("excluded")],
            tags: Vec::new(),
            preset_id: None,
        });

        let exports = SkiaViewportRenderer::new()
            .export_area(&workspace, &id("area"))
            .expect("area export");

        assert_eq!(exports.len(), 1);
        assert_eq!((exports[0].width, exports[0].height), (32, 28));
        assert!(exports[0].bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn export_encodes_jpeg_and_webp_with_background_handling() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 8.0, 8.0, 0.5));
        let mut jpeg = output("jpeg", OutputFormat::Jpeg, 1.0);
        jpeg.filename = "image.jpg".to_owned();
        jpeg.background = ExportBackground::Solid {
            color: RgbaColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        };
        jpeg.transparency = TransparencyBehavior::Flatten;
        let webp = output("webp", OutputFormat::WebP, 1.0);

        let renderer = SkiaViewportRenderer::new();
        let jpg_export = renderer
            .export_selection(
                &workspace_with_selection(workspace.clone()),
                &id("selection"),
                &jpeg,
            )
            .expect("jpeg");
        let webp_export = renderer
            .export_selection(
                &workspace_with_selection(workspace),
                &id("selection"),
                &webp,
            )
            .expect("webp");

        assert!(jpg_export.bytes.starts_with(&[0xff, 0xd8]));
        assert!(webp_export.bytes.starts_with(b"RIFF"));
        assert_eq!((jpg_export.width, jpg_export.height), (8, 8));
    }

    #[test]
    fn transparent_and_solid_png_exports_have_expected_alpha() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 0.0, 0.0, 4.0, 4.0, 1.0));
        let workspace = workspace_with_selection(workspace);
        let transparent = output("transparent", OutputFormat::Png, 1.0);
        let mut solid = output("solid", OutputFormat::Png, 1.0);
        solid.background = ExportBackground::Solid {
            color: RgbaColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        };
        solid.transparency = TransparencyBehavior::Flatten;

        let renderer = SkiaViewportRenderer::new();
        let transparent_export = renderer
            .export_selection(&workspace, &id("selection"), &transparent)
            .expect("transparent export");
        let solid_export = renderer
            .export_selection(&workspace, &id("selection"), &solid)
            .expect("solid export");

        assert_eq!(decoded_pixel(&transparent_export.bytes, 7, 7)[3], 0);
        assert_eq!(decoded_pixel(&solid_export.bytes, 7, 7)[3], 255);
    }

    #[test]
    fn transparent_trim_crops_empty_padding_before_flattening() {
        let mut workspace = workspace();
        workspace
            .layers
            .push(layer("base", 4.0, 4.0, 4.0, 4.0, 1.0));
        workspace
            .outputs
            .push(output("png", OutputFormat::Png, 1.0));
        workspace.export_areas.push(ExportArea {
            id: id("area"),
            name: "Area".to_owned(),
            bounds: rect(0.0, 0.0, 16.0, 16.0),
            padding: Padding::default(),
            background: ExportBackground::Transparent,
            trim: TrimBehavior::TransparentPixels,
            output_ids: vec![id("png")],
            included_layer_ids: Vec::new(),
            excluded_layer_ids: Vec::new(),
            tags: Vec::new(),
            preset_id: None,
        });

        let export = SkiaViewportRenderer::new()
            .export_area(&workspace, &id("area"))
            .expect("area export")
            .remove(0);

        assert_eq!((export.width, export.height), (4, 4));
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

    fn decoded_pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
        let image = image::load_from_memory(bytes)
            .expect("decode export")
            .to_rgba8();
        let pixel = image.get_pixel(x, y).0;
        [pixel[0], pixel[1], pixel[2], pixel[3]]
    }

    fn workspace() -> Workspace {
        Workspace::empty(id("workspace"))
    }

    fn workspace_with_selection(mut workspace: Workspace) -> Workspace {
        workspace.selections.push(Selection {
            id: id("selection"),
            kind: SelectionKind::Rectangular,
            bounds: rect(0.0, 0.0, 8.0, 8.0),
            feather_radius: 0.0,
            source_layer_ids: Vec::new(),
            mask: None,
        });
        workspace
    }

    fn output(id_value: &str, format: OutputFormat, scale: f32) -> OutputDefinition {
        OutputDefinition {
            id: id(id_value),
            filename: format!("{id_value}.png"),
            folder: None,
            format,
            width: None,
            height: None,
            scale,
            quality: Some(85),
            compression: fleck_core::model::CompressionSettings::default(),
            background: ExportBackground::Transparent,
            transparency: TransparencyBehavior::Preserve,
            metadata: MetadataBehavior::Strip,
        }
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
