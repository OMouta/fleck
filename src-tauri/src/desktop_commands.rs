use fleck_core::command::{
    default_command_registry, CommandEngine, CommandId, CommandInvocation, CommandParameters,
};
use fleck_core::export::{preview_export_area, ExportWarning, OutputScale};
use fleck_core::model::{
    AssetSource, ExportArea, ExportBackground, HistoryState, ImageObject, JsonValue, Layer,
    ObjectId, OutputFormat, Padding, Rect, Workspace,
};
use fleck_core::persistence::{
    load_package_from_path, save_package_to_path, LoadWarning, WorkspacePackage,
};
use fleck_render::{DefaultExportOptions, SkiaViewportRenderer};
use serde::Serialize;
use std::collections::BTreeMap;
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
}

impl DocumentState {
    fn new_empty() -> Self {
        Self {
            package: WorkspacePackage::new(Workspace::empty(generated_id("workspace"))),
            path: None,
            engine: CommandEngine::new(),
            dirty: false,
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
    dimensions: String,
    position: String,
    padding: String,
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
    destination: Option<String>,
    dimensions: String,
    bytes: String,
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
pub struct CanvasDto {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderLayerDto {
    id: String,
    rect: RectDto,
    color: String,
    opacity: f32,
    visible: bool,
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
    *document = DocumentState::new_empty();
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
    document.path = Some(path);
    document.dirty = false;
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
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn get_recent_files(
    state: tauri::State<'_, DesktopState>,
) -> Result<Vec<RecentFileDto>, String> {
    with_document(&state, |document| {
        Ok(document
            .path
            .as_ref()
            .map(|path| RecentFileDto {
                path: path.to_string_lossy().into_owned(),
                name: file_name(path),
                opened_at: "current session".to_owned(),
            })
            .into_iter()
            .collect())
    })
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

#[tauri::command]
pub fn acquire_clipboard_asset() -> Result<Option<serde_json::Value>, String> {
    Ok(None)
}

#[tauri::command]
pub fn acquire_dropped_asset(_name: String) -> Result<Option<serde_json::Value>, String> {
    Ok(None)
}

#[tauri::command]
pub fn acquire_replacement_asset() -> Result<Option<String>, String> {
    Ok(None)
}

#[tauri::command(rename_all = "camelCase")]
pub fn reveal_image_source(_object_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn relink_asset(_asset_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn get_render_model(state: tauri::State<'_, DesktopState>) -> Result<RenderModelDto, String> {
    with_document(&state, |document| {
        Ok(render_model(&document.package.workspace))
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
pub fn export_area(state: tauri::State<'_, DesktopState>, id: String) -> Result<(), String> {
    with_document(&state, |document| {
        let id = ObjectId::new(id).map_err(|error| error.to_string())?;
        match SkiaViewportRenderer::new().export_area(&document.package.workspace, &id) {
            Ok(_) => {}
            Err(fleck_render::ExportPipelineError::AreaHasNoOutputs { .. }) => {}
            Err(error) => return Err(error.to_string()),
        }
        Ok(())
    })
}

#[tauri::command]
pub fn export_all(state: tauri::State<'_, DesktopState>) -> Result<(), String> {
    with_document(&state, |document| {
        match SkiaViewportRenderer::new().export_all(
            &document.package.workspace,
            &DefaultExportOptions::default(),
        ) {
            Ok(_) => {}
            Err(fleck_render::ExportPipelineError::NoExportableContent) => {}
            Err(error) => return Err(error.to_string()),
        }
        Ok(())
    })
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
    document.path = Some(path);
    document.engine = CommandEngine::new();
    document.dirty = false;
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
    let bounds = default_canvas_rect(workspace);
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
    let quality_of = |output_id: &str| {
        workspace
            .outputs
            .iter()
            .find(|output| output.id.as_str() == output_id)
            .and_then(|output| output.quality)
    };
    let outputs = preview
        .as_ref()
        .map(|preview| {
            preview
                .outputs
                .iter()
                .map(|output| OutputDto {
                    id: output.output_id.as_str().to_owned(),
                    filename: output.filename.clone(),
                    format: output_format_label(output.format).to_owned(),
                    scale: scale_label(output.scale),
                    quality: quality_of(output.output_id.as_str()),
                    destination: output.destination.clone(),
                    dimensions: format!("{} × {} px", output.pixel_width, output.pixel_height),
                    bytes: "pending".to_owned(),
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
        dimensions: format!(
            "{} × {} px",
            area.bounds.width.round(),
            area.bounds.height.round()
        ),
        position: format!("{}, {}", area.bounds.x.round(), area.bounds.y.round()),
        padding: padding_label(&area.padding),
        background: background_label(&area.background),
        format: outputs
            .first()
            .map(|output| output.format.clone())
            .unwrap_or_else(|| "—".to_owned()),
        status: if warnings.is_empty() { "ready" } else { "warning" }.to_owned(),
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

fn render_model(workspace: &Workspace) -> RenderModelDto {
    let bounds = default_canvas_rect(workspace);
    RenderModelDto {
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
            .map(|selection| RenderSelectionDto {
                id: selection.id.as_str().to_owned(),
                rect: rect_dto(selection.bounds),
            })
            .collect(),
    }
}

fn default_canvas_rect(workspace: &Workspace) -> Rect {
    workspace
        .layers
        .iter()
        .map(layer_workspace_rect)
        .chain(workspace.image_objects.iter().map(image_object_rect))
        .reduce(union_rect)
        .unwrap_or(Rect {
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 768.0,
        })
}

fn image_object_rect(object: &ImageObject) -> Rect {
    Rect {
        x: object.position.x,
        y: object.position.y,
        width: object.scale.width,
        height: object.scale.height,
    }
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
