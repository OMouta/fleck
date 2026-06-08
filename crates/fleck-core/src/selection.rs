use crate::export::{self, NewExportArea};
use crate::layer::{self, NewLayer};
use crate::model::{
    ExportBackground, ObjectId, Padding, Point, Rect, RgbaColor, Selection, SelectionKind,
    SelectionMask, TrimBehavior, Workspace,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NewSelection {
    pub id: ObjectId,
    pub kind: SelectionKind,
    pub bounds: Rect,
    pub source_layer_ids: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionCopy {
    pub bounds: Rect,
    pub source_layer_ids: Vec<ObjectId>,
    pub mask: Option<SelectionMask>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SelectionError {
    #[error("selection `{id}` was not found")]
    NotFound { id: ObjectId },
    #[error("selection `{id}` already exists")]
    DuplicateId { id: ObjectId },
    #[error("layer `{id}` was not found")]
    LayerNotFound { id: ObjectId },
    #[error("selection bounds must be positive")]
    NonPositiveBounds,
    #[error("selection mask is empty")]
    EmptyMask,
    #[error("selection point list must contain at least {minimum} point(s)")]
    NotEnoughPoints { minimum: usize },
}

pub type SelectionResult<T> = Result<T, SelectionError>;

pub fn create_selection(
    workspace: &mut Workspace,
    new_selection: NewSelection,
) -> SelectionResult<()> {
    ensure_unique_selection_id(workspace, &new_selection.id)?;
    validate_bounds(new_selection.bounds)?;
    require_layers(workspace, &new_selection.source_layer_ids)?;

    let mask = mask_for_kind(new_selection.bounds, &new_selection.kind)?;
    workspace.selections.push(Selection {
        id: new_selection.id,
        kind: new_selection.kind,
        bounds: new_selection.bounds,
        feather_radius: 0.0,
        source_layer_ids: new_selection.source_layer_ids,
        mask: Some(mask),
    });
    Ok(())
}

pub fn expand_selection(
    workspace: &mut Workspace,
    id: &ObjectId,
    amount: f32,
) -> SelectionResult<()> {
    resize_selection(workspace, id, amount.abs())
}

pub fn contract_selection(
    workspace: &mut Workspace,
    id: &ObjectId,
    amount: f32,
) -> SelectionResult<()> {
    resize_selection(workspace, id, -amount.abs())
}

pub fn feather_selection(
    workspace: &mut Workspace,
    id: &ObjectId,
    radius: f32,
) -> SelectionResult<()> {
    let selection = require_selection_mut(workspace, id)?;
    selection.feather_radius = radius.max(0.0);
    if let Some(mask) = &mut selection.mask {
        apply_feather(mask, selection.feather_radius);
    }
    Ok(())
}

pub fn invert_selection(workspace: &mut Workspace, id: &ObjectId) -> SelectionResult<()> {
    let selection = require_selection_mut(workspace, id)?;
    if let Some(mask) = &mut selection.mask {
        for alpha in &mut mask.alpha {
            *alpha = 255 - *alpha;
        }
        if mask.alpha.iter().all(|alpha| *alpha == 0) {
            return Err(SelectionError::EmptyMask);
        }
    }
    Ok(())
}

pub fn move_selection(
    workspace: &mut Workspace,
    id: &ObjectId,
    dx: f32,
    dy: f32,
) -> SelectionResult<()> {
    let selection = require_selection_mut(workspace, id)?;
    selection.bounds.x += dx;
    selection.bounds.y += dy;
    Ok(())
}

pub fn delete_selection(workspace: &mut Workspace, id: &ObjectId) -> SelectionResult<Selection> {
    let index = require_selection_index(workspace, id)?;
    Ok(workspace.selections.remove(index))
}

pub fn copy_selection(workspace: &Workspace, id: &ObjectId) -> SelectionResult<SelectionCopy> {
    let selection = require_selection(workspace, id)?;
    Ok(SelectionCopy {
        bounds: selection.bounds,
        source_layer_ids: selection.source_layer_ids.clone(),
        mask: selection.mask.clone(),
    })
}

pub fn layer_from_selection(
    workspace: &mut Workspace,
    selection_id: &ObjectId,
    layer_id: ObjectId,
    name: String,
) -> SelectionResult<()> {
    let selection = require_selection(workspace, selection_id)?.clone();
    layer::create_layer(
        workspace,
        NewLayer {
            id: layer_id,
            name,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: selection.bounds.width,
                height: selection.bounds.height,
            },
            position: Point {
                x: selection.bounds.x,
                y: selection.bounds.y,
            },
        },
    )
    .map_err(|error| match error {
        layer::LayerError::NotFound { id } => SelectionError::LayerNotFound { id },
        layer::LayerError::DuplicateId { id } => SelectionError::DuplicateId { id },
        layer::LayerError::NonPositiveBounds => SelectionError::NonPositiveBounds,
        _ => SelectionError::EmptyMask,
    })?;
    Ok(())
}

pub fn export_area_from_selection(
    workspace: &mut Workspace,
    selection_id: &ObjectId,
    area_id: ObjectId,
    name: String,
) -> SelectionResult<()> {
    let selection = require_selection(workspace, selection_id)?.clone();
    export::create_export_area(
        workspace,
        NewExportArea {
            id: area_id,
            name,
            bounds: selection.bounds,
            padding: Padding::default(),
            background: ExportBackground::Transparent,
            trim: TrimBehavior::None,
            output_ids: Vec::new(),
            included_layer_ids: selection.source_layer_ids,
            excluded_layer_ids: Vec::new(),
            tags: Vec::new(),
            preset_id: None,
        },
    )
    .map_err(|error| match error {
        export::ExportError::LayerNotFound { id } => SelectionError::LayerNotFound { id },
        export::ExportError::DuplicateAreaId { id } => SelectionError::DuplicateId { id },
        export::ExportError::NonPositiveAreaBounds => SelectionError::NonPositiveBounds,
        _ => SelectionError::EmptyMask,
    })?;
    Ok(())
}

fn resize_selection(workspace: &mut Workspace, id: &ObjectId, delta: f32) -> SelectionResult<()> {
    let selection = require_selection_mut(workspace, id)?;
    let bounds = Rect {
        x: selection.bounds.x - delta,
        y: selection.bounds.y - delta,
        width: selection.bounds.width + delta * 2.0,
        height: selection.bounds.height + delta * 2.0,
    };
    validate_bounds(bounds)?;
    selection.bounds = bounds;
    selection.mask = Some(mask_for_kind(bounds, &selection.kind)?);
    if selection.feather_radius > 0.0 {
        if let Some(mask) = &mut selection.mask {
            apply_feather(mask, selection.feather_radius);
        }
    }
    Ok(())
}

fn mask_for_kind(bounds: Rect, kind: &SelectionKind) -> SelectionResult<SelectionMask> {
    validate_bounds(bounds)?;
    let width = bounds.width.ceil().max(1.0) as u32;
    let height = bounds.height.ceil().max(1.0) as u32;
    let mut alpha = vec![0; width as usize * height as usize];

    match kind {
        SelectionKind::Rectangular
        | SelectionKind::MagicWand { .. }
        | SelectionKind::ColorRange { .. } => alpha.fill(255),
        SelectionKind::Elliptical => fill_ellipse(&mut alpha, width, height),
        SelectionKind::Lasso | SelectionKind::Polygon { .. } => {
            alpha.fill(255);
        }
    }

    if alpha.iter().all(|alpha| *alpha == 0) {
        return Err(SelectionError::EmptyMask);
    }

    Ok(SelectionMask {
        width,
        height,
        alpha,
    })
}

fn fill_ellipse(alpha: &mut [u8], width: u32, height: u32) {
    let rx = width as f32 / 2.0;
    let ry = height as f32 / 2.0;
    let cx = rx - 0.5;
    let cy = ry - 0.5;
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - cx) / rx.max(1.0);
            let dy = (y as f32 - cy) / ry.max(1.0);
            if dx * dx + dy * dy <= 1.0 {
                alpha[(y * width + x) as usize] = 255;
            }
        }
    }
}

fn apply_feather(mask: &mut SelectionMask, radius: f32) {
    if radius <= 0.0 {
        return;
    }
    let fade = radius.ceil().max(1.0) as u32;
    for y in 0..mask.height {
        for x in 0..mask.width {
            let edge_distance = x
                .min(y)
                .min(mask.width.saturating_sub(1).saturating_sub(x))
                .min(mask.height.saturating_sub(1).saturating_sub(y));
            if edge_distance < fade {
                let index = (y * mask.width + x) as usize;
                let factor = edge_distance as f32 / fade as f32;
                mask.alpha[index] = ((mask.alpha[index] as f32) * factor).round() as u8;
            }
        }
    }
}

fn validate_bounds(bounds: Rect) -> SelectionResult<()> {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        Err(SelectionError::NonPositiveBounds)
    } else {
        Ok(())
    }
}

fn require_selection<'a>(
    workspace: &'a Workspace,
    id: &ObjectId,
) -> SelectionResult<&'a Selection> {
    workspace
        .selections
        .iter()
        .find(|selection| selection.id == *id)
        .ok_or_else(|| SelectionError::NotFound { id: id.clone() })
}

fn require_selection_mut<'a>(
    workspace: &'a mut Workspace,
    id: &ObjectId,
) -> SelectionResult<&'a mut Selection> {
    workspace
        .selections
        .iter_mut()
        .find(|selection| selection.id == *id)
        .ok_or_else(|| SelectionError::NotFound { id: id.clone() })
}

fn require_selection_index(workspace: &Workspace, id: &ObjectId) -> SelectionResult<usize> {
    workspace
        .selections
        .iter()
        .position(|selection| selection.id == *id)
        .ok_or_else(|| SelectionError::NotFound { id: id.clone() })
}

fn ensure_unique_selection_id(workspace: &Workspace, id: &ObjectId) -> SelectionResult<()> {
    if workspace
        .selections
        .iter()
        .any(|selection| selection.id == *id)
    {
        Err(SelectionError::DuplicateId { id: id.clone() })
    } else {
        Ok(())
    }
}

fn require_layers(workspace: &Workspace, ids: &[ObjectId]) -> SelectionResult<()> {
    for id in ids {
        if !workspace.layers.iter().any(|layer| layer.id == *id) {
            return Err(SelectionError::LayerNotFound { id: id.clone() });
        }
    }
    Ok(())
}

pub fn polygon_bounds(points: &[Point]) -> SelectionResult<Rect> {
    if points.len() < 3 {
        return Err(SelectionError::NotEnoughPoints { minimum: 3 });
    }
    let mut left = points[0].x;
    let mut top = points[0].y;
    let mut right = points[0].x;
    let mut bottom = points[0].y;
    for point in points {
        left = left.min(point.x);
        top = top.min(point.y);
        right = right.max(point.x);
        bottom = bottom.max(point.y);
    }
    validate_bounds(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })?;
    Ok(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

pub fn color_range_kind(color: RgbaColor, tolerance: f32) -> SelectionKind {
    SelectionKind::ColorRange {
        color,
        tolerance: tolerance.clamp(0.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BlendMode, ClippingBehavior, ExportParticipation, Layer, Transform, Workspace,
    };

    #[test]
    fn creates_alpha_masks_for_rectangular_and_elliptical_selections() {
        let mut workspace = workspace();
        workspace.layers.push(layer("base"));
        create_selection(
            &mut workspace,
            NewSelection {
                id: id("rect"),
                kind: SelectionKind::Rectangular,
                bounds: rect(0.0, 0.0, 4.0, 4.0),
                source_layer_ids: vec![id("base")],
            },
        )
        .expect("rect");
        create_selection(
            &mut workspace,
            NewSelection {
                id: id("ellipse"),
                kind: SelectionKind::Elliptical,
                bounds: rect(0.0, 0.0, 4.0, 4.0),
                source_layer_ids: vec![id("base")],
            },
        )
        .expect("ellipse");

        let rect_mask = workspace.selections[0].mask.as_ref().expect("mask");
        let ellipse_mask = workspace.selections[1].mask.as_ref().expect("mask");
        assert!(rect_mask.alpha.iter().all(|alpha| *alpha == 255));
        assert!(ellipse_mask.alpha.iter().any(|alpha| *alpha == 0));
        assert!(ellipse_mask.alpha.iter().any(|alpha| *alpha == 255));
    }

    #[test]
    fn edit_operations_preserve_mask_alpha_shape() {
        let mut workspace = workspace();
        workspace.layers.push(layer("base"));
        create_selection(
            &mut workspace,
            NewSelection {
                id: id("selection"),
                kind: SelectionKind::Rectangular,
                bounds: rect(2.0, 2.0, 4.0, 4.0),
                source_layer_ids: vec![id("base")],
            },
        )
        .expect("selection");

        feather_selection(&mut workspace, &id("selection"), 2.0).expect("feather");
        invert_selection(&mut workspace, &id("selection")).expect("invert");
        move_selection(&mut workspace, &id("selection"), 3.0, -1.0).expect("move");

        let selection = &workspace.selections[0];
        assert_eq!(selection.bounds.x, 5.0);
        assert_eq!(selection.bounds.y, 1.0);
        assert!(selection
            .mask
            .as_ref()
            .expect("mask")
            .alpha
            .iter()
            .any(|alpha| *alpha < 255));
    }

    #[test]
    fn conversion_uses_selection_bounds() {
        let mut workspace = workspace();
        workspace.layers.push(layer("base"));
        create_selection(
            &mut workspace,
            NewSelection {
                id: id("selection"),
                kind: SelectionKind::Rectangular,
                bounds: rect(10.0, 12.0, 20.0, 24.0),
                source_layer_ids: vec![id("base")],
            },
        )
        .expect("selection");

        layer_from_selection(
            &mut workspace,
            &id("selection"),
            id("from-selection"),
            "From Selection".to_owned(),
        )
        .expect("layer");
        export_area_from_selection(
            &mut workspace,
            &id("selection"),
            id("area"),
            "Area".to_owned(),
        )
        .expect("area");

        assert_eq!(workspace.layers[1].position, Point { x: 10.0, y: 12.0 });
        assert_eq!(workspace.layers[1].bounds.width, 20.0);
        assert_eq!(
            workspace.export_areas[0].bounds,
            rect(10.0, 12.0, 20.0, 24.0)
        );
        assert_eq!(
            workspace.export_areas[0].included_layer_ids,
            vec![id("base")]
        );
    }

    fn workspace() -> Workspace {
        Workspace::empty(id("workspace"))
    }

    fn layer(value: &str) -> Layer {
        Layer {
            id: id(value),
            name: value.to_owned(),
            visible: true,
            opacity: 1.0,
            locked: false,
            position: Point::ZERO,
            bounds: rect(0.0, 0.0, 16.0, 16.0),
            blend_mode: BlendMode::Normal,
            alpha_channel: true,
            transform: Transform::default(),
            clipping: ClippingBehavior::None,
            mask_layer_id: None,
            group_id: None,
            export_participation: ExportParticipation::Included,
            raster: None,
        }
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
