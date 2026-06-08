use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

pub const CURRENT_WORKSPACE_FORMAT_VERSION: u32 = 1;

pub use serde_json_value::JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationIssue> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ValidationIssue::EmptyId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub format_version: u32,
    pub id: ObjectId,
    pub metadata: WorkspaceMetadata,
    pub canvas: CanvasSettings,
    pub layers: Vec<Layer>,
    pub image_objects: Vec<ImageObject>,
    pub selections: Vec<Selection>,
    pub guides: Vec<Guide>,
    pub export_areas: Vec<ExportArea>,
    pub outputs: Vec<OutputDefinition>,
    pub recipes: Vec<Recipe>,
    pub assets: Vec<Asset>,
    pub object_groups: Vec<ObjectGroup>,
    pub history: HistoryState,
    pub document_settings: DocumentSettings,
}

impl Workspace {
    pub fn empty(id: ObjectId) -> Self {
        Self {
            format_version: CURRENT_WORKSPACE_FORMAT_VERSION,
            id,
            metadata: WorkspaceMetadata::default(),
            canvas: CanvasSettings::default(),
            layers: Vec::new(),
            image_objects: Vec::new(),
            selections: Vec::new(),
            guides: Vec::new(),
            export_areas: Vec::new(),
            outputs: Vec::new(),
            recipes: Vec::new(),
            assets: Vec::new(),
            object_groups: Vec::new(),
            history: HistoryState::default(),
            document_settings: DocumentSettings::default(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut issues = Vec::new();

        if self.format_version == 0 {
            issues.push(ValidationIssue::InvalidFormatVersion {
                version: self.format_version,
            });
        }

        collect_duplicate_ids(
            &mut issues,
            "layer",
            self.layers.iter().map(|layer| &layer.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "image_object",
            self.image_objects.iter().map(|image| &image.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "selection",
            self.selections.iter().map(|selection| &selection.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "guide",
            self.guides.iter().map(|guide| &guide.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "export_area",
            self.export_areas.iter().map(|area| &area.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "output",
            self.outputs.iter().map(|output| &output.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "recipe",
            self.recipes.iter().map(|recipe| &recipe.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "asset",
            self.assets.iter().map(|asset| &asset.id),
        );
        collect_duplicate_ids(
            &mut issues,
            "group",
            self.object_groups.iter().map(|group| &group.id),
        );

        let layer_ids = self
            .layers
            .iter()
            .map(|layer| &layer.id)
            .collect::<HashSet<_>>();
        let asset_ids = self
            .assets
            .iter()
            .map(|asset| &asset.id)
            .collect::<HashSet<_>>();
        let output_ids = self
            .outputs
            .iter()
            .map(|output| &output.id)
            .collect::<HashSet<_>>();

        for layer in &self.layers {
            if !(0.0..=1.0).contains(&layer.opacity) {
                issues.push(ValidationIssue::OpacityOutOfRange {
                    object_kind: "layer",
                    id: layer.id.clone(),
                    opacity: layer.opacity,
                });
            }
            if let Some(mask_id) = &layer.mask_layer_id {
                require_reference(
                    &mut issues,
                    "layer.mask_layer_id",
                    &layer.id,
                    mask_id,
                    &layer_ids,
                );
            }
            if let Some(raster) = &layer.raster {
                let expected_len = raster.width as usize * raster.height as usize * 4;
                if raster.pixels.len() != expected_len {
                    issues.push(ValidationIssue::InvalidRasterPixels {
                        layer_id: layer.id.clone(),
                        expected_len,
                        actual_len: raster.pixels.len(),
                    });
                }
            }
        }

        for image in &self.image_objects {
            if !(0.0..=1.0).contains(&image.opacity) {
                issues.push(ValidationIssue::OpacityOutOfRange {
                    object_kind: "image_object",
                    id: image.id.clone(),
                    opacity: image.opacity,
                });
            }
            require_reference(
                &mut issues,
                "image_object.source_asset_id",
                &image.id,
                &image.source_asset_id,
                &asset_ids,
            );
        }

        for export_area in &self.export_areas {
            if export_area.bounds.width <= 0.0 || export_area.bounds.height <= 0.0 {
                issues.push(ValidationIssue::NonPositiveBounds {
                    object_kind: "export_area",
                    id: export_area.id.clone(),
                });
            }
            for output_id in &export_area.output_ids {
                require_reference(
                    &mut issues,
                    "export_area.output_ids",
                    &export_area.id,
                    output_id,
                    &output_ids,
                );
            }
            for layer_id in &export_area.included_layer_ids {
                require_reference(
                    &mut issues,
                    "export_area.included_layer_ids",
                    &export_area.id,
                    layer_id,
                    &layer_ids,
                );
            }
            for layer_id in &export_area.excluded_layer_ids {
                require_reference(
                    &mut issues,
                    "export_area.excluded_layer_ids",
                    &export_area.id,
                    layer_id,
                    &layer_ids,
                );
            }
        }

        for selection in &self.selections {
            if selection.bounds.width <= 0.0 || selection.bounds.height <= 0.0 {
                issues.push(ValidationIssue::NonPositiveBounds {
                    object_kind: "selection",
                    id: selection.id.clone(),
                });
            }
            for layer_id in &selection.source_layer_ids {
                require_reference(
                    &mut issues,
                    "selection.source_layer_ids",
                    &selection.id,
                    layer_id,
                    &layer_ids,
                );
            }
            if let Some(mask) = &selection.mask {
                let expected_len = mask.width as usize * mask.height as usize;
                if mask.alpha.len() != expected_len {
                    issues.push(ValidationIssue::InvalidSelectionMask {
                        selection_id: selection.id.clone(),
                        expected_len,
                        actual_len: mask.alpha.len(),
                    });
                }
            }
        }

        for output in &self.outputs {
            if output.filename.trim().is_empty() {
                issues.push(ValidationIssue::EmptyFilename {
                    output_id: output.id.clone(),
                });
            }
            if output.scale <= 0.0 || output.width == Some(0) || output.height == Some(0) {
                issues.push(ValidationIssue::InvalidOutputSize {
                    output_id: output.id.clone(),
                });
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { issues })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub name: String,
    pub created_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for WorkspaceMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled Workspace".to_owned(),
            created_by: APP_NAME.to_owned(),
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSettings {
    pub origin: Point,
    pub background: CanvasBackground,
    pub transparency_grid: TransparencyGrid,
    pub default_scaling: ScalingMode,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            origin: Point::ZERO,
            background: CanvasBackground::Transparent,
            transparency_grid: TransparencyGrid::default(),
            default_scaling: ScalingMode::Lanczos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: ObjectId,
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub locked: bool,
    pub position: Point,
    pub bounds: Rect,
    pub blend_mode: BlendMode,
    pub alpha_channel: bool,
    pub transform: Transform,
    pub clipping: ClippingBehavior,
    pub mask_layer_id: Option<ObjectId>,
    pub group_id: Option<ObjectId>,
    pub export_participation: ExportParticipation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raster: Option<RasterPixels>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RasterPixels {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: ObjectId,
    pub name: String,
    pub source_asset_id: ObjectId,
    pub position: Point,
    pub scale: Size,
    pub rotation_degrees: f32,
    pub opacity: f32,
    pub crop_bounds: Option<Rect>,
    pub rasterized_layer_id: Option<ObjectId>,
    pub export_inclusion: ExportParticipation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    pub id: ObjectId,
    pub kind: SelectionKind,
    pub bounds: Rect,
    pub feather_radius: f32,
    pub source_layer_ids: Vec<ObjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<SelectionMask>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionMask {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    pub id: ObjectId,
    pub axis: Axis,
    pub position: f32,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportArea {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputDefinition {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: ObjectId,
    pub name: String,
    pub target: RecipeTarget,
    pub steps: Vec<RecipeStep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeStep {
    pub command_id: String,
    pub parameters_json: JsonValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: ObjectId,
    pub name: String,
    pub source: AssetSource,
    pub media_type: Option<String>,
    pub color_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_metadata: Option<ImageAssetMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAssetMetadata {
    pub width: u32,
    pub height: u32,
    pub format: Option<ImageFormat>,
    pub color_type: String,
    pub has_alpha: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectGroup {
    pub id: ObjectId,
    pub name: String,
    pub member_ids: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HistoryState {
    pub entries: Vec<HistoryEntry>,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: ObjectId,
    pub command_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSettings {
    pub default_export_format: OutputFormat,
    pub default_scaling: ScalingMode,
    pub color_profile_policy: ColorProfilePolicy,
    pub autosave: bool,
}

impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            default_export_format: OutputFormat::Png,
            default_scaling: ScalingMode::Lanczos,
            color_profile_policy: ColorProfilePolicy::Preserve,
            autosave: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_degrees: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_degrees: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransparencyGrid {
    pub enabled: bool,
    pub cell_size: u32,
}

impl Default for TransparencyGrid {
    fn default() -> Self {
        Self {
            enabled: true,
            cell_size: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressionSettings {
    pub optimize: bool,
    pub target_bytes: Option<u64>,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            optimize: false,
            target_bytes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanvasBackground {
    Transparent,
    Solid { color: RgbaColor },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClippingBehavior {
    None,
    ClipToLayerBelow,
    ClipToGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportParticipation {
    Included,
    Excluded,
    Inherit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectionKind {
    Rectangular,
    Elliptical,
    Lasso,
    Polygon { points: Vec<Point> },
    MagicWand { tolerance: f32 },
    ColorRange { color: RgbaColor, tolerance: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportBackground {
    Transparent,
    Solid { color: RgbaColor },
    CheckerboardPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrimBehavior {
    None,
    TransparentPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Png,
    Jpeg,
    WebP,
    Avif,
    Gif,
    Bmp,
    Tiff,
    Ico,
    Icns,
    SvgRasterized,
    Pdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Tiff,
    Ico,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransparencyBehavior {
    Preserve,
    Flatten,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataBehavior {
    Preserve,
    Strip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalingMode {
    NearestNeighbor,
    Bilinear,
    Bicubic,
    Lanczos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorProfilePolicy {
    Preserve,
    Remove,
    Convert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeTarget {
    Layer,
    ImageObject,
    Selection,
    ExportArea,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetSource {
    Embedded { digest: Option<String> },
    Linked { path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, thiserror::Error)]
#[error("workspace validation failed with {issue_count} issue(s)", issue_count = .issues.len())]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ValidationIssue {
    #[error("object id cannot be empty")]
    EmptyId,
    #[error("format version {version} is invalid")]
    InvalidFormatVersion { version: u32 },
    #[error("{object_kind} id `{id}` is duplicated")]
    DuplicateId {
        object_kind: &'static str,
        id: ObjectId,
    },
    #[error("{field} on `{owner_id}` references missing id `{referenced_id}`")]
    MissingReference {
        field: &'static str,
        owner_id: ObjectId,
        referenced_id: ObjectId,
    },
    #[error("{object_kind} `{id}` has opacity {opacity}; expected 0.0..=1.0")]
    OpacityOutOfRange {
        object_kind: &'static str,
        id: ObjectId,
        opacity: f32,
    },
    #[error("{object_kind} `{id}` must have positive width and height")]
    NonPositiveBounds {
        object_kind: &'static str,
        id: ObjectId,
    },
    #[error(
        "selection `{selection_id}` mask has {actual_len} alpha value(s); expected {expected_len}"
    )]
    InvalidSelectionMask {
        selection_id: ObjectId,
        expected_len: usize,
        actual_len: usize,
    },
    #[error("layer `{layer_id}` raster has {actual_len} byte(s); expected {expected_len}")]
    InvalidRasterPixels {
        layer_id: ObjectId,
        expected_len: usize,
        actual_len: usize,
    },
    #[error("output `{output_id}` must have a filename")]
    EmptyFilename { output_id: ObjectId },
    #[error("output `{output_id}` must have positive dimensions and scale")]
    InvalidOutputSize { output_id: ObjectId },
}

fn collect_duplicate_ids<'a>(
    issues: &mut Vec<ValidationIssue>,
    object_kind: &'static str,
    ids: impl Iterator<Item = &'a ObjectId>,
) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            issues.push(ValidationIssue::DuplicateId {
                object_kind,
                id: id.clone(),
            });
        }
    }
}

fn require_reference(
    issues: &mut Vec<ValidationIssue>,
    field: &'static str,
    owner_id: &ObjectId,
    referenced_id: &ObjectId,
    valid_ids: &HashSet<&ObjectId>,
) {
    if !valid_ids.contains(referenced_id) {
        issues.push(ValidationIssue::MissingReference {
            field,
            owner_id: owner_id.clone(),
            referenced_id: referenced_id.clone(),
        });
    }
}

mod serde_json_value {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum JsonValue {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<JsonValue>),
        Object(BTreeMap<String, JsonValue>),
    }
}

use crate::APP_NAME;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_workspace_validates() {
        let workspace = Workspace::empty(id("workspace"));

        workspace
            .validate()
            .expect("empty workspace should validate");
        assert_eq!(workspace.format_version, CURRENT_WORKSPACE_FORMAT_VERSION);
    }

    #[test]
    fn workspace_round_trips_through_json() {
        let workspace = populated_workspace();

        let json = serde_json::to_string_pretty(&workspace).expect("serialize workspace");
        let decoded: Workspace = serde_json::from_str(&json).expect("deserialize workspace");

        assert_eq!(decoded, workspace);
        decoded.validate().expect("decoded workspace validates");
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut workspace = Workspace::empty(id("workspace"));
        workspace.layers.push(layer("same"));
        workspace.layers.push(layer("same"));

        let error = workspace
            .validate()
            .expect_err("duplicate layer ids should fail");

        assert!(error.issues.iter().any(|issue| {
            matches!(
                issue,
                ValidationIssue::DuplicateId {
                    object_kind: "layer",
                    ..
                }
            )
        }));
    }

    #[test]
    fn missing_references_are_rejected() {
        let mut workspace = Workspace::empty(id("workspace"));
        workspace.image_objects.push(ImageObject {
            id: id("image"),
            name: "Logo".to_owned(),
            source_asset_id: id("missing-asset"),
            position: Point::ZERO,
            scale: Size {
                width: 1.0,
                height: 1.0,
            },
            rotation_degrees: 0.0,
            opacity: 1.0,
            crop_bounds: None,
            rasterized_layer_id: None,
            export_inclusion: ExportParticipation::Included,
        });

        let error = workspace.validate().expect_err("missing asset should fail");

        assert!(error.issues.iter().any(|issue| {
            matches!(
                issue,
                ValidationIssue::MissingReference {
                    field: "image_object.source_asset_id",
                    ..
                }
            )
        }));
    }

    #[test]
    fn invalid_export_output_settings_are_rejected() {
        let mut workspace = Workspace::empty(id("workspace"));
        workspace.outputs.push(OutputDefinition {
            id: id("output"),
            filename: String::new(),
            folder: None,
            format: OutputFormat::Png,
            width: Some(0),
            height: Some(32),
            scale: 0.0,
            quality: None,
            compression: CompressionSettings::default(),
            background: ExportBackground::Transparent,
            transparency: TransparencyBehavior::Preserve,
            metadata: MetadataBehavior::Strip,
        });

        let error = workspace
            .validate()
            .expect_err("invalid output should fail");

        assert!(error
            .issues
            .iter()
            .any(|issue| matches!(issue, ValidationIssue::EmptyFilename { .. })));
        assert!(error
            .issues
            .iter()
            .any(|issue| matches!(issue, ValidationIssue::InvalidOutputSize { .. })));
    }

    fn populated_workspace() -> Workspace {
        let mut workspace = Workspace::empty(id("workspace"));
        workspace.metadata.name = "Asset Sheet".to_owned();
        workspace.assets.push(Asset {
            id: id("asset-logo"),
            name: "logo.png".to_owned(),
            source: AssetSource::Embedded {
                digest: Some("sha256:example".to_owned()),
            },
            media_type: Some("image/png".to_owned()),
            color_profile: Some("sRGB".to_owned()),
            image_metadata: None,
        });
        workspace.layers.push(layer("layer-logo"));
        workspace.image_objects.push(ImageObject {
            id: id("image-logo"),
            name: "Placed Logo".to_owned(),
            source_asset_id: id("asset-logo"),
            position: Point { x: 32.0, y: 48.0 },
            scale: Size {
                width: 1.0,
                height: 1.0,
            },
            rotation_degrees: 0.0,
            opacity: 1.0,
            crop_bounds: None,
            rasterized_layer_id: Some(id("layer-logo")),
            export_inclusion: ExportParticipation::Included,
        });
        workspace.selections.push(Selection {
            id: id("selection-main"),
            kind: SelectionKind::Rectangular,
            bounds: rect(0.0, 0.0, 64.0, 64.0),
            feather_radius: 0.0,
            source_layer_ids: vec![id("layer-logo")],
            mask: Some(SelectionMask {
                width: 64,
                height: 64,
                alpha: vec![255; 64 * 64],
            }),
        });
        workspace.guides.push(Guide {
            id: id("guide-center"),
            axis: Axis::Vertical,
            position: 32.0,
            locked: false,
        });
        workspace.outputs.push(OutputDefinition {
            id: id("output-png"),
            filename: "logo.png".to_owned(),
            folder: Some("dist".to_owned()),
            format: OutputFormat::Png,
            width: Some(128),
            height: Some(128),
            scale: 2.0,
            quality: None,
            compression: CompressionSettings {
                optimize: true,
                target_bytes: None,
            },
            background: ExportBackground::Transparent,
            transparency: TransparencyBehavior::Preserve,
            metadata: MetadataBehavior::Strip,
        });
        workspace.export_areas.push(ExportArea {
            id: id("export-logo"),
            name: "logo".to_owned(),
            bounds: rect(0.0, 0.0, 64.0, 64.0),
            padding: Padding::default(),
            background: ExportBackground::Transparent,
            trim: TrimBehavior::None,
            output_ids: vec![id("output-png")],
            included_layer_ids: vec![id("layer-logo")],
            excluded_layer_ids: Vec::new(),
            tags: vec!["brand".to_owned()],
            preset_id: None,
        });
        workspace.recipes.push(Recipe {
            id: id("recipe-favicon"),
            name: "Favicon".to_owned(),
            target: RecipeTarget::ExportArea,
            steps: vec![RecipeStep {
                command_id: "export.area".to_owned(),
                parameters_json: JsonValue::Object(Default::default()),
            }],
        });
        workspace.object_groups.push(ObjectGroup {
            id: id("group-brand"),
            name: "Brand".to_owned(),
            member_ids: vec![id("layer-logo"), id("export-logo")],
        });
        workspace.history.entries.push(HistoryEntry {
            id: id("history-1"),
            command_id: "workspace.create".to_owned(),
            label: "Create workspace".to_owned(),
        });
        workspace.history.current_index = Some(0);
        workspace
    }

    fn layer(value: &str) -> Layer {
        Layer {
            id: id(value),
            name: value.to_owned(),
            visible: true,
            opacity: 1.0,
            locked: false,
            position: Point::ZERO,
            bounds: rect(0.0, 0.0, 64.0, 64.0),
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
