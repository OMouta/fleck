use crate::model::{
    CompressionSettings, Area, ExportBackground, ExportParticipation, MetadataBehavior,
    ObjectGroup, ObjectId, OutputDefinition, OutputFormat, Padding, Rect, TransparencyBehavior,
    TrimBehavior, Workspace,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NewArea {
    pub id: ObjectId,
    pub name: String,
    pub bounds: Rect,
    pub padding: Padding,
    pub background: ExportBackground,
    pub trim: TrimBehavior,
    pub output_ids: Vec<ObjectId>,
    pub included_layer_ids: Vec<ObjectId>,
    pub excluded_layer_ids: Vec<ObjectId>,
    pub tags: Vec<String>,
    pub preset_id: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewOutput {
    pub id: ObjectId,
    pub filename: String,
    pub folder: Option<String>,
    pub format: OutputFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale: f32,
    pub quality: Option<u8>,
    pub compression: CompressionSettings,
    pub background: ExportBackground,
    pub transparency: TransparencyBehavior,
    pub metadata: MetadataBehavior,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OutputUpdate {
    pub filename: Option<String>,
    pub folder: Option<Option<String>>,
    pub format: Option<OutputFormat>,
    pub width: Option<Option<u32>>,
    pub height: Option<Option<u32>>,
    pub scale: Option<f32>,
    pub quality: Option<Option<u8>>,
    pub background: Option<ExportBackground>,
    pub transparency: Option<TransparencyBehavior>,
    pub metadata: Option<MetadataBehavior>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPreview {
    pub area_id: ObjectId,
    pub name: String,
    pub source_bounds: PixelRect,
    pub padded_bounds: PixelRect,
    pub participating_layer_ids: Vec<ObjectId>,
    pub outputs: Vec<OutputPreview>,
    pub warnings: Vec<ExportWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPreview {
    pub output_id: ObjectId,
    pub filename: String,
    pub destination: Option<String>,
    pub format: OutputFormat,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub scale: OutputScale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputScale {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportWarning {
    NoOutputs,
    NoParticipatingLayers,
    LayerIncludedAndExcluded { layer_id: ObjectId },
    JpegCannotPreserveTransparency { output_id: ObjectId },
    CheckerboardPreviewBackground { output_id: ObjectId },
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExportError {
    #[error("area `{id}` was not found")]
    AreaNotFound { id: ObjectId },
    #[error("output `{id}` was not found")]
    OutputNotFound { id: ObjectId },
    #[error("layer `{id}` was not found")]
    LayerNotFound { id: ObjectId },
    #[error("area `{id}` already exists")]
    DuplicateAreaId { id: ObjectId },
    #[error("output `{id}` already exists")]
    DuplicateOutputId { id: ObjectId },
    #[error("object group `{id}` already exists")]
    DuplicateGroupId { id: ObjectId },
    #[error("area bounds must be positive")]
    NonPositiveAreaBounds,
    #[error("output filename cannot be empty")]
    EmptyFilename,
    #[error("output dimensions and scale must be positive")]
    InvalidOutputSize,
}

pub type ExportResult<T> = Result<T, ExportError>;

pub fn create_area(workspace: &mut Workspace, area: NewArea) -> ExportResult<()> {
    ensure_area_id_available(workspace, &area.id)?;
    validate_area_bounds(area.bounds)?;
    require_outputs(workspace, &area.output_ids)?;
    require_layers(workspace, &area.included_layer_ids)?;
    require_layers(workspace, &area.excluded_layer_ids)?;
    workspace.areas.push(Area {
        id: area.id,
        name: area.name,
        bounds: area.bounds,
        layers: Vec::new(),
        padding: area.padding,
        background: area.background,
        trim: area.trim,
        output_ids: area.output_ids,
        included_layer_ids: area.included_layer_ids,
        excluded_layer_ids: area.excluded_layer_ids,
        tags: normalized_tags(area.tags),
        preset_id: area.preset_id,
    });
    Ok(())
}

pub fn rename_area(
    workspace: &mut Workspace,
    id: &ObjectId,
    name: String,
) -> ExportResult<()> {
    require_area_mut(workspace, id)?.name = name;
    Ok(())
}

pub fn move_area(
    workspace: &mut Workspace,
    id: &ObjectId,
    x: f32,
    y: f32,
) -> ExportResult<()> {
    let area = require_area_mut(workspace, id)?;
    area.bounds.x = x;
    area.bounds.y = y;
    Ok(())
}

pub fn resize_area(
    workspace: &mut Workspace,
    id: &ObjectId,
    width: f32,
    height: f32,
) -> ExportResult<()> {
    validate_area_bounds(Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    })?;
    let area = require_area_mut(workspace, id)?;
    area.bounds.width = width;
    area.bounds.height = height;
    Ok(())
}

pub fn set_area_padding(
    workspace: &mut Workspace,
    id: &ObjectId,
    padding: Padding,
) -> ExportResult<()> {
    let area = require_area_mut(workspace, id)?;
    area.padding = padding;
    Ok(())
}

pub fn set_area_background(
    workspace: &mut Workspace,
    id: &ObjectId,
    background: ExportBackground,
) -> ExportResult<()> {
    let area = require_area_mut(workspace, id)?;
    area.background = background;
    Ok(())
}

pub fn duplicate_area(
    workspace: &mut Workspace,
    id: &ObjectId,
    new_id: ObjectId,
) -> ExportResult<()> {
    ensure_area_id_available(workspace, &new_id)?;
    let source = require_area(workspace, id)?;
    let mut duplicate = source.clone();
    duplicate.id = new_id;
    duplicate.name = format!("{} Copy", duplicate.name);
    workspace.areas.push(duplicate);
    Ok(())
}

pub fn set_area_tags(
    workspace: &mut Workspace,
    id: &ObjectId,
    tags: Vec<String>,
) -> ExportResult<()> {
    require_area_mut(workspace, id)?.tags = normalized_tags(tags);
    Ok(())
}

pub fn group_area(
    workspace: &mut Workspace,
    id: &ObjectId,
    group_id: ObjectId,
    name: String,
) -> ExportResult<()> {
    require_area(workspace, id)?;
    if workspace
        .object_groups
        .iter()
        .any(|group| group.id == group_id)
    {
        return Err(ExportError::DuplicateGroupId { id: group_id });
    }
    workspace.object_groups.push(ObjectGroup {
        id: group_id,
        name,
        member_ids: vec![id.clone()],
    });
    Ok(())
}

pub fn delete_area(workspace: &mut Workspace, id: &ObjectId) -> ExportResult<Area> {
    let index = require_area_index(workspace, id)?;
    let deleted = workspace.areas.remove(index);
    for group in &mut workspace.object_groups {
        group.member_ids.retain(|member_id| member_id != id);
    }
    workspace
        .object_groups
        .retain(|group| !group.member_ids.is_empty());
    Ok(deleted)
}

pub fn add_output(workspace: &mut Workspace, output: NewOutput) -> ExportResult<()> {
    ensure_output_id_available(workspace, &output.id)?;
    validate_output(&output.filename, output.width, output.height, output.scale)?;
    workspace.outputs.push(OutputDefinition {
        id: output.id,
        filename: output.filename,
        folder: output.folder,
        format: output.format,
        width: output.width,
        height: output.height,
        scale: output.scale,
        quality: output.quality,
        compression: output.compression,
        background: output.background,
        transparency: output.transparency,
        metadata: output.metadata,
    });
    Ok(())
}

pub fn remove_output(workspace: &mut Workspace, id: &ObjectId) -> ExportResult<OutputDefinition> {
    let index = require_output_index(workspace, id)?;
    let removed = workspace.outputs.remove(index);
    for area in &mut workspace.areas {
        area.output_ids.retain(|output_id| output_id != id);
    }
    Ok(removed)
}

pub fn duplicate_output(
    workspace: &mut Workspace,
    id: &ObjectId,
    new_id: ObjectId,
) -> ExportResult<()> {
    ensure_output_id_available(workspace, &new_id)?;
    let source = require_output(workspace, id)?;
    let mut duplicate = source.clone();
    duplicate.id = new_id;
    duplicate.filename = copied_filename(&duplicate.filename);
    workspace.outputs.push(duplicate);
    Ok(())
}

pub fn update_output(
    workspace: &mut Workspace,
    id: &ObjectId,
    update: OutputUpdate,
) -> ExportResult<()> {
    let output = require_output_mut(workspace, id)?;
    let filename = update
        .filename
        .as_deref()
        .unwrap_or(output.filename.as_str());
    let width = update.width.unwrap_or(output.width);
    let height = update.height.unwrap_or(output.height);
    let scale = update.scale.unwrap_or(output.scale);
    validate_output(filename, width, height, scale)?;

    if let Some(filename) = update.filename {
        output.filename = filename;
    }
    if let Some(folder) = update.folder {
        output.folder = folder;
    }
    if let Some(format) = update.format {
        output.format = format;
    }
    if let Some(width) = update.width {
        output.width = width;
    }
    if let Some(height) = update.height {
        output.height = height;
    }
    if let Some(scale) = update.scale {
        output.scale = scale;
    }
    if let Some(quality) = update.quality {
        output.quality = quality;
    }
    if let Some(background) = update.background {
        output.background = background;
    }
    if let Some(transparency) = update.transparency {
        output.transparency = transparency;
    }
    if let Some(metadata) = update.metadata {
        output.metadata = metadata;
    }
    Ok(())
}

pub fn attach_output_to_area(
    workspace: &mut Workspace,
    area_id: &ObjectId,
    output_id: ObjectId,
) -> ExportResult<()> {
    require_output(workspace, &output_id)?;
    let area = require_area_mut(workspace, area_id)?;
    if !area.output_ids.contains(&output_id) {
        area.output_ids.push(output_id);
    }
    Ok(())
}

pub fn detach_output_from_area(
    workspace: &mut Workspace,
    area_id: &ObjectId,
    output_id: &ObjectId,
) -> ExportResult<()> {
    let area = require_area_mut(workspace, area_id)?;
    area.output_ids.retain(|id| id != output_id);
    Ok(())
}

pub fn set_layer_inclusion(
    workspace: &mut Workspace,
    area_id: &ObjectId,
    layer_id: ObjectId,
    participation: ExportParticipation,
) -> ExportResult<()> {
    require_layer(workspace, &layer_id)?;
    let area = require_area_mut(workspace, area_id)?;
    area.included_layer_ids.retain(|id| id != &layer_id);
    area.excluded_layer_ids.retain(|id| id != &layer_id);
    match participation {
        ExportParticipation::Included => area.included_layer_ids.push(layer_id),
        ExportParticipation::Excluded => area.excluded_layer_ids.push(layer_id),
        ExportParticipation::Inherit => {}
    }
    Ok(())
}

pub fn preview_workspace_exports(workspace: &Workspace) -> Vec<ExportPreview> {
    workspace
        .areas
        .iter()
        .map(|area| preview_area(workspace, &area.id))
        .collect::<ExportResult<Vec<_>>>()
        .unwrap_or_default()
}

pub fn preview_area(
    workspace: &Workspace,
    area_id: &ObjectId,
) -> ExportResult<ExportPreview> {
    let area = require_area(workspace, area_id)?;
    let participating_layer_ids = participating_layers(workspace, area);
    let mut warnings = Vec::new();
    for layer_id in &area.included_layer_ids {
        if area.excluded_layer_ids.contains(layer_id) {
            warnings.push(ExportWarning::LayerIncludedAndExcluded {
                layer_id: layer_id.clone(),
            });
        }
    }
    if area.output_ids.is_empty() {
        warnings.push(ExportWarning::NoOutputs);
    }
    if participating_layer_ids.is_empty() {
        warnings.push(ExportWarning::NoParticipatingLayers);
    }

    let source_bounds = pixel_rect(area.bounds);
    let padded_bounds = padded_pixel_rect(area.bounds, area.padding);
    let mut outputs = Vec::new();
    for output_id in &area.output_ids {
        let output = require_output(workspace, output_id)?;
        let output_preview = output_preview(output, padded_bounds);
        if output.format == OutputFormat::Jpeg
            && output.transparency == TransparencyBehavior::Preserve
        {
            warnings.push(ExportWarning::JpegCannotPreserveTransparency {
                output_id: output.id.clone(),
            });
        }
        if matches!(output.background, ExportBackground::CheckerboardPreview) {
            warnings.push(ExportWarning::CheckerboardPreviewBackground {
                output_id: output.id.clone(),
            });
        }
        outputs.push(output_preview);
    }

    Ok(ExportPreview {
        area_id: area.id.clone(),
        name: area.name.clone(),
        source_bounds,
        padded_bounds,
        participating_layer_ids,
        outputs,
        warnings,
    })
}

fn output_preview(output: &OutputDefinition, bounds: PixelRect) -> OutputPreview {
    let pixel_width = output
        .width
        .unwrap_or_else(|| scaled_dimension(bounds.width, output.scale));
    let pixel_height = output
        .height
        .unwrap_or_else(|| scaled_dimension(bounds.height, output.scale));
    OutputPreview {
        output_id: output.id.clone(),
        filename: output.filename.clone(),
        destination: output
            .folder
            .as_ref()
            .map(|folder| format!("{folder}/{}", output.filename)),
        format: output.format,
        pixel_width,
        pixel_height,
        scale: scale_ratio(output.scale),
    }
}

fn participating_layers(workspace: &Workspace, area: &Area) -> Vec<ObjectId> {
    workspace
        .layers()
        .filter(|layer| layer.visible && layer.opacity > 0.0)
        .filter(|layer| layer.export_participation != ExportParticipation::Excluded)
        .filter(|layer| {
            if area.excluded_layer_ids.contains(&layer.id) {
                return false;
            }
            area.included_layer_ids.is_empty() || area.included_layer_ids.contains(&layer.id)
        })
        .map(|layer| layer.id.clone())
        .collect()
}

fn validate_area_bounds(bounds: Rect) -> ExportResult<()> {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        Err(ExportError::NonPositiveAreaBounds)
    } else {
        Ok(())
    }
}

fn validate_output(
    filename: &str,
    width: Option<u32>,
    height: Option<u32>,
    scale: f32,
) -> ExportResult<()> {
    if filename.trim().is_empty() {
        return Err(ExportError::EmptyFilename);
    }
    if width == Some(0) || height == Some(0) || scale <= 0.0 {
        return Err(ExportError::InvalidOutputSize);
    }
    Ok(())
}

fn normalized_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|tag| tag.trim().to_owned())
        .filter(|tag| !tag.is_empty())
        .fold(Vec::new(), |mut tags, tag| {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
            tags
        })
}

fn copied_filename(filename: &str) -> String {
    if let Some((stem, extension)) = filename.rsplit_once('.') {
        format!("{stem}-copy.{extension}")
    } else {
        format!("{filename}-copy")
    }
}

fn padded_pixel_rect(bounds: Rect, padding: Padding) -> PixelRect {
    pixel_rect(Rect {
        x: bounds.x - padding.left,
        y: bounds.y - padding.top,
        width: bounds.width + padding.left + padding.right,
        height: bounds.height + padding.top + padding.bottom,
    })
}

fn pixel_rect(bounds: Rect) -> PixelRect {
    PixelRect {
        x: bounds.x.floor() as i32,
        y: bounds.y.floor() as i32,
        width: bounds.width.ceil().max(1.0) as u32,
        height: bounds.height.ceil().max(1.0) as u32,
    }
}

fn scaled_dimension(value: u32, scale: f32) -> u32 {
    ((value as f32) * scale).round().max(1.0) as u32
}

fn scale_ratio(scale: f32) -> OutputScale {
    const DENOMINATOR: u32 = 1000;
    OutputScale {
        numerator: (scale * DENOMINATOR as f32).round().max(1.0) as u32,
        denominator: DENOMINATOR,
    }
}

fn require_area<'a>(workspace: &'a Workspace, id: &ObjectId) -> ExportResult<&'a Area> {
    workspace
        .areas
        .iter()
        .find(|area| area.id == *id)
        .ok_or_else(|| ExportError::AreaNotFound { id: id.clone() })
}

fn require_area_mut<'a>(
    workspace: &'a mut Workspace,
    id: &ObjectId,
) -> ExportResult<&'a mut Area> {
    workspace
        .areas
        .iter_mut()
        .find(|area| area.id == *id)
        .ok_or_else(|| ExportError::AreaNotFound { id: id.clone() })
}

fn require_area_index(workspace: &Workspace, id: &ObjectId) -> ExportResult<usize> {
    workspace
        .areas
        .iter()
        .position(|area| area.id == *id)
        .ok_or_else(|| ExportError::AreaNotFound { id: id.clone() })
}

fn require_output<'a>(
    workspace: &'a Workspace,
    id: &ObjectId,
) -> ExportResult<&'a OutputDefinition> {
    workspace
        .outputs
        .iter()
        .find(|output| output.id == *id)
        .ok_or_else(|| ExportError::OutputNotFound { id: id.clone() })
}

fn require_output_mut<'a>(
    workspace: &'a mut Workspace,
    id: &ObjectId,
) -> ExportResult<&'a mut OutputDefinition> {
    workspace
        .outputs
        .iter_mut()
        .find(|output| output.id == *id)
        .ok_or_else(|| ExportError::OutputNotFound { id: id.clone() })
}

fn require_output_index(workspace: &Workspace, id: &ObjectId) -> ExportResult<usize> {
    workspace
        .outputs
        .iter()
        .position(|output| output.id == *id)
        .ok_or_else(|| ExportError::OutputNotFound { id: id.clone() })
}

fn require_layer<'a>(
    workspace: &'a Workspace,
    id: &ObjectId,
) -> ExportResult<&'a crate::model::Layer> {
    workspace
        .layers()
        .find(|layer| layer.id == *id)
        .ok_or_else(|| ExportError::LayerNotFound { id: id.clone() })
}

fn require_outputs(workspace: &Workspace, ids: &[ObjectId]) -> ExportResult<()> {
    for id in ids {
        require_output(workspace, id)?;
    }
    Ok(())
}

fn require_layers(workspace: &Workspace, ids: &[ObjectId]) -> ExportResult<()> {
    for id in ids {
        require_layer(workspace, id)?;
    }
    Ok(())
}

fn ensure_area_id_available(workspace: &Workspace, id: &ObjectId) -> ExportResult<()> {
    if workspace.areas.iter().any(|area| area.id == *id) {
        Err(ExportError::DuplicateAreaId { id: id.clone() })
    } else {
        Ok(())
    }
}

fn ensure_output_id_available(workspace: &Workspace, id: &ObjectId) -> ExportResult<()> {
    if workspace.outputs.iter().any(|output| output.id == *id) {
        Err(ExportError::DuplicateOutputId { id: id.clone() })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BlendMode, ClippingBehavior, Layer, Point, Transform};

    #[test]
    fn area_and_output_operations_manage_metadata_only() {
        let mut workspace = workspace();
        add_output(&mut workspace, output("png")).expect("add output");
        create_area(&mut workspace, area("icon")).expect("create area");
        workspace.areas[0].layers.push(layer("base"));

        rename_area(&mut workspace, &id("icon"), "app-icon".to_owned()).expect("rename");
        move_area(&mut workspace, &id("icon"), 10.0, 20.0).expect("move");
        resize_area(&mut workspace, &id("icon"), 64.0, 32.0).expect("resize");
        set_area_tags(
            &mut workspace,
            &id("icon"),
            vec![" app ".to_owned(), "app".to_owned(), "icon".to_owned()],
        )
        .expect("tags");
        set_layer_inclusion(
            &mut workspace,
            &id("icon"),
            id("base"),
            ExportParticipation::Included,
        )
        .expect("include");

        let area = &workspace.areas[0];
        assert_eq!(area.name, "app-icon");
        assert_eq!(area.bounds.x, 10.0);
        assert_eq!(area.bounds.width, 64.0);
        assert_eq!(area.tags, vec!["app", "icon"]);
        assert_eq!(area.included_layer_ids, vec![id("base")]);
        assert!(workspace.layers().any(|layer| layer.id == id("base")));
    }

    #[test]
    fn output_operations_update_area_references() {
        let mut workspace = workspace();
        add_output(&mut workspace, output("png")).expect("add");
        duplicate_output(&mut workspace, &id("png"), id("webp")).expect("duplicate");
        update_output(
            &mut workspace,
            &id("webp"),
            OutputUpdate {
                filename: Some("icon.webp".to_owned()),
                format: Some(OutputFormat::WebP),
                scale: Some(2.0),
                ..OutputUpdate::default()
            },
        )
        .expect("update");
        create_area(&mut workspace, area("icon")).expect("area");
        attach_output_to_area(&mut workspace, &id("icon"), id("webp")).expect("attach");
        remove_output(&mut workspace, &id("png")).expect("remove");

        assert_eq!(workspace.outputs.len(), 1);
        assert_eq!(workspace.outputs[0].filename, "icon.webp");
        assert_eq!(workspace.areas[0].output_ids, vec![id("webp")]);
    }

    #[test]
    fn preview_reports_outputs_dimensions_layers_and_warnings() {
        let mut workspace = workspace();
        let mut jpeg = output("jpeg");
        jpeg.format = OutputFormat::Jpeg;
        jpeg.transparency = TransparencyBehavior::Preserve;
        jpeg.width = Some(128);
        add_output(&mut workspace, jpeg).expect("output");
        let mut area = area("hero");
        area.bounds = Rect {
            x: 1.2,
            y: 2.4,
            width: 40.2,
            height: 20.0,
        };
        area.padding = Padding {
            top: 2.0,
            right: 3.0,
            bottom: 4.0,
            left: 5.0,
        };
        area.output_ids = vec![id("jpeg")];
        create_area(&mut workspace, area).expect("area");
        workspace.areas[0].layers.push(layer("base"));

        let preview = preview_area(&workspace, &id("hero")).expect("preview");

        assert_eq!(preview.source_bounds.width, 41);
        assert_eq!(preview.padded_bounds.width, 49);
        assert_eq!(preview.participating_layer_ids, vec![id("base")]);
        assert_eq!(preview.outputs[0].pixel_width, 128);
        assert_eq!(preview.outputs[0].pixel_height, 26);
        assert!(preview.warnings.iter().any(|warning| {
            matches!(
                warning,
                ExportWarning::JpegCannotPreserveTransparency { .. }
            )
        }));
    }

    fn workspace() -> Workspace {
        Workspace::empty(id("workspace"))
    }

    fn area(value: &str) -> NewArea {
        NewArea {
            id: id(value),
            name: value.to_owned(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            padding: Padding::default(),
            background: ExportBackground::Transparent,
            trim: TrimBehavior::None,
            output_ids: vec![id("png")],
            included_layer_ids: Vec::new(),
            excluded_layer_ids: Vec::new(),
            tags: Vec::new(),
            preset_id: None,
        }
    }

    fn output(value: &str) -> NewOutput {
        NewOutput {
            id: id(value),
            filename: format!("{value}.png"),
            folder: None,
            format: OutputFormat::Png,
            width: None,
            height: None,
            scale: 1.0,
            quality: None,
            compression: CompressionSettings::default(),
            background: ExportBackground::Transparent,
            transparency: TransparencyBehavior::Preserve,
            metadata: MetadataBehavior::Strip,
        }
    }

    fn layer(value: &str) -> Layer {
        Layer {
            id: id(value),
            name: value.to_owned(),
            visible: true,
            opacity: 1.0,
            locked: false,
            position: Point::ZERO,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 16.0,
            },
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

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }
}
