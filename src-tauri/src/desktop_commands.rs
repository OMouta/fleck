use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose, Engine as _};
use fleck_core::command::{
    default_command_registry, CommandEngine, CommandId, CommandInvocation, CommandParameters,
};
use fleck_core::export::{preview_export_area, ExportWarning, OutputScale};
use fleck_core::image_import;
use fleck_core::model::{
    AssetSource, ExportArea, ExportBackground, HistoryState, ImageObject, JsonValue, Layer,
    ObjectId, OutputFormat, Padding, Rect, SelectionKind, TransparencyBehavior, Workspace,
};
use fleck_core::persistence::{
    load_package_from_path, save_package_to_path, LoadWarning, WorkspacePackage,
};
use fleck_render::{DefaultExportOptions, EncodedExport, SkiaViewportRenderer};
use image::{ImageBuffer, Rgba};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct DesktopState {
    inner: Mutex<DocumentState>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(DocumentState::new_empty()),
        }
    }
}

#[cfg(test)]
pub const REGISTERED_TAURI_COMMANDS: &[&str] = &[
    "ownership_boundaries",
    "get_workspace_meta",
    "get_layers",
    "get_image_objects",
    "get_export_areas",
    "get_history",
    "get_commands",
    "new_workspace",
    "open_workspace",
    "open_workspace_path",
    "save_workspace",
    "save_workspace_as",
    "get_recent_files",
    "pick_image_file",
    "acquire_clipboard_asset",
    "acquire_dropped_asset",
    "acquire_replacement_asset",
    "reveal_image_source",
    "relink_asset",
    "get_render_model",
    "get_viewport_focus",
    "create_export_area",
    "export_area",
    "export_all",
    "reveal_exported_file",
    "copy_export_result",
    "run_command",
    "undo",
    "redo",
    "jump_to_history",
    "supports_history_jump",
];

struct DocumentState {
    package: WorkspacePackage,
    path: Option<PathBuf>,
    engine: CommandEngine,
    dirty: bool,
    has_document: bool,
    /// Cache of the most recent export job's encoded outputs, indexed by the
    /// `filename` we use as the output id in the frontend `ExportResult`.
    /// Used by `copy_export_result` so we don't re-run the pipeline to copy.
    last_export: Vec<EncodedExport>,
}

impl DocumentState {
    fn new_empty() -> Self {
        Self {
            package: WorkspacePackage::new(Workspace::empty(generated_id("workspace"))),
            path: None,
            engine: CommandEngine::new(),
            dirty: false,
            has_document: false,
            last_export: Vec::new(),
        }
    }

    fn new_workspace() -> Self {
        Self {
            has_document: true,
            ..Self::new_empty()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMetaDto {
    name: String,
    dirty: bool,
    layer_count: usize,
    selected_count: usize,
    canvas_size: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerDto {
    id: String,
    name: String,
    kind: String,
    visible: bool,
    locked: bool,
    opacity: u8,
    blend: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObjectDto {
    id: String,
    name: String,
    source_asset_id: String,
    source_state: String,
    source_name: String,
    source_path: Option<String>,
    format: Option<String>,
    dimensions: Option<String>,
    position: PointDto,
    scale: SizeDto,
    rotation_degrees: f32,
    opacity: u8,
    crop: Option<RectDto>,
    rasterized_layer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAreaDto {
    id: String,
    name: String,
    bounds: RectDto,
    dimensions: String,
    position: String,
    padding_px: PaddingDto,
    padding: String,
    background_param: String,
    background: String,
    format: String,
    status: String,
    warnings: Vec<String>,
    outputs: Vec<OutputDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputDto {
    id: String,
    filename: String,
    format: String,
    scale: String,
    quality: Option<u8>,
    transparency: String,
    destination: Option<String>,
    dimensions: String,
    estimated_size: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResultDto {
    scope: String,
    outputs: Vec<ExportResultOutputDto>,
    warnings: Vec<String>,
    failures: Vec<ExportFailureDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResultOutputDto {
    id: String,
    filename: String,
    destination: Option<String>,
    format: String,
    dimensions: String,
    size: String,
    data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFailureDto {
    filename: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStateDto {
    entries: Vec<HistoryEntryDto>,
    current_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryDto {
    id: String,
    command_id: String,
    label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandDefinitionDto {
    id: String,
    label: String,
    description: String,
    group: String,
    aliases: Vec<String>,
    shortcut: Option<String>,
    undoable: bool,
    parameter_prompts: Vec<ParameterPromptDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterPromptDto {
    key: String,
    label: String,
    kind: String,
    required: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecutionDto {
    command_id: String,
    operation_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWorkspaceResultDto {
    path: String,
    name: String,
    warnings: Vec<LoadWarningDto>,
    missing_assets: Vec<MissingAssetDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum LoadWarningDto {
    Migrated { from: u32, to: u32 },
    NewerFile { found: u32, supported: u32 },
    NewerWorkspace { found: u32, supported: u32 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingAssetDto {
    asset_id: String,
    name: String,
    path: String,
    resolved_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFileDto {
    path: String,
    name: String,
    opened_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderModelDto {
    has_document: bool,
    canvas: CanvasDto,
    layers: Vec<RenderLayerDto>,
    export_areas: Vec<RenderExportAreaDto>,
    guides: Vec<RenderGuideDto>,
    selections: Vec<RenderSelectionDto>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PointDto {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SizeDto {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RectDto {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PaddingDto {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CanvasDto {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLayerDto {
    id: String,
    rect: RectDto,
    color: String,
    opacity: f32,
    visible: bool,
    image_src: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderExportAreaDto {
    id: String,
    name: String,
    rect: RectDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderGuideDto {
    axis: String,
    position: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderSelectionDto {
    id: String,
    rect: RectDto,
    /// Snake-case discriminator matching `SelectionKind` (e.g. "rectangular",
    /// "elliptical", "lasso", "polygon", "magic_wand", "color_range").
    kind: String,
    /// Path vertices in workspace coordinates, sent for lasso/polygon so the
    /// frontend can draw the actual shape outline instead of falling back to the
    /// bounding rectangle.
    #[serde(skip_serializing_if = "Option::is_none")]
    points: Option<Vec<PointDto>>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct SizeDtoInput {
    width: f32,
    height: f32,
}

#[tauri::command]
pub fn get_workspace_meta(
    state: tauri::State<'_, DesktopState>,
) -> Result<WorkspaceMetaDto, String> {
    with_document(&state, |document| {
        Ok(workspace_meta(
            &document.package.workspace,
            document.path.as_deref(),
            document.dirty,
        ))
    })
}

#[tauri::command]
pub fn get_layers(state: tauri::State<'_, DesktopState>) -> Result<Vec<LayerDto>, String> {
    with_document(&state, |document| {
        Ok(document
            .package
            .workspace
            .layers
            .iter()
            .map(layer_dto)
            .collect())
    })
}

#[tauri::command]
pub fn get_image_objects(
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<ImageObjectDto>, String> {
    with_document(&state, |document| {
        Ok(document
            .package
            .workspace
            .image_objects
            .iter()
            .map(|object| image_object_dto(&document.package.workspace, object))
            .collect())
    })
}

#[tauri::command]
pub fn get_export_areas(
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<ExportAreaDto>, String> {
    with_document(&state, |document| {
        Ok(document
            .package
            .workspace
            .export_areas
            .iter()
            .map(|area| export_area_dto(&document.package.workspace, area))
            .collect())
    })
}

#[tauri::command]
pub fn get_history(state: tauri::State<'_, DesktopState>) -> Result<HistoryStateDto, String> {
    with_document(&state, |document| {
        Ok(history_dto(&document.package.workspace.history))
    })
}

#[tauri::command]
pub fn get_commands() -> Result<Vec<CommandDefinitionDto>, String> {
    let registry = default_command_registry().map_err(|error| error.to_string())?;
    Ok(registry.definitions().map(command_definition_dto).collect())
}

#[tauri::command]
pub fn new_workspace(state: tauri::State<'_, DesktopState>) -> Result<(), String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    *document = DocumentState::new_workspace();
    Ok(())
}

#[tauri::command]
pub fn open_workspace(
    state: tauri::State<'_, DesktopState>,
) -> Result<Option<OpenWorkspaceResultDto>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Fleck workspace", &["fleck"])
        .pick_file()
    else {
        return Ok(None);
    };
    open_workspace_path_inner(&state, path).map(Some)
}

#[tauri::command]
pub fn open_workspace_path(
    state: tauri::State<'_, DesktopState>,
    path: String,
) -> Result<Option<OpenWorkspaceResultDto>, String> {
    open_workspace_path_inner(&state, PathBuf::from(path)).map(Some)
}

#[tauri::command]
pub fn save_workspace(state: tauri::State<'_, DesktopState>) -> Result<(), String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let path = match &document.path {
        Some(path) => path.clone(),
        None => {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Fleck workspace", &["fleck"])
                .set_file_name("Untitled.fleck")
                .save_file()
            else {
                return Ok(());
            };
            path
        }
    };
    save_package_to_path(&document.package, &path).map_err(|error| error.to_string())?;
    document.path = Some(path.clone());
    document.dirty = false;
    drop(document);
    push_recent_file(&path);
    Ok(())
}

#[tauri::command]
pub fn save_workspace_as(state: tauri::State<'_, DesktopState>) -> Result<Option<String>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Fleck workspace", &["fleck"])
        .set_file_name("Untitled.fleck")
        .save_file()
    else {
        return Ok(None);
    };
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    save_package_to_path(&document.package, &path).map_err(|error| error.to_string())?;
    document.path = Some(path.clone());
    document.dirty = false;
    drop(document);
    push_recent_file(&path);
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn get_recent_files(
    _state: tauri::State<'_, DesktopState>,
) -> Result<Vec<RecentFileDto>, String> {
    Ok(read_recent_files()
        .into_iter()
        .filter(|entry| Path::new(&entry.path).exists())
        .map(|entry| RecentFileDto {
            name: file_name(Path::new(&entry.path)),
            path: entry.path,
            opened_at: relative_time(entry.opened_at_secs),
        })
        .collect())
}

#[tauri::command]
pub fn pick_image_file() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .add_filter(
            "Images",
            &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "ico"],
        )
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned()))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquiredAssetDto {
    asset_id: String,
    name: String,
}

#[tauri::command]
pub fn acquire_clipboard_asset(
    state: tauri::State<'_, DesktopState>,
) -> Result<Option<AcquiredAssetDto>, String> {
    let image = match Clipboard::new()
        .map_err(|error| format!("clipboard unavailable: {error}"))?
        .get_image()
    {
        Ok(image) => image,
        Err(arboard::Error::ContentNotAvailable) => return Ok(None),
        Err(error) => return Err(format!("clipboard read failed: {error}")),
    };
    let bytes = encode_rgba_to_png(&image)?;
    let name = format!("pasted-{}.png", short_timestamp());
    Ok(Some(register_acquired_asset(&state, &name, bytes)?))
}

#[tauri::command]
pub fn acquire_dropped_asset(
    state: tauri::State<'_, DesktopState>,
    name: String,
    bytes: Vec<u8>,
) -> Result<Option<AcquiredAssetDto>, String> {
    if bytes.is_empty() {
        return Ok(None);
    }
    Ok(Some(register_acquired_asset(&state, &name, bytes)?))
}

#[tauri::command]
pub fn acquire_replacement_asset(
    state: tauri::State<'_, DesktopState>,
) -> Result<Option<String>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(
            "Images",
            &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "ico"],
        )
        .pick_file()
    else {
        return Ok(None);
    };
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let name = file_name(&path);
    let asset = register_acquired_asset(&state, &name, bytes)?;
    Ok(Some(asset.asset_id))
}

#[tauri::command(rename_all = "camelCase")]
pub fn reveal_image_source(
    state: tauri::State<'_, DesktopState>,
    object_id: String,
) -> Result<(), String> {
    let document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let id = ObjectId::new(object_id).map_err(|error| error.to_string())?;
    let object = document
        .package
        .workspace
        .image_objects
        .iter()
        .find(|object| object.id == id)
        .ok_or_else(|| format!("image object `{}` was not found", id))?;
    let asset = document
        .package
        .workspace
        .assets
        .iter()
        .find(|asset| asset.id == object.source_asset_id)
        .ok_or_else(|| "image object has no resolved asset".to_owned())?;
    let path = match &asset.source {
        AssetSource::Linked { path } => PathBuf::from(path),
        AssetSource::Embedded { .. } => {
            return Err("embedded assets have no on-disk source to reveal".to_owned())
        }
    };
    drop(document);
    tauri_plugin_opener::reveal_item_in_dir(&path).map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn relink_asset(state: tauri::State<'_, DesktopState>, asset_id: String) -> Result<(), String> {
    let Some(new_path) = rfd::FileDialog::new()
        .add_filter(
            "Images",
            &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "ico"],
        )
        .pick_file()
    else {
        return Ok(());
    };
    let id = ObjectId::new(asset_id).map_err(|error| error.to_string())?;
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let asset = document
        .package
        .workspace
        .assets
        .iter_mut()
        .find(|asset| asset.id == id)
        .ok_or_else(|| format!("asset `{}` was not found", id))?;
    asset.source = AssetSource::Linked {
        path: new_path.to_string_lossy().into_owned(),
    };
    document.dirty = true;
    Ok(())
}

#[tauri::command]
pub fn get_render_model(state: tauri::State<'_, DesktopState>) -> Result<RenderModelDto, String> {
    with_document(&state, |document| {
        Ok(render_model(&document.package, document.has_document))
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_viewport_focus(
    state: tauri::State<'_, DesktopState>,
    kind: String,
    screen: SizeDtoInput,
) -> Result<Option<serde_json::Value>, String> {
    with_document(&state, |document| {
        let workspace = &document.package.workspace;
        let rect = match kind.as_str() {
            "selection" => workspace
                .selections
                .first()
                .map(|selection| selection.bounds),
            "export-area" => workspace.export_areas.first().map(|area| area.bounds),
            _ => Some(default_canvas_rect(workspace)),
        };
        Ok(rect.map(|rect| {
            let fitted = fit_rect(rect, screen);
            serde_json::json!({
                "origin": { "x": fitted.0.x, "y": fitted.0.y },
                "zoom": fitted.1
            })
        }))
    })
}

#[tauri::command]
pub fn create_export_area(state: tauri::State<'_, DesktopState>) -> Result<(), String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let workspace = &mut document.package.workspace;
    let id = generated_id("export-area");
    let output_id = generated_id("output");
    workspace.outputs.push(fleck_core::model::OutputDefinition {
        id: output_id.clone(),
        filename: "export.png".to_owned(),
        folder: None,
        format: OutputFormat::Png,
        width: None,
        height: None,
        scale: 1.0,
        quality: None,
        compression: Default::default(),
        background: ExportBackground::Transparent,
        transparency: fleck_core::model::TransparencyBehavior::Preserve,
        metadata: fleck_core::model::MetadataBehavior::Strip,
    });
    workspace.export_areas.push(ExportArea {
        id,
        name: "export".to_owned(),
        bounds: default_canvas_rect(workspace),
        padding: Default::default(),
        background: ExportBackground::Transparent,
        trim: fleck_core::model::TrimBehavior::None,
        output_ids: vec![output_id],
        included_layer_ids: Vec::new(),
        excluded_layer_ids: Vec::new(),
        tags: Vec::new(),
        preset_id: None,
    });
    document.dirty = true;
    Ok(())
}

#[tauri::command]
pub fn export_area(
    state: tauri::State<'_, DesktopState>,
    id: String,
) -> Result<ExportResultDto, String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let workspace = &document.package.workspace;
    let id = ObjectId::new(id).map_err(|error| error.to_string())?;
    let scope = workspace
        .export_areas
        .iter()
        .find(|area| area.id == id)
        .map(|area| area.name.clone())
        .unwrap_or_else(|| "Export area".to_owned());
    let warnings = preview_export_area(workspace, &id)
        .map(|preview| preview.warnings.iter().map(export_warning_label).collect())
        .unwrap_or_default();
    match SkiaViewportRenderer::new().export_area(workspace, &id) {
        Ok(encoded) => {
            document.last_export = encoded.clone();
            Ok(export_result_dto(scope, encoded, warnings))
        }
        // An area with no outputs is a no-op job, not an error — report it.
        Err(fleck_render::ExportPipelineError::AreaHasNoOutputs { .. }) => {
            Ok(export_result_dto(scope, Vec::new(), warnings))
        }
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
pub fn export_all(state: tauri::State<'_, DesktopState>) -> Result<ExportResultDto, String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    match SkiaViewportRenderer::new().export_all(
        &document.package.workspace,
        &DefaultExportOptions::default(),
    ) {
        Ok(encoded) => {
            document.last_export = encoded.clone();
            Ok(export_result_dto(
                "All areas".to_owned(),
                encoded,
                Vec::new(),
            ))
        }
        Err(fleck_render::ExportPipelineError::NoExportableContent) => Ok(export_result_dto(
            "All areas".to_owned(),
            Vec::new(),
            vec!["No visible exportable content".to_owned()],
        )),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn reveal_exported_file(destination: String) -> Result<(), String> {
    tauri_plugin_opener::reveal_item_in_dir(PathBuf::from(destination))
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn copy_export_result(
    state: tauri::State<'_, DesktopState>,
    output_id: String,
    mode: String,
) -> Result<(), String> {
    // Look up the last-export cache. We use the `filename` field as the id the
    // frontend received in `ExportResult.outputs[i].id`.
    let document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let export = document
        .last_export
        .iter()
        .find(|export| export.filename == output_id)
        .cloned()
        .ok_or_else(|| {
            "no cached export for this output \u{2014} run the export again first".to_owned()
        })?;
    drop(document);

    let mut clipboard =
        Clipboard::new().map_err(|error| format!("clipboard unavailable: {error}"))?;
    match mode.as_str() {
        "image" => {
            let decoded = image::load_from_memory(&export.bytes)
                .map_err(|error| format!("decode export for clipboard: {error}"))?
                .to_rgba8();
            let (width, height) = decoded.dimensions();
            clipboard
                .set_image(ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: Cow::Owned(decoded.into_raw()),
                })
                .map_err(|error| format!("clipboard write failed: {error}"))?;
        }
        "base64" => {
            let encoded = general_purpose::STANDARD.encode(&export.bytes);
            clipboard
                .set_text(encoded)
                .map_err(|error| format!("clipboard write failed: {error}"))?;
        }
        "markdown" => {
            let media = match export.format {
                OutputFormat::Jpeg => "image/jpeg",
                OutputFormat::WebP => "image/webp",
                OutputFormat::Gif => "image/gif",
                OutputFormat::Bmp => "image/bmp",
                _ => "image/png",
            };
            let encoded = general_purpose::STANDARD.encode(&export.bytes);
            let markdown = format!("![{}](data:{};base64,{})", export.filename, media, encoded);
            clipboard
                .set_text(markdown)
                .map_err(|error| format!("clipboard write failed: {error}"))?;
        }
        other => return Err(format!("unknown copy mode `{other}`")),
    }
    Ok(())
}

fn export_result_dto(
    scope: String,
    encoded: Vec<fleck_render::EncodedExport>,
    warnings: Vec<String>,
) -> ExportResultDto {
    ExportResultDto {
        scope,
        outputs: encoded
            .into_iter()
            .map(|export| ExportResultOutputDto {
                id: export.filename.clone(),
                filename: export.filename,
                destination: export.destination,
                format: output_format_label(export.format).to_owned(),
                dimensions: format!("{} × {} px", export.width, export.height),
                size: human_bytes(export.bytes.len() as u64),
                data_url: None,
            })
            .collect(),
        warnings,
        failures: Vec::new(),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn run_command(
    state: tauri::State<'_, DesktopState>,
    command_id: String,
    parameters: BTreeMap<String, serde_json::Value>,
) -> Result<CommandExecutionDto, String> {
    let registry = default_command_registry().map_err(|error| error.to_string())?;
    let invocation = CommandInvocation {
        id: CommandId::new(command_id).map_err(|error| error.to_string())?,
        parameters: CommandParameters::new(
            parameters
                .into_iter()
                .map(|(key, value)| (key, json_value(value))),
        ),
        context: Default::default(),
    };
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let mut workspace = std::mem::replace(
        &mut document.package.workspace,
        Workspace::empty(generated_id("workspace-temp")),
    );
    let execution = document
        .engine
        .execute(
            &mut workspace,
            &registry,
            invocation,
            &fleck_core::command::CommandRuntime::default(),
        )
        .map_err(|error| error.to_string());
    document.package.workspace = workspace;
    if execution.is_ok() {
        document.has_document = true;
        document.dirty = true;
    }
    execution.map(|execution| CommandExecutionDto {
        command_id: execution.command_id.as_str().to_owned(),
        operation_label: execution.operation_label,
    })
}

#[tauri::command]
pub fn undo(state: tauri::State<'_, DesktopState>) -> Result<Option<CommandExecutionDto>, String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let mut workspace = std::mem::replace(
        &mut document.package.workspace,
        Workspace::empty(generated_id("workspace-temp")),
    );
    let execution = match document.engine.undo(&mut workspace) {
        Ok(execution) => Ok(Some(CommandExecutionDto {
            command_id: execution.command_id.as_str().to_owned(),
            operation_label: execution.operation_label,
        })),
        Err(fleck_core::command::CommandError::NothingToUndo) => Ok(None),
        Err(error) => Err(error.to_string()),
    };
    document.package.workspace = workspace;
    if matches!(execution, Ok(Some(_))) {
        document.dirty = true;
    }
    execution
}

#[tauri::command]
pub fn redo(state: tauri::State<'_, DesktopState>) -> Result<Option<CommandExecutionDto>, String> {
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    let mut workspace = std::mem::replace(
        &mut document.package.workspace,
        Workspace::empty(generated_id("workspace-temp")),
    );
    let execution = match document.engine.redo(&mut workspace) {
        Ok(execution) => Ok(Some(CommandExecutionDto {
            command_id: execution.command_id.as_str().to_owned(),
            operation_label: execution.operation_label,
        })),
        Err(fleck_core::command::CommandError::NothingToRedo) => Ok(None),
        Err(error) => Err(error.to_string()),
    };
    document.package.workspace = workspace;
    if matches!(execution, Ok(Some(_))) {
        document.dirty = true;
    }
    execution
}

#[tauri::command]
pub fn jump_to_history(_state: tauri::State<'_, DesktopState>, _index: i32) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn supports_history_jump() -> Result<bool, String> {
    Ok(false)
}

fn open_workspace_path_inner(
    state: &tauri::State<'_, DesktopState>,
    path: PathBuf,
) -> Result<OpenWorkspaceResultDto, String> {
    let outcome = load_package_from_path(&path).map_err(|error| error.to_string())?;
    let workspace_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let missing_assets = outcome
        .package
        .missing_linked_assets(workspace_dir)
        .into_iter()
        .map(|asset| MissingAssetDto {
            asset_id: asset.asset_id.as_str().to_owned(),
            name: asset.name,
            path: asset.path.to_string_lossy().into_owned(),
            resolved_path: asset.resolved_path.to_string_lossy().into_owned(),
        })
        .collect();
    let warnings = outcome.warnings.into_iter().map(load_warning_dto).collect();
    let name = file_name(&path);
    let result = OpenWorkspaceResultDto {
        path: path.to_string_lossy().into_owned(),
        name,
        warnings,
        missing_assets,
    };
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    document.package = outcome.package;
    document.path = Some(path.clone());
    document.engine = CommandEngine::new();
    document.dirty = false;
    document.has_document = true;
    drop(document);
    push_recent_file(&path);
    Ok(result)
}

fn with_document<T>(
    state: &tauri::State<'_, DesktopState>,
    f: impl FnOnce(&DocumentState) -> Result<T, String>,
) -> Result<T, String> {
    let document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    f(&document)
}

fn workspace_meta(workspace: &Workspace, path: Option<&Path>, dirty: bool) -> WorkspaceMetaDto {
    let bounds = content_canvas_rect(workspace).unwrap_or(Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    });
    WorkspaceMetaDto {
        name: path
            .map(file_name)
            .unwrap_or_else(|| workspace.metadata.name.clone()),
        dirty,
        layer_count: workspace.layers.len(),
        selected_count: workspace.selections.len(),
        canvas_size: format!("{} × {} px", bounds.width.round(), bounds.height.round()),
    }
}

fn layer_dto(layer: &Layer) -> LayerDto {
    LayerDto {
        id: layer.id.as_str().to_owned(),
        name: layer.name.clone(),
        kind: "image".to_owned(),
        visible: layer.visible,
        locked: layer.locked,
        opacity: (layer.opacity.clamp(0.0, 1.0) * 100.0).round() as u8,
        blend: format!("{:?}", layer.blend_mode),
    }
}

fn image_object_dto(workspace: &Workspace, object: &ImageObject) -> ImageObjectDto {
    let asset = workspace
        .assets
        .iter()
        .find(|asset| asset.id == object.source_asset_id);
    let (source_state, source_path) = match asset.map(|asset| &asset.source) {
        Some(AssetSource::Linked { path }) => ("linked".to_owned(), Some(path.clone())),
        Some(AssetSource::Embedded { .. }) => ("embedded".to_owned(), None),
        None => ("missing".to_owned(), None),
    };
    ImageObjectDto {
        id: object.id.as_str().to_owned(),
        name: object.name.clone(),
        source_asset_id: object.source_asset_id.as_str().to_owned(),
        source_state,
        source_name: asset
            .map(|asset| asset.name.clone())
            .unwrap_or_else(|| "(missing asset)".to_owned()),
        source_path,
        format: asset.and_then(|asset| asset.media_type.clone()),
        dimensions: asset
            .and_then(|asset| asset.image_metadata.as_ref())
            .map(|metadata| format!("{} × {} px", metadata.width, metadata.height)),
        position: PointDto {
            x: object.position.x,
            y: object.position.y,
        },
        scale: SizeDto {
            width: object.scale.width,
            height: object.scale.height,
        },
        rotation_degrees: object.rotation_degrees,
        opacity: (object.opacity.clamp(0.0, 1.0) * 100.0).round() as u8,
        crop: object.crop_bounds.map(rect_dto),
        rasterized_layer_id: object
            .rasterized_layer_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
    }
}

fn export_area_dto(workspace: &Workspace, area: &ExportArea) -> ExportAreaDto {
    // Source warnings + per-output preview dimensions from core preview metadata
    // so the UI consumes the same numbers/warnings the export pipeline would.
    let preview = preview_export_area(workspace, &area.id).ok();
    let definition_of = |output_id: &str| {
        workspace
            .outputs
            .iter()
            .find(|output| output.id.as_str() == output_id)
    };
    let outputs = preview
        .as_ref()
        .map(|preview| {
            preview
                .outputs
                .iter()
                .map(|output| {
                    let definition = definition_of(output.output_id.as_str());
                    let quality = definition.and_then(|definition| definition.quality);
                    let transparency = definition
                        .map(|definition| transparency_label(definition.transparency))
                        .unwrap_or("Preserve")
                        .to_owned();
                    OutputDto {
                        id: output.output_id.as_str().to_owned(),
                        filename: output.filename.clone(),
                        format: output_format_label(output.format).to_owned(),
                        scale: scale_label(output.scale),
                        quality,
                        transparency,
                        destination: output.destination.clone(),
                        dimensions: format!("{} × {} px", output.pixel_width, output.pixel_height),
                        estimated_size: format!(
                            "~{}",
                            human_bytes(estimate_bytes(
                                output.pixel_width,
                                output.pixel_height,
                                output.format,
                                quality,
                            ))
                        ),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let warnings = preview
        .as_ref()
        .map(|preview| {
            preview
                .warnings
                .iter()
                .map(export_warning_label)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ExportAreaDto {
        id: area.id.as_str().to_owned(),
        name: area.name.clone(),
        bounds: rect_dto(area.bounds),
        dimensions: format!(
            "{} × {} px",
            area.bounds.width.round(),
            area.bounds.height.round()
        ),
        position: format!("{}, {}", area.bounds.x.round(), area.bounds.y.round()),
        padding_px: padding_dto(area.padding),
        padding: padding_label(&area.padding),
        background_param: background_param(&area.background),
        background: background_label(&area.background),
        format: outputs
            .first()
            .map(|output| output.format.clone())
            .unwrap_or_else(|| "—".to_owned()),
        status: if warnings.is_empty() {
            "ready"
        } else {
            "warning"
        }
        .to_owned(),
        warnings,
        outputs,
    }
}

fn output_format_label(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Png => "PNG",
        OutputFormat::Jpeg => "JPEG",
        OutputFormat::WebP => "WebP",
        OutputFormat::Avif => "AVIF",
        OutputFormat::Gif => "GIF",
        OutputFormat::Bmp => "BMP",
        OutputFormat::Tiff => "TIFF",
        OutputFormat::Ico => "ICO",
        OutputFormat::Icns => "ICNS",
        OutputFormat::SvgRasterized => "SVG",
        OutputFormat::Pdf => "PDF",
    }
}

fn scale_label(scale: OutputScale) -> String {
    let value = scale.numerator as f32 / scale.denominator as f32;
    format!("{}×", (value * 1000.0).round() / 1000.0)
}

fn transparency_label(transparency: TransparencyBehavior) -> &'static str {
    match transparency {
        TransparencyBehavior::Preserve => "Preserve",
        TransparencyBehavior::Flatten => "Flatten",
    }
}

/// Rough encoded-size estimate for the export preview (REQ-032). Mirrors the
/// frontend heuristic; the real exact size is reported after encoding.
fn estimate_bytes(width: u32, height: u32, format: OutputFormat, quality: Option<u8>) -> u64 {
    let pixels = width as f64 * height as f64;
    let factor = match format {
        OutputFormat::Jpeg => 0.42 * (quality.unwrap_or(80) as f64 / 100.0),
        OutputFormat::WebP => 0.32 * (quality.unwrap_or(80) as f64 / 100.0),
        OutputFormat::Avif => 0.22 * (quality.unwrap_or(75) as f64 / 100.0),
        OutputFormat::Gif => 0.8,
        OutputFormat::Png => 1.9,
        _ => 1.4,
    };
    (pixels * factor).round() as u64
}

fn human_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn padding_label(padding: &Padding) -> String {
    let (t, r, b, l) = (padding.top, padding.right, padding.bottom, padding.left);
    if t == 0.0 && r == 0.0 && b == 0.0 && l == 0.0 {
        "None".to_owned()
    } else if t == r && r == b && b == l {
        format!("{} px", round1(t))
    } else {
        format!(
            "T{} R{} B{} L{}",
            round1(t),
            round1(r),
            round1(b),
            round1(l)
        )
    }
}

fn background_label(background: &ExportBackground) -> String {
    match background {
        ExportBackground::Transparent => "Transparent".to_owned(),
        ExportBackground::Solid { color } => {
            format!("Solid #{:02x}{:02x}{:02x}", color.r, color.g, color.b)
        }
        ExportBackground::CheckerboardPreview => "Checkerboard".to_owned(),
    }
}

fn background_param(background: &ExportBackground) -> String {
    match background {
        ExportBackground::Transparent => "transparent".to_owned(),
        ExportBackground::Solid { color } if color.r == 255 && color.g == 255 && color.b == 255 => {
            "white".to_owned()
        }
        ExportBackground::Solid { color } if color.r == 0 && color.g == 0 && color.b == 0 => {
            "black".to_owned()
        }
        ExportBackground::Solid { color } => {
            format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
        }
        ExportBackground::CheckerboardPreview => "checkerboard_preview".to_owned(),
    }
}

fn round1(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

fn export_warning_label(warning: &ExportWarning) -> String {
    match warning {
        ExportWarning::NoOutputs => "No outputs configured".to_owned(),
        ExportWarning::NoParticipatingLayers => "No layers participate in this export".to_owned(),
        ExportWarning::LayerIncludedAndExcluded { .. } => {
            "A layer is both included and excluded".to_owned()
        }
        ExportWarning::JpegCannotPreserveTransparency { .. } => {
            "JPEG cannot preserve transparency".to_owned()
        }
        ExportWarning::CheckerboardPreviewBackground { .. } => {
            "Checkerboard is a preview-only background".to_owned()
        }
    }
}

fn history_dto(history: &HistoryState) -> HistoryStateDto {
    HistoryStateDto {
        entries: history
            .entries
            .iter()
            .map(|entry| HistoryEntryDto {
                id: entry.id.as_str().to_owned(),
                command_id: entry.command_id.clone(),
                label: entry.label.clone(),
            })
            .collect(),
        current_index: history.current_index,
    }
}

fn command_definition_dto(
    definition: &fleck_core::command::CommandDefinition,
) -> CommandDefinitionDto {
    CommandDefinitionDto {
        id: definition.id.as_str().to_owned(),
        label: definition.label.clone(),
        description: definition.description.clone(),
        group: format!("{:?}", definition.group).to_case_snake(),
        aliases: definition.aliases.clone(),
        shortcut: definition.shortcut.clone(),
        undoable: definition.undoable,
        parameter_prompts: definition
            .parameter_prompts
            .iter()
            .map(|prompt| ParameterPromptDto {
                key: prompt.key.clone(),
                label: prompt.label.clone(),
                kind: format!("{:?}", prompt.kind).to_case_snake(),
                required: prompt.required,
            })
            .collect(),
    }
}

fn render_model(package: &WorkspacePackage, has_document: bool) -> RenderModelDto {
    let workspace = &package.workspace;
    let bounds = content_canvas_rect(workspace).unwrap_or(Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    });
    RenderModelDto {
        has_document,
        canvas: CanvasDto {
            width: bounds.width,
            height: bounds.height,
        },
        layers: workspace
            .layers
            .iter()
            .enumerate()
            .map(|(index, layer)| RenderLayerDto {
                id: layer.id.as_str().to_owned(),
                rect: rect_dto(layer_workspace_rect(layer)),
                color: render_color(index),
                opacity: layer.opacity,
                visible: layer.visible,
                image_src: None,
            })
            .chain(
                workspace
                    .image_objects
                    .iter()
                    .enumerate()
                    .map(|(index, object)| RenderLayerDto {
                        id: object.id.as_str().to_owned(),
                        rect: rect_dto(image_object_rect(object)),
                        color: render_color(index + workspace.layers.len()),
                        opacity: object.opacity,
                        visible: true,
                        image_src: image_object_source(package, object),
                    }),
            )
            .collect(),
        export_areas: workspace
            .export_areas
            .iter()
            .map(|area| RenderExportAreaDto {
                id: area.id.as_str().to_owned(),
                name: area.name.clone(),
                rect: rect_dto(area.bounds),
            })
            .collect(),
        guides: workspace
            .guides
            .iter()
            .map(|guide| RenderGuideDto {
                axis: format!("{:?}", guide.axis).to_lowercase(),
                position: guide.position,
            })
            .collect(),
        selections: workspace
            .selections
            .iter()
            .map(|selection| {
                let (kind, points) = match &selection.kind {
                    SelectionKind::Rectangular => ("rectangular", None),
                    SelectionKind::Elliptical => ("elliptical", None),
                    SelectionKind::Lasso { points } => (
                        "lasso",
                        Some(points.iter().map(|p| PointDto { x: p.x, y: p.y }).collect()),
                    ),
                    SelectionKind::Polygon { points } => (
                        "polygon",
                        Some(points.iter().map(|p| PointDto { x: p.x, y: p.y }).collect()),
                    ),
                    SelectionKind::MagicWand { .. } => ("magic_wand", None),
                    SelectionKind::ColorRange { .. } => ("color_range", None),
                };
                RenderSelectionDto {
                    id: selection.id.as_str().to_owned(),
                    rect: rect_dto(selection.bounds),
                    kind: kind.to_owned(),
                    points,
                }
            })
            .collect(),
    }
}

fn default_canvas_rect(workspace: &Workspace) -> Rect {
    content_canvas_rect(workspace).unwrap_or(Rect {
        x: 0.0,
        y: 0.0,
        width: 1024.0,
        height: 768.0,
    })
}

fn content_canvas_rect(workspace: &Workspace) -> Option<Rect> {
    workspace
        .layers
        .iter()
        .map(layer_workspace_rect)
        .chain(workspace.image_objects.iter().map(image_object_rect))
        .reduce(union_rect)
}

fn image_object_rect(object: &ImageObject) -> Rect {
    Rect {
        x: object.position.x,
        y: object.position.y,
        width: object.scale.width,
        height: object.scale.height,
    }
}

fn image_object_source(package: &WorkspacePackage, object: &ImageObject) -> Option<String> {
    package
        .workspace
        .assets
        .iter()
        .find(|asset| asset.id == object.source_asset_id)
        .and_then(|asset| match &asset.source {
            AssetSource::Linked { path, .. } => Some(path.clone()),
            AssetSource::Embedded { .. } => package
                .embedded_assets
                .iter()
                .find(|blob| blob.asset_id == asset.id)
                .map(|blob| embedded_asset_data_url(asset.media_type.as_deref(), &blob.bytes)),
        })
}

fn embedded_asset_data_url(media_type: Option<&str>, bytes: &[u8]) -> String {
    let media_type = media_type.unwrap_or("application/octet-stream");
    format!(
        "data:{media_type};base64,{}",
        general_purpose::STANDARD.encode(bytes)
    )
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

fn fit_rect(rect: Rect, screen: SizeDtoInput) -> (PointDto, f32) {
    let zoom = (screen.width / rect.width)
        .min(screen.height / rect.height)
        .mul_add(0.85, 0.0)
        .clamp(0.02, 64.0);
    let visible_width = screen.width / zoom;
    let visible_height = screen.height / zoom;
    (
        PointDto {
            x: rect.x + rect.width / 2.0 - visible_width / 2.0,
            y: rect.y + rect.height / 2.0 - visible_height / 2.0,
        },
        zoom,
    )
}

fn rect_dto(rect: Rect) -> RectDto {
    RectDto {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn padding_dto(padding: Padding) -> PaddingDto {
    PaddingDto {
        top: padding.top,
        right: padding.right,
        bottom: padding.bottom,
        left: padding.left,
    }
}

fn load_warning_dto(warning: LoadWarning) -> LoadWarningDto {
    match warning {
        LoadWarning::MigratedFileFormat { from, to } => LoadWarningDto::Migrated { from, to },
        LoadWarning::NewerFileFormat { found, supported } => {
            LoadWarningDto::NewerFile { found, supported }
        }
        LoadWarning::NewerWorkspaceFormat { found, supported } => {
            LoadWarningDto::NewerWorkspace { found, supported }
        }
    }
}

fn json_value(value: serde_json::Value) -> JsonValue {
    match value {
        serde_json::Value::Null => JsonValue::Null,
        serde_json::Value::Bool(value) => JsonValue::Bool(value),
        serde_json::Value::Number(value) => JsonValue::Number(value.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(value) => JsonValue::String(value),
        serde_json::Value::Array(values) => {
            JsonValue::Array(values.into_iter().map(json_value).collect())
        }
        serde_json::Value::Object(values) => JsonValue::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, json_value(value)))
                .collect(),
        ),
    }
}

/// Register an embedded image asset on the open workspace and return its id +
/// name. Shared by clipboard, drag/drop, and replace flows so they all produce
/// assets the subsequent `image.import_*` / `image.replace_source` core
/// commands can find.
fn register_acquired_asset(
    state: &tauri::State<'_, DesktopState>,
    name: &str,
    bytes: Vec<u8>,
) -> Result<AcquiredAssetDto, String> {
    let asset_id = generated_id("asset");
    let mut document = state.inner.lock().map_err(|_| "document lock poisoned")?;
    image_import::register_embedded_asset(
        &mut document.package,
        asset_id.clone(),
        name.to_owned(),
        bytes,
    )
    .map_err(|error| error.to_string())?;
    document.has_document = true;
    document.dirty = true;
    Ok(AcquiredAssetDto {
        asset_id: asset_id.as_str().to_owned(),
        name: name.to_owned(),
    })
}

fn encode_rgba_to_png(image: &ImageData<'_>) -> Result<Vec<u8>, String> {
    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.to_vec(),
    )
    .ok_or_else(|| "clipboard image had unexpected byte length".to_owned())?;
    let mut encoded = Vec::with_capacity(image.bytes.len());
    buffer
        .write_to(&mut Cursor::new(&mut encoded), image::ImageFormat::Png)
        .map_err(|error| format!("encode clipboard image: {error}"))?;
    Ok(encoded)
}

fn short_timestamp() -> String {
    format!("{:x}", unique_suffix())
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct RecentEntry {
    path: String,
    opened_at_secs: u64,
}

const RECENT_FILES_LIMIT: usize = 10;

fn recent_files_path() -> Option<PathBuf> {
    let mut dir = dirs::config_dir()?;
    dir.push("fleck");
    fs::create_dir_all(&dir).ok()?;
    dir.push("recent.json");
    Some(dir)
}

fn read_recent_files() -> Vec<RecentEntry> {
    let Some(path) = recent_files_path() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn push_recent_file(path: &Path) {
    let Some(store) = recent_files_path() else {
        return;
    };
    let canonical = path.to_string_lossy().into_owned();
    let mut entries = read_recent_files();
    entries.retain(|entry| entry.path != canonical);
    entries.insert(
        0,
        RecentEntry {
            path: canonical,
            opened_at_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
        },
    );
    entries.truncate(RECENT_FILES_LIMIT);
    let Ok(text) = serde_json::to_string_pretty(&entries) else {
        return;
    };
    let _ = fs::write(&store, text);
}

fn relative_time(secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let delta = now.saturating_sub(secs);
    if delta < 60 {
        "just now".to_owned()
    } else if delta < 3600 {
        format!("{} minutes ago", delta / 60)
    } else if delta < 86_400 {
        let hours = delta / 3600;
        if hours == 1 {
            "1 hour ago".to_owned()
        } else {
            format!("{hours} hours ago")
        }
    } else if delta < 7 * 86_400 {
        let days = delta / 86_400;
        if days == 1 {
            "yesterday".to_owned()
        } else {
            format!("{days} days ago")
        }
    } else {
        let weeks = delta / (7 * 86_400);
        if weeks == 1 {
            "1 week ago".to_owned()
        } else {
            format!("{weeks} weeks ago")
        }
    }
}

fn generated_id(prefix: &str) -> ObjectId {
    ObjectId::new(format!("{prefix}-{}", unique_suffix())).expect("generated id is valid")
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled.fleck")
        .to_owned()
}

fn render_color(index: usize) -> String {
    ["#3a86ff", "#06d6a0", "#ffbe0b", "#ef476f", "#8338ec"][index % 5].to_owned()
}

trait CaseSnake {
    fn to_case_snake(&self) -> String;
}

impl CaseSnake for str {
    fn to_case_snake(&self) -> String {
        let mut out = String::new();
        for (index, ch) in self.chars().enumerate() {
            if ch.is_uppercase() && index > 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        }
        out
    }
}
