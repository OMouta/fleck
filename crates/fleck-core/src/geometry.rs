use crate::model::{Axis, ExportArea, Guide, Layer, Point, Rect, Size, Workspace};
use serde::{Deserialize, Serialize};

pub const DEFAULT_MIN_PIXEL_GRID_ZOOM: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub origin: Point,
    pub zoom: f32,
    pub screen_size: Size,
}

impl Viewport {
    pub fn new(origin: Point, zoom: f32, screen_size: Size) -> Result<Self, GeometryError> {
        if zoom <= 0.0 {
            return Err(GeometryError::InvalidZoom { zoom });
        }
        if screen_size.width <= 0.0 || screen_size.height <= 0.0 {
            return Err(GeometryError::InvalidScreenSize { screen_size });
        }
        Ok(Self {
            origin,
            zoom,
            screen_size,
        })
    }

    pub fn workspace_to_screen(&self, point: Point) -> Point {
        Point {
            x: (point.x - self.origin.x) * self.zoom,
            y: (point.y - self.origin.y) * self.zoom,
        }
    }

    pub fn screen_to_workspace(&self, point: Point) -> Point {
        Point {
            x: point.x / self.zoom + self.origin.x,
            y: point.y / self.zoom + self.origin.y,
        }
    }

    pub fn pan_by_screen_delta(&mut self, delta: Point) {
        self.origin.x -= delta.x / self.zoom;
        self.origin.y -= delta.y / self.zoom;
    }

    pub fn zoom_around_screen_point(
        &mut self,
        screen_anchor: Point,
        new_zoom: f32,
    ) -> Result<(), GeometryError> {
        if new_zoom <= 0.0 {
            return Err(GeometryError::InvalidZoom { zoom: new_zoom });
        }

        let workspace_anchor = self.screen_to_workspace(screen_anchor);
        self.zoom = new_zoom;
        self.origin = Point {
            x: workspace_anchor.x - screen_anchor.x / self.zoom,
            y: workspace_anchor.y - screen_anchor.y / self.zoom,
        };
        Ok(())
    }

    pub fn visible_workspace_rect(&self) -> Rect {
        Rect {
            x: self.origin.x,
            y: self.origin.y,
            width: self.screen_size.width / self.zoom,
            height: self.screen_size.height / self.zoom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayerTransform {
    pub position: Point,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_degrees: f32,
}

impl LayerTransform {
    pub fn from_layer(layer: &Layer) -> Self {
        Self {
            position: layer.position,
            scale_x: layer.transform.scale_x,
            scale_y: layer.transform.scale_y,
            rotation_degrees: layer.transform.rotation_degrees,
        }
    }

    pub fn local_to_workspace(&self, local: Point) -> Point {
        let scaled = Point {
            x: local.x * self.scale_x,
            y: local.y * self.scale_y,
        };
        let angle = self.rotation_degrees.to_radians();
        let (sin, cos) = angle.sin_cos();
        Point {
            x: self.position.x + scaled.x * cos - scaled.y * sin,
            y: self.position.y + scaled.x * sin + scaled.y * cos,
        }
    }

    pub fn workspace_to_local(&self, workspace: Point) -> Result<Point, GeometryError> {
        if self.scale_x == 0.0 || self.scale_y == 0.0 {
            return Err(GeometryError::NonInvertibleTransform);
        }

        let translated = Point {
            x: workspace.x - self.position.x,
            y: workspace.y - self.position.y,
        };
        let angle = (-self.rotation_degrees).to_radians();
        let (sin, cos) = angle.sin_cos();
        Ok(Point {
            x: (translated.x * cos - translated.y * sin) / self.scale_x,
            y: (translated.x * sin + translated.y * cos) / self.scale_y,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub pixels: bool,
    pub layer_bounds: bool,
    pub export_area_bounds: bool,
    pub centers: bool,
    pub edges: bool,
    pub guides: bool,
    pub common_sizes: Vec<Size>,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.5,
            pixels: true,
            layer_bounds: true,
            export_area_bounds: true,
            centers: true,
            edges: true,
            guides: true,
            common_sizes: vec![
                Size {
                    width: 16.0,
                    height: 16.0,
                },
                Size {
                    width: 32.0,
                    height: 32.0,
                },
                Size {
                    width: 64.0,
                    height: 64.0,
                },
                Size {
                    width: 128.0,
                    height: 128.0,
                },
                Size {
                    width: 256.0,
                    height: 256.0,
                },
                Size {
                    width: 512.0,
                    height: 512.0,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapResult {
    pub point: Point,
    pub hits: Vec<SnapHit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapSizeResult {
    pub size: Size,
    pub hits: Vec<SnapHit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapHit {
    pub axis: Axis,
    pub target: SnapTarget,
    pub value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapTarget {
    Pixel,
    LayerBounds,
    ExportAreaBounds,
    Center,
    Edge,
    Guide,
    CommonSize,
}

pub fn snap_workspace_point(
    workspace: &Workspace,
    point: Point,
    settings: &SnapSettings,
) -> SnapResult {
    if !settings.enabled {
        return SnapResult {
            point,
            hits: Vec::new(),
        };
    }

    let x_candidates = snap_candidates_for_axis(workspace, Axis::Vertical, settings);
    let y_candidates = snap_candidates_for_axis(workspace, Axis::Horizontal, settings);
    let (x, mut hits) = nearest_snap(point.x, &x_candidates, settings.threshold);
    let (y, y_hits) = nearest_snap(point.y, &y_candidates, settings.threshold);
    hits.extend(y_hits);

    SnapResult {
        point: Point { x, y },
        hits,
    }
}

pub fn snap_size(size: Size, settings: &SnapSettings) -> SnapSizeResult {
    if !settings.enabled {
        return SnapSizeResult {
            size,
            hits: Vec::new(),
        };
    }

    let mut hits = Vec::new();
    let width = nearest_size_dimension(
        size.width,
        Axis::Vertical,
        &settings.common_sizes,
        settings.threshold,
        &mut hits,
    );
    let height = nearest_size_dimension(
        size.height,
        Axis::Horizontal,
        &settings.common_sizes,
        settings.threshold,
        &mut hits,
    );
    SnapSizeResult {
        size: Size { width, height },
        hits,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PixelGridSettings {
    pub enabled: bool,
    pub min_zoom: f32,
}

impl Default for PixelGridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            min_zoom: DEFAULT_MIN_PIXEL_GRID_ZOOM,
        }
    }
}

impl PixelGridSettings {
    pub fn visible_at_zoom(&self, zoom: f32) -> bool {
        self.enabled && zoom >= self.min_zoom
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelGrid {
    pub vertical_lines: Vec<f32>,
    pub horizontal_lines: Vec<f32>,
}

pub fn pixel_grid_for_rect(
    visible_workspace: Rect,
    zoom: f32,
    settings: &PixelGridSettings,
) -> PixelGrid {
    if !settings.visible_at_zoom(zoom) {
        return PixelGrid {
            vertical_lines: Vec::new(),
            horizontal_lines: Vec::new(),
        };
    }

    PixelGrid {
        vertical_lines: integer_lines(visible_workspace.x, visible_workspace.right()),
        horizontal_lines: integer_lines(visible_workspace.y, visible_workspace.bottom()),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlaySettings {
    pub checkerboard: bool,
    pub guides: bool,
    pub pixel_grid: PixelGridSettings,
    pub selections: bool,
    pub transform_handles: bool,
    pub export_areas: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            checkerboard: true,
            guides: true,
            pixel_grid: PixelGridSettings::default(),
            selections: true,
            transform_handles: true,
            export_areas: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuideLine {
    pub axis: Axis,
    pub position: f32,
}

pub fn guide_lines(workspace: &Workspace) -> Vec<GuideLine> {
    workspace.guides.iter().map(GuideLine::from).collect()
}

impl From<&Guide> for GuideLine {
    fn from(guide: &Guide) -> Self {
        Self {
            axis: guide.axis,
            position: guide.position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GeometryError {
    #[error("zoom must be positive, got {zoom}")]
    InvalidZoom { zoom: f32 },
    #[error("screen size must be positive, got {screen_size:?}")]
    InvalidScreenSize { screen_size: Size },
    #[error("transform cannot be inverted")]
    NonInvertibleTransform,
}

trait RectExt {
    fn right(&self) -> f32;
    fn bottom(&self) -> f32;
    fn center_x(&self) -> f32;
    fn center_y(&self) -> f32;
}

impl RectExt for Rect {
    fn right(&self) -> f32 {
        self.x + self.width
    }

    fn bottom(&self) -> f32 {
        self.y + self.height
    }

    fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }
}

fn snap_candidates_for_axis(
    workspace: &Workspace,
    axis: Axis,
    settings: &SnapSettings,
) -> Vec<SnapHit> {
    let mut candidates = Vec::new();

    if settings.pixels {
        candidates.push(SnapHit {
            axis,
            target: SnapTarget::Pixel,
            value: 0.0,
        });
    }

    if settings.layer_bounds || settings.centers || settings.edges {
        for layer in &workspace.layers {
            candidates.extend(rect_snap_candidates(layer.bounds, axis, settings, true));
        }
    }

    if settings.export_area_bounds || settings.centers || settings.edges {
        for export_area in &workspace.export_areas {
            candidates.extend(export_area_snap_candidates(export_area, axis, settings));
        }
    }

    if settings.guides {
        candidates.extend(workspace.guides.iter().filter_map(|guide| {
            guide_matches_axis(guide, axis).then_some(SnapHit {
                axis,
                target: SnapTarget::Guide,
                value: guide.position,
            })
        }));
    }

    candidates
}

fn rect_snap_candidates(
    rect: Rect,
    axis: Axis,
    settings: &SnapSettings,
    is_layer: bool,
) -> Vec<SnapHit> {
    let mut candidates = Vec::new();
    let bounds_target = if is_layer {
        SnapTarget::LayerBounds
    } else {
        SnapTarget::ExportAreaBounds
    };
    let values = match axis {
        Axis::Vertical => (rect.x, rect.right(), rect.center_x()),
        Axis::Horizontal => (rect.y, rect.bottom(), rect.center_y()),
    };

    if (is_layer && settings.layer_bounds) || (!is_layer && settings.export_area_bounds) {
        candidates.push(SnapHit {
            axis,
            target: bounds_target,
            value: values.0,
        });
        candidates.push(SnapHit {
            axis,
            target: bounds_target,
            value: values.1,
        });
    }

    if settings.edges {
        candidates.push(SnapHit {
            axis,
            target: SnapTarget::Edge,
            value: values.0,
        });
        candidates.push(SnapHit {
            axis,
            target: SnapTarget::Edge,
            value: values.1,
        });
    }

    if settings.centers {
        candidates.push(SnapHit {
            axis,
            target: SnapTarget::Center,
            value: values.2,
        });
    }

    candidates
}

fn export_area_snap_candidates(
    export_area: &ExportArea,
    axis: Axis,
    settings: &SnapSettings,
) -> Vec<SnapHit> {
    rect_snap_candidates(export_area.bounds, axis, settings, false)
}

fn guide_matches_axis(guide: &Guide, point_axis: Axis) -> bool {
    matches!(
        (guide.axis, point_axis),
        (Axis::Vertical, Axis::Vertical) | (Axis::Horizontal, Axis::Horizontal)
    )
}

fn nearest_snap(value: f32, candidates: &[SnapHit], threshold: f32) -> (f32, Vec<SnapHit>) {
    let mut best = candidates
        .iter()
        .filter_map(|candidate| {
            let candidate_value = if candidate.target == SnapTarget::Pixel {
                value.round()
            } else {
                candidate.value
            };
            let distance = (candidate_value - value).abs();
            (distance <= threshold).then_some((distance, candidate, candidate_value))
        })
        .min_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| target_priority(left.1.target).cmp(&target_priority(right.1.target)))
        });

    if let Some((_, _, snapped)) = best.take() {
        let mut hits = candidates
            .iter()
            .filter_map(|candidate| {
                let candidate_value = if candidate.target == SnapTarget::Pixel {
                    value.round()
                } else {
                    candidate.value
                };
                ((candidate_value - snapped).abs() < f32::EPSILON).then(|| {
                    let mut hit = candidate.clone();
                    hit.value = snapped;
                    hit
                })
            })
            .collect::<Vec<_>>();
        hits.sort_by_key(|hit| target_priority(hit.target));
        hits.dedup_by(|left, right| {
            left.axis == right.axis
                && left.target == right.target
                && (left.value - right.value).abs() < f32::EPSILON
        });
        (snapped, hits)
    } else {
        (value, Vec::new())
    }
}

fn target_priority(target: SnapTarget) -> u8 {
    match target {
        SnapTarget::Guide => 0,
        SnapTarget::LayerBounds | SnapTarget::ExportAreaBounds => 1,
        SnapTarget::Center | SnapTarget::Edge => 2,
        SnapTarget::CommonSize => 3,
        SnapTarget::Pixel => 4,
    }
}

fn nearest_size_dimension(
    value: f32,
    axis: Axis,
    common_sizes: &[Size],
    threshold: f32,
    hits: &mut Vec<SnapHit>,
) -> f32 {
    let candidate = common_sizes
        .iter()
        .map(|size| match axis {
            Axis::Vertical => size.width,
            Axis::Horizontal => size.height,
        })
        .filter_map(|candidate| {
            let distance = (candidate - value).abs();
            (distance <= threshold).then_some((distance, candidate))
        })
        .min_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    if let Some((_, snapped)) = candidate {
        hits.push(SnapHit {
            axis,
            target: SnapTarget::CommonSize,
            value: snapped,
        });
        snapped
    } else {
        value
    }
}

fn integer_lines(start: f32, end: f32) -> Vec<f32> {
    let first = start.floor() as i32;
    let last = end.ceil() as i32;
    (first..=last).map(|line| line as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BlendMode, ClippingBehavior, ExportBackground, ExportParticipation, Guide, ObjectId,
        Padding, Transform, TrimBehavior, Workspace,
    };

    #[test]
    fn viewport_converts_between_workspace_and_screen() {
        let viewport = Viewport::new(
            Point { x: 10.0, y: 20.0 },
            2.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
        )
        .expect("valid viewport");

        let screen = viewport.workspace_to_screen(Point { x: 15.0, y: 30.0 });

        assert_eq!(screen, Point { x: 10.0, y: 20.0 });
        assert_eq!(
            viewport.screen_to_workspace(screen),
            Point { x: 15.0, y: 30.0 }
        );
    }

    #[test]
    fn zoom_around_anchor_keeps_anchor_stable() {
        let mut viewport = Viewport::new(
            Point { x: 0.0, y: 0.0 },
            1.0,
            Size {
                width: 100.0,
                height: 100.0,
            },
        )
        .expect("valid viewport");
        let anchor = Point { x: 25.0, y: 30.0 };
        let before = viewport.screen_to_workspace(anchor);

        viewport
            .zoom_around_screen_point(anchor, 4.0)
            .expect("zoom");

        assert_eq!(viewport.screen_to_workspace(anchor), before);
    }

    #[test]
    fn layer_transform_round_trips_points() {
        let transform = LayerTransform {
            position: Point { x: 10.0, y: 20.0 },
            scale_x: 2.0,
            scale_y: 3.0,
            rotation_degrees: 90.0,
        };
        let local = Point { x: 4.0, y: 5.0 };

        let workspace = transform.local_to_workspace(local);
        let decoded = transform
            .workspace_to_local(workspace)
            .expect("invert transform");

        assert_close(decoded.x, local.x);
        assert_close(decoded.y, local.y);
    }

    #[test]
    fn snap_point_targets_pixels_layers_export_areas_centers_edges_and_guides() {
        let workspace = snapping_workspace();
        let settings = SnapSettings {
            threshold: 0.25,
            ..SnapSettings::default()
        };

        let pixel = snap_workspace_point(&workspace, Point { x: 2.9, y: 4.1 }, &settings);
        assert_eq!(pixel.point, Point { x: 3.0, y: 4.0 });
        assert!(pixel.hits.iter().any(|hit| hit.target == SnapTarget::Pixel));

        let layer = snap_workspace_point(&workspace, Point { x: 9.9, y: 20.1 }, &settings);
        assert_eq!(layer.point, Point { x: 10.0, y: 20.0 });
        assert!(layer
            .hits
            .iter()
            .any(|hit| hit.target == SnapTarget::LayerBounds));

        let export = snap_workspace_point(&workspace, Point { x: 200.1, y: 70.0 }, &settings);
        assert_eq!(export.point.x, 200.0);
        assert!(export
            .hits
            .iter()
            .any(|hit| hit.target == SnapTarget::ExportAreaBounds));

        let center = snap_workspace_point(&workspace, Point { x: 60.0, y: 45.0 }, &settings);
        assert!(center
            .hits
            .iter()
            .any(|hit| hit.target == SnapTarget::Center));

        let guide = snap_workspace_point(&workspace, Point { x: 99.9, y: 99.9 }, &settings);
        assert_eq!(guide.point, Point { x: 100.0, y: 100.0 });
        assert!(guide.hits.iter().any(|hit| hit.target == SnapTarget::Guide));

        let edge = snap_workspace_point(&workspace, Point { x: 110.0, y: 69.9 }, &settings);
        assert!(edge.hits.iter().any(|hit| hit.target == SnapTarget::Edge));
    }

    #[test]
    fn snap_size_targets_common_sizes() {
        let settings = SnapSettings {
            threshold: 2.0,
            ..SnapSettings::default()
        };

        let result = snap_size(
            Size {
                width: 127.0,
                height: 513.0,
            },
            &settings,
        );

        assert_eq!(
            result.size,
            Size {
                width: 128.0,
                height: 512.0
            }
        );
        assert_eq!(result.hits.len(), 2);
    }

    #[test]
    fn pixel_grid_only_appears_at_useful_zoom_and_uses_integer_boundaries() {
        let rect = Rect {
            x: 0.25,
            y: 1.25,
            width: 2.5,
            height: 2.5,
        };
        let settings = PixelGridSettings::default();

        let hidden = pixel_grid_for_rect(rect, 2.0, &settings);
        assert!(hidden.vertical_lines.is_empty());

        let visible = pixel_grid_for_rect(rect, 8.0, &settings);
        assert_eq!(visible.vertical_lines, vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(visible.horizontal_lines, vec![1.0, 2.0, 3.0, 4.0]);
    }

    fn snapping_workspace() -> Workspace {
        let mut workspace = Workspace::empty(id("workspace"));
        workspace.layers.push(Layer {
            id: id("layer"),
            name: "Layer".to_owned(),
            visible: true,
            opacity: 1.0,
            locked: false,
            position: Point::ZERO,
            bounds: Rect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
            },
            blend_mode: BlendMode::Normal,
            alpha_channel: true,
            transform: Transform::default(),
            clipping: ClippingBehavior::None,
            mask_layer_id: None,
            group_id: None,
            export_participation: ExportParticipation::Included,
        });
        workspace.export_areas.push(crate::model::ExportArea {
            id: id("export"),
            name: "Export".to_owned(),
            bounds: Rect {
                x: 200.0,
                y: 30.0,
                width: 64.0,
                height: 64.0,
            },
            padding: Padding::default(),
            background: ExportBackground::Transparent,
            trim: TrimBehavior::None,
            output_ids: Vec::new(),
            included_layer_ids: Vec::new(),
            excluded_layer_ids: Vec::new(),
            tags: Vec::new(),
            preset_id: None,
        });
        workspace.guides.push(Guide {
            id: id("guide-x"),
            axis: Axis::Vertical,
            position: 100.0,
            locked: false,
        });
        workspace.guides.push(Guide {
            id: id("guide-y"),
            axis: Axis::Horizontal,
            position: 100.0,
            locked: false,
        });
        workspace
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
