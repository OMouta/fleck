use crate::model::{
    ObjectId, Point, RasterPixels, Rect, RgbaColor, ScalingMode, Selection, Workspace,
};
use image::{imageops, ImageBuffer, Rgba};

#[derive(Debug, Clone, PartialEq)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub points: Vec<StrokePoint>,
    pub color: RgbaColor,
    pub radius: f32,
    pub opacity: f32,
    pub selection_id: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub start: Point,
    pub end: Point,
    pub start_color: RgbaColor,
    pub end_color: RgbaColor,
    pub selection_id: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CloneSample {
    pub source: Point,
    pub target: Point,
    pub radius: f32,
    pub selection_id: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PixelError {
    #[error("layer `{id}` was not found")]
    LayerNotFound { id: ObjectId },
    #[error("layer `{id}` is locked")]
    LockedLayer { id: ObjectId },
    #[error("selection `{id}` was not found")]
    SelectionNotFound { id: ObjectId },
    #[error("layer `{id}` has no raster pixels")]
    MissingRaster { id: ObjectId },
    #[error("pixel bounds must be positive")]
    NonPositiveBounds,
    #[error("stroke requires at least one point")]
    EmptyStroke,
    #[error("raster dimensions are too large")]
    RasterTooLarge,
}

pub type PixelResult<T> = Result<T, PixelError>;

pub fn move_layer(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    dx: f32,
    dy: f32,
) -> PixelResult<()> {
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    layer.position.x += dx;
    layer.position.y += dy;
    Ok(())
}

pub fn crop_layer(workspace: &mut Workspace, layer_id: &ObjectId, crop: Rect) -> PixelResult<()> {
    validate_rect(crop)?;
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;

    let x = crop.x.floor().max(0.0) as u32;
    let y = crop.y.floor().max(0.0) as u32;
    let right = (crop.x + crop.width)
        .ceil()
        .min(raster.width as f32)
        .max(x as f32) as u32;
    let bottom = (crop.y + crop.height)
        .ceil()
        .min(raster.height as f32)
        .max(y as f32) as u32;
    let width = right.saturating_sub(x).max(1);
    let height = bottom.saturating_sub(y).max(1);
    let mut pixels = vec![0; checked_len(width, height)?];
    for row in 0..height {
        let src = (((y + row) * raster.width + x) * 4) as usize;
        let dst = (row * width * 4) as usize;
        let len = (width * 4) as usize;
        pixels[dst..dst + len].copy_from_slice(&raster.pixels[src..src + len]);
    }

    layer.position.x += x as f32;
    layer.position.y += y as f32;
    layer.bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    *raster = RasterPixels {
        width,
        height,
        pixels,
    };
    Ok(())
}

pub fn resize_layer(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    width: u32,
    height: u32,
    scaling: ScalingMode,
) -> PixelResult<()> {
    if width == 0 || height == 0 {
        return Err(PixelError::NonPositiveBounds);
    }
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let image = image_from_raster(raster)?;
    let resized = imageops::resize(&image, width, height, filter_for_scaling(scaling));
    *raster = raster_from_image(resized);
    layer.bounds.width = width as f32;
    layer.bounds.height = height as f32;
    Ok(())
}

pub fn resize_canvas(workspace: &mut Workspace, origin: Point) -> PixelResult<()> {
    workspace.canvas.origin = origin;
    Ok(())
}

pub fn rotate_layer(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    degrees: i32,
) -> PixelResult<()> {
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let image = image_from_raster(raster)?;
    let rotated = match degrees.rem_euclid(360) {
        90 => imageops::rotate90(&image),
        180 => imageops::rotate180(&image),
        270 => imageops::rotate270(&image),
        _ => image,
    };
    *raster = raster_from_image(rotated);
    layer.bounds.width = raster.width as f32;
    layer.bounds.height = raster.height as f32;
    Ok(())
}

pub fn flip_layer(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    horizontal: bool,
) -> PixelResult<()> {
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let image = image_from_raster(raster)?;
    let flipped = if horizontal {
        imageops::flip_horizontal(&image)
    } else {
        imageops::flip_vertical(&image)
    };
    *raster = raster_from_image(flipped);
    Ok(())
}

pub fn brush(workspace: &mut Workspace, layer_id: &ObjectId, stroke: Stroke) -> PixelResult<()> {
    paint_stroke(workspace, layer_id, stroke, false)
}

pub fn pencil(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    mut stroke: Stroke,
) -> PixelResult<()> {
    stroke.radius = stroke.radius.max(0.5);
    stroke.opacity = 1.0;
    paint_stroke(workspace, layer_id, stroke, false)
}

pub fn eraser(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    mut stroke: Stroke,
) -> PixelResult<()> {
    stroke.color = RgbaColor {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    paint_stroke(workspace, layer_id, stroke, true)
}

pub fn fill(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    start: Point,
    color: RgbaColor,
    selection_id: Option<ObjectId>,
) -> PixelResult<()> {
    let selection = selection(workspace, selection_id.as_ref())?.cloned();
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let layer_position = layer.position;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let x = start.x.floor() as i32;
    let y = start.y.floor() as i32;
    if !in_bounds(raster, x, y) {
        return Ok(());
    }
    let target = pixel_at(raster, x as u32, y as u32);
    let mut stack = vec![(x, y)];
    let mut seen = vec![false; raster.width as usize * raster.height as usize];
    while let Some((cx, cy)) = stack.pop() {
        if !in_bounds(raster, cx, cy) {
            continue;
        }
        let seen_index = (cy as u32 * raster.width + cx as u32) as usize;
        if seen[seen_index] || pixel_at(raster, cx as u32, cy as u32) != target {
            continue;
        }
        seen[seen_index] = true;
        write_pixel(
            raster,
            layer_position,
            cx,
            cy,
            color,
            1.0,
            selection.as_ref(),
        );
        stack.extend([(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)]);
    }
    Ok(())
}

pub fn gradient(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    gradient: Gradient,
) -> PixelResult<()> {
    let selection = selection(workspace, gradient.selection_id.as_ref())?.cloned();
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let layer_position = layer.position;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let dx = gradient.end.x - gradient.start.x;
    let dy = gradient.end.y - gradient.start.y;
    let len2 = (dx * dx + dy * dy).max(1.0);
    for y in 0..raster.height as i32 {
        for x in 0..raster.width as i32 {
            let px = x as f32 - gradient.start.x;
            let py = y as f32 - gradient.start.y;
            let t = ((px * dx + py * dy) / len2).clamp(0.0, 1.0);
            write_pixel(
                raster,
                layer_position,
                x,
                y,
                lerp_color(gradient.start_color, gradient.end_color, t),
                1.0,
                selection.as_ref(),
            );
        }
    }
    Ok(())
}

pub fn color_picker(
    workspace: &Workspace,
    layer_id: &ObjectId,
    point: Point,
) -> PixelResult<RgbaColor> {
    let layer = require_layer(workspace, layer_id)?;
    let raster = require_raster(layer_id, &layer.raster)?;
    let x = point.x.floor() as i32;
    let y = point.y.floor() as i32;
    if !in_bounds(raster, x, y) {
        return Ok(RgbaColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        });
    }
    Ok(pixel_at(raster, x as u32, y as u32))
}

pub fn clone_pixels(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    sample: CloneSample,
) -> PixelResult<()> {
    copy_sample(workspace, layer_id, sample, 1.0)
}

pub fn heal(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    sample: CloneSample,
) -> PixelResult<()> {
    copy_sample(workspace, layer_id, sample, 0.5)
}

pub fn blur(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    selection_id: Option<ObjectId>,
) -> PixelResult<()> {
    convolve(workspace, layer_id, selection_id, &[[1.0 / 9.0; 3]; 3])
}

pub fn sharpen(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    selection_id: Option<ObjectId>,
) -> PixelResult<()> {
    convolve(
        workspace,
        layer_id,
        selection_id,
        &[[0.0, -1.0, 0.0], [-1.0, 5.0, -1.0], [0.0, -1.0, 0.0]],
    )
}

pub fn smudge(workspace: &mut Workspace, layer_id: &ObjectId, stroke: Stroke) -> PixelResult<()> {
    if stroke.points.len() < 2 {
        return Err(PixelError::EmptyStroke);
    }
    let selection = selection(workspace, stroke.selection_id.as_ref())?.cloned();
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let layer_position = layer.position;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let mut carried = color_picker_from_raster(raster, stroke.points[0].x, stroke.points[0].y);
    for point in stroke.points.iter().skip(1) {
        let current = color_picker_from_raster(raster, point.x, point.y);
        write_disc(
            raster,
            layer_position,
            point.x,
            point.y,
            stroke.radius,
            carried,
            stroke.opacity,
            selection.as_ref(),
            false,
        );
        carried = blend_colors(carried, current, 0.5);
    }
    Ok(())
}

fn paint_stroke(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    stroke: Stroke,
    erase: bool,
) -> PixelResult<()> {
    if stroke.points.is_empty() {
        return Err(PixelError::EmptyStroke);
    }
    let selection = selection(workspace, stroke.selection_id.as_ref())?.cloned();
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let layer_position = layer.position;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    for pair in stroke.points.windows(2) {
        draw_segment(
            raster,
            layer_position,
            &stroke,
            &pair[0],
            &pair[1],
            selection.as_ref(),
            erase,
        );
    }
    if stroke.points.len() == 1 {
        let point = &stroke.points[0];
        write_disc(
            raster,
            layer_position,
            point.x,
            point.y,
            stroke.radius,
            stroke.color,
            stroke.opacity,
            selection.as_ref(),
            erase,
        );
    }
    Ok(())
}

fn draw_segment(
    raster: &mut RasterPixels,
    layer_position: Point,
    stroke: &Stroke,
    start: &StrokePoint,
    end: &StrokePoint,
    selection: Option<&Selection>,
    erase: bool,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as u32;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        write_disc(
            raster,
            layer_position,
            start.x + dx * t,
            start.y + dy * t,
            stroke.radius,
            stroke.color,
            stroke.opacity,
            selection,
            erase,
        );
    }
}

fn write_disc(
    raster: &mut RasterPixels,
    layer_position: Point,
    cx: f32,
    cy: f32,
    radius: f32,
    color: RgbaColor,
    opacity: f32,
    selection: Option<&Selection>,
    erase: bool,
) {
    let radius = radius.max(0.5);
    let left = (cx - radius).floor() as i32;
    let top = (cy - radius).floor() as i32;
    let right = (cx + radius).ceil() as i32;
    let bottom = (cy + radius).ceil() as i32;
    for y in top..=bottom {
        for x in left..=right {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= radius * radius {
                write_pixel(raster, layer_position, x, y, color, opacity, selection);
                if erase
                    && in_bounds(raster, x, y)
                    && selection_alpha(selection, layer_position, x, y) > 0
                {
                    let index = ((y as u32 * raster.width + x as u32) * 4 + 3) as usize;
                    raster.pixels[index] = 0;
                }
            }
        }
    }
}

fn write_pixel(
    raster: &mut RasterPixels,
    layer_position: Point,
    x: i32,
    y: i32,
    color: RgbaColor,
    opacity: f32,
    selection: Option<&Selection>,
) {
    if !in_bounds(raster, x, y) {
        return;
    }
    let selection_alpha = selection_alpha(selection, layer_position, x, y);
    if selection_alpha == 0 {
        return;
    }
    let alpha = (opacity.clamp(0.0, 1.0) * (selection_alpha as f32 / 255.0)).clamp(0.0, 1.0);
    let index = ((y as u32 * raster.width + x as u32) * 4) as usize;
    let existing = RgbaColor {
        r: raster.pixels[index],
        g: raster.pixels[index + 1],
        b: raster.pixels[index + 2],
        a: raster.pixels[index + 3],
    };
    let blended = blend_colors(existing, color, alpha);
    raster.pixels[index..index + 4].copy_from_slice(&[blended.r, blended.g, blended.b, blended.a]);
}

fn selection_alpha(selection: Option<&Selection>, layer_position: Point, x: i32, y: i32) -> u8 {
    let Some(selection) = selection else {
        return 255;
    };
    let sx = (layer_position.x + x as f32 - selection.bounds.x).floor() as i32;
    let sy = (layer_position.y + y as f32 - selection.bounds.y).floor() as i32;
    if sx < 0 || sy < 0 {
        return 0;
    }
    match &selection.mask {
        Some(mask) if sx < mask.width as i32 && sy < mask.height as i32 => {
            mask.alpha[(sy as u32 * mask.width + sx as u32) as usize]
        }
        Some(_) => 0,
        None => {
            let inside = layer_position.x + x as f32 >= selection.bounds.x
                && layer_position.y + y as f32 >= selection.bounds.y
                && layer_position.x + (x as f32) < selection.bounds.x + selection.bounds.width
                && layer_position.y + (y as f32) < selection.bounds.y + selection.bounds.height;
            if inside {
                255
            } else {
                0
            }
        }
    }
}

fn copy_sample(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    sample: CloneSample,
    opacity: f32,
) -> PixelResult<()> {
    let selection = selection(workspace, sample.selection_id.as_ref())?.cloned();
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let layer_position = layer.position;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let radius = sample.radius.max(0.5);
    for y in -(radius.ceil() as i32)..=(radius.ceil() as i32) {
        for x in -(radius.ceil() as i32)..=(radius.ceil() as i32) {
            if (x * x + y * y) as f32 > radius * radius {
                continue;
            }
            let sx = sample.source.x.floor() as i32 + x;
            let sy = sample.source.y.floor() as i32 + y;
            let tx = sample.target.x.floor() as i32 + x;
            let ty = sample.target.y.floor() as i32 + y;
            if in_bounds(raster, sx, sy) {
                write_pixel(
                    raster,
                    layer_position,
                    tx,
                    ty,
                    pixel_at(raster, sx as u32, sy as u32),
                    opacity,
                    selection.as_ref(),
                );
            }
        }
    }
    Ok(())
}

fn convolve(
    workspace: &mut Workspace,
    layer_id: &ObjectId,
    selection_id: Option<ObjectId>,
    kernel: &[[f32; 3]; 3],
) -> PixelResult<()> {
    let selection = selection(workspace, selection_id.as_ref())?.cloned();
    let layer = require_layer_mut(workspace, layer_id)?;
    ensure_unlocked(layer)?;
    let layer_position = layer.position;
    let raster = require_raster_mut(layer_id, &mut layer.raster)?;
    let source = raster.pixels.clone();
    for y in 0..raster.height as i32 {
        for x in 0..raster.width as i32 {
            if selection_alpha(selection.as_ref(), layer_position, x, y) == 0 {
                continue;
            }
            let mut rgba = [0.0; 4];
            for ky in 0..3 {
                for kx in 0..3 {
                    let sx = (x + kx as i32 - 1).clamp(0, raster.width as i32 - 1) as u32;
                    let sy = (y + ky as i32 - 1).clamp(0, raster.height as i32 - 1) as u32;
                    let src = ((sy * raster.width + sx) * 4) as usize;
                    for channel in 0..4 {
                        rgba[channel] += source[src + channel] as f32 * kernel[ky][kx];
                    }
                }
            }
            let dst = ((y as u32 * raster.width + x as u32) * 4) as usize;
            for channel in 0..4 {
                raster.pixels[dst + channel] = rgba[channel].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

fn selection<'a>(
    workspace: &'a Workspace,
    id: Option<&ObjectId>,
) -> PixelResult<Option<&'a Selection>> {
    match id {
        Some(id) => workspace
            .selections
            .iter()
            .find(|selection| selection.id == *id)
            .map(Some)
            .ok_or_else(|| PixelError::SelectionNotFound { id: id.clone() }),
        None => Ok(None),
    }
}

fn require_layer<'a>(
    workspace: &'a Workspace,
    id: &ObjectId,
) -> PixelResult<&'a crate::model::Layer> {
    workspace
        .layers
        .iter()
        .find(|layer| layer.id == *id)
        .ok_or_else(|| PixelError::LayerNotFound { id: id.clone() })
}

fn require_layer_mut<'a>(
    workspace: &'a mut Workspace,
    id: &ObjectId,
) -> PixelResult<&'a mut crate::model::Layer> {
    workspace
        .layers
        .iter_mut()
        .find(|layer| layer.id == *id)
        .ok_or_else(|| PixelError::LayerNotFound { id: id.clone() })
}

fn ensure_unlocked(layer: &crate::model::Layer) -> PixelResult<()> {
    if layer.locked {
        Err(PixelError::LockedLayer {
            id: layer.id.clone(),
        })
    } else {
        Ok(())
    }
}

fn require_raster<'a>(
    id: &ObjectId,
    raster: &'a Option<RasterPixels>,
) -> PixelResult<&'a RasterPixels> {
    raster
        .as_ref()
        .ok_or_else(|| PixelError::MissingRaster { id: id.clone() })
}

fn require_raster_mut<'a>(
    id: &ObjectId,
    raster: &'a mut Option<RasterPixels>,
) -> PixelResult<&'a mut RasterPixels> {
    raster
        .as_mut()
        .ok_or_else(|| PixelError::MissingRaster { id: id.clone() })
}

fn validate_rect(rect: Rect) -> PixelResult<()> {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        Err(PixelError::NonPositiveBounds)
    } else {
        Ok(())
    }
}

fn checked_len(width: u32, height: u32) -> PixelResult<usize> {
    width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .map(|bytes| bytes as usize)
        .ok_or(PixelError::RasterTooLarge)
}

fn image_from_raster(raster: &RasterPixels) -> PixelResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    ImageBuffer::from_raw(raster.width, raster.height, raster.pixels.clone())
        .ok_or(PixelError::RasterTooLarge)
}

fn raster_from_image(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> RasterPixels {
    RasterPixels {
        width: image.width(),
        height: image.height(),
        pixels: image.into_raw(),
    }
}

fn filter_for_scaling(scaling: ScalingMode) -> imageops::FilterType {
    match scaling {
        ScalingMode::NearestNeighbor => imageops::FilterType::Nearest,
        ScalingMode::Bilinear => imageops::FilterType::Triangle,
        ScalingMode::Bicubic => imageops::FilterType::CatmullRom,
        ScalingMode::Lanczos => imageops::FilterType::Lanczos3,
    }
}

fn in_bounds(raster: &RasterPixels, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < raster.width as i32 && y < raster.height as i32
}

fn pixel_at(raster: &RasterPixels, x: u32, y: u32) -> RgbaColor {
    let index = ((y * raster.width + x) * 4) as usize;
    RgbaColor {
        r: raster.pixels[index],
        g: raster.pixels[index + 1],
        b: raster.pixels[index + 2],
        a: raster.pixels[index + 3],
    }
}

fn color_picker_from_raster(raster: &RasterPixels, x: f32, y: f32) -> RgbaColor {
    let x = x.floor() as i32;
    let y = y.floor() as i32;
    if in_bounds(raster, x, y) {
        pixel_at(raster, x as u32, y as u32)
    } else {
        RgbaColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

fn blend_colors(a: RgbaColor, b: RgbaColor, t: f32) -> RgbaColor {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    RgbaColor {
        r: (a.r as f32 * inv + b.r as f32 * t).round() as u8,
        g: (a.g as f32 * inv + b.g as f32 * t).round() as u8,
        b: (a.b as f32 * inv + b.b as f32 * t).round() as u8,
        a: (a.a as f32 * inv + b.a as f32 * t).round() as u8,
    }
}

fn lerp_color(start: RgbaColor, end: RgbaColor, t: f32) -> RgbaColor {
    blend_colors(start, end, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::{self, NewLayer};
    use crate::model::SelectionKind;

    #[test]
    fn brush_and_eraser_modify_raster_pixels() {
        let mut workspace = workspace_with_layer();
        brush(
            &mut workspace,
            &id("layer"),
            stroke(vec![(2.0, 2.0)], red(), 1.0, None),
        )
        .expect("brush");
        assert_eq!(pixel(&workspace, 2, 2), red());

        eraser(
            &mut workspace,
            &id("layer"),
            stroke(vec![(2.0, 2.0)], red(), 1.0, None),
        )
        .expect("eraser");
        assert_eq!(pixel(&workspace, 2, 2).a, 0);
    }

    #[test]
    fn selection_limits_fill() {
        let mut workspace = workspace_with_layer();
        workspace.selections.push(Selection {
            id: id("selection"),
            kind: SelectionKind::Rectangular,
            bounds: Rect {
                x: 1.0,
                y: 1.0,
                width: 2.0,
                height: 2.0,
            },
            feather_radius: 0.0,
            source_layer_ids: vec![id("layer")],
            mask: None,
        });

        fill(
            &mut workspace,
            &id("layer"),
            Point { x: 0.0, y: 0.0 },
            red(),
            Some(id("selection")),
        )
        .expect("fill");

        assert_eq!(pixel(&workspace, 0, 0).a, 0);
        assert_eq!(pixel(&workspace, 1, 1), red());
    }

    #[test]
    fn crop_resize_rotate_and_flip_update_raster_shape() {
        let mut workspace = workspace_with_layer();
        brush(
            &mut workspace,
            &id("layer"),
            stroke(vec![(1.0, 1.0)], red(), 0.5, None),
        )
        .expect("brush");

        crop_layer(
            &mut workspace,
            &id("layer"),
            Rect {
                x: 1.0,
                y: 1.0,
                width: 2.0,
                height: 3.0,
            },
        )
        .expect("crop");
        resize_layer(
            &mut workspace,
            &id("layer"),
            4,
            2,
            ScalingMode::NearestNeighbor,
        )
        .expect("resize");
        rotate_layer(&mut workspace, &id("layer"), 90).expect("rotate");
        flip_layer(&mut workspace, &id("layer"), true).expect("flip");

        let raster = workspace.layers[0].raster.as_ref().expect("raster");
        assert_eq!((raster.width, raster.height), (2, 4));
        assert_eq!(workspace.layers[0].bounds.width, 2.0);
        assert_eq!(workspace.layers[0].bounds.height, 4.0);
    }

    fn workspace_with_layer() -> Workspace {
        let mut workspace = Workspace::empty(id("workspace"));
        layer::create_layer(
            &mut workspace,
            NewLayer {
                id: id("layer"),
                name: "Layer".to_owned(),
                bounds: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 4.0,
                    height: 4.0,
                },
                position: Point::ZERO,
            },
        )
        .expect("layer");
        workspace
    }

    fn stroke(
        points: Vec<(f32, f32)>,
        color: RgbaColor,
        radius: f32,
        selection_id: Option<ObjectId>,
    ) -> Stroke {
        Stroke {
            points: points
                .into_iter()
                .map(|(x, y)| StrokePoint { x, y })
                .collect(),
            color,
            radius,
            opacity: 1.0,
            selection_id,
        }
    }

    fn pixel(workspace: &Workspace, x: u32, y: u32) -> RgbaColor {
        let raster = workspace.layers[0].raster.as_ref().expect("raster");
        pixel_at(raster, x, y)
    }

    fn red() -> RgbaColor {
        RgbaColor {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id")
    }
}
