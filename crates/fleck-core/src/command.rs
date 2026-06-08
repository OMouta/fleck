use crate::export::{self, ExportError, NewExportArea, NewOutput, OutputUpdate};
use crate::image_import::{self, ImageImportError, ImagePlacement, LinkedImageImport};
use crate::layer::{self, LayerError, NewLayer};
use crate::model::{
    BlendMode, ClippingBehavior, CompressionSettings, ExportBackground, ExportParticipation,
    HistoryEntry, HistoryState, JsonValue, MetadataBehavior, ObjectId, OutputFormat, Padding,
    Point, Rect, TransparencyBehavior, TrimBehavior, ValidationError, Workspace,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandId(String);

impl CommandId {
    pub fn new(value: impl Into<String>) -> Result<Self, CommandError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CommandError::InvalidCommandId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommandParameters {
    values: BTreeMap<String, JsonValue>,
}

impl CommandParameters {
    pub fn new(values: impl IntoIterator<Item = (String, JsonValue)>) -> Self {
        Self {
            values: values.into_iter().collect(),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.values.get(key)
    }

    pub fn required_string(&self, key: &'static str) -> Result<&str, CommandError> {
        match self.get(key) {
            Some(JsonValue::String(value)) if !value.trim().is_empty() => Ok(value),
            Some(_) => Err(CommandError::InvalidParameter {
                key,
                expected: "non-empty string",
            }),
            None => Err(CommandError::MissingParameter { key }),
        }
    }

    pub fn optional_string(&self, key: &'static str) -> Result<Option<&str>, CommandError> {
        match self.get(key) {
            Some(JsonValue::String(value)) if !value.trim().is_empty() => Ok(Some(value)),
            Some(JsonValue::Null) | None => Ok(None),
            Some(_) => Err(CommandError::InvalidParameter {
                key,
                expected: "non-empty string or null",
            }),
        }
    }

    pub fn required_bool(&self, key: &'static str) -> Result<bool, CommandError> {
        match self.get(key) {
            Some(JsonValue::Bool(value)) => Ok(*value),
            Some(_) => Err(CommandError::InvalidParameter {
                key,
                expected: "boolean",
            }),
            None => Err(CommandError::MissingParameter { key }),
        }
    }

    pub fn optional_f32(&self, key: &'static str, default: f32) -> Result<f32, CommandError> {
        match self.get(key) {
            Some(JsonValue::Number(value)) if value.is_finite() => Ok(*value as f32),
            Some(JsonValue::Null) | None => Ok(default),
            Some(_) => Err(CommandError::InvalidParameter {
                key,
                expected: "finite number",
            }),
        }
    }

    pub fn optional_usize(&self, key: &'static str) -> Result<Option<usize>, CommandError> {
        match self.get(key) {
            Some(JsonValue::Number(value)) if value.is_finite() && *value >= 0.0 => {
                Ok(Some(*value as usize))
            }
            Some(JsonValue::Null) | None => Ok(None),
            Some(_) => Err(CommandError::InvalidParameter {
                key,
                expected: "non-negative integer",
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandInvocation {
    pub id: CommandId,
    pub parameters: CommandParameters,
    pub context: CommandContext,
}

impl CommandInvocation {
    pub fn new(id: CommandId) -> Self {
        Self {
            id,
            parameters: CommandParameters::empty(),
            context: CommandContext::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommandContext {
    pub selected_layer_id: Option<ObjectId>,
    pub selected_export_area_id: Option<ObjectId>,
    pub active_selection_id: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub id: CommandId,
    pub label: String,
    pub description: String,
    pub group: CommandGroup,
    pub aliases: Vec<String>,
    pub shortcut: Option<String>,
    pub undoable: bool,
    pub parameter_prompts: Vec<ParameterPrompt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandGroup {
    Workspace,
    Layer,
    ImageObject,
    Selection,
    Export,
    Recipe,
    View,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterPrompt {
    pub key: String,
    pub label: String,
    pub kind: ParameterKind,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterKind {
    String,
    Number,
    Boolean,
    ObjectId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandEffect {
    pub operation_label: String,
    pub progress: Option<CommandProgress>,
}

impl CommandEffect {
    pub fn undoable(operation_label: impl Into<String>) -> Self {
        Self {
            operation_label: operation_label.into(),
            progress: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandProgress {
    pub completed_units: u64,
    pub total_units: Option<u64>,
}

type CommandHandler =
    Box<dyn Fn(&mut Workspace, &CommandInvocation, &CommandRuntime) -> CommandResult + Send + Sync>;

pub type CommandResult = Result<CommandEffect, CommandError>;

pub struct RegisteredCommand {
    definition: CommandDefinition,
    handler: CommandHandler,
}

impl RegisteredCommand {
    pub fn definition(&self) -> &CommandDefinition {
        &self.definition
    }
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: BTreeMap<CommandId, RegisteredCommand>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        definition: CommandDefinition,
        handler: impl Fn(&mut Workspace, &CommandInvocation, &CommandRuntime) -> CommandResult
            + Send
            + Sync
            + 'static,
    ) -> Result<(), CommandError> {
        if self.commands.contains_key(&definition.id) {
            return Err(CommandError::DuplicateCommand {
                id: definition.id.clone(),
            });
        }

        self.commands.insert(
            definition.id.clone(),
            RegisteredCommand {
                definition,
                handler: Box::new(handler),
            },
        );
        Ok(())
    }

    pub fn get(&self, id: &CommandId) -> Option<&RegisteredCommand> {
        self.commands.get(id)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &CommandDefinition> {
        self.commands.values().map(RegisteredCommand::definition)
    }
}

#[derive(Debug, Clone)]
pub struct CommandRuntime {
    cancellation: CancellationToken,
    progress: ProgressSink,
}

impl CommandRuntime {
    pub fn new(cancellation: CancellationToken, progress: ProgressSink) -> Self {
        Self {
            cancellation,
            progress,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub fn ensure_not_cancelled(&self) -> Result<(), CommandError> {
        if self.is_cancelled() {
            Err(CommandError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn report_progress(&self, progress: CommandProgress) {
        self.progress.report(progress);
    }
}

impl Default for CommandRuntime {
    fn default() -> Self {
        Self {
            cancellation: CancellationToken::new(),
            progress: ProgressSink::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProgressSink {
    reports: Arc<std::sync::Mutex<Vec<CommandProgress>>>,
}

impl ProgressSink {
    pub fn report(&self, progress: CommandProgress) {
        if let Ok(mut reports) = self.reports.lock() {
            reports.push(progress);
        }
    }

    pub fn reports(&self) -> Vec<CommandProgress> {
        self.reports
            .lock()
            .map(|reports| reports.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandExecution {
    pub command_id: CommandId,
    pub operation_label: String,
}

#[derive(Debug, Clone, PartialEq)]
struct UndoEntry {
    command_id: CommandId,
    operation_label: String,
    before: Workspace,
    after: Workspace,
}

#[derive(Debug, Default)]
pub struct CommandEngine {
    undo_entries: Vec<UndoEntry>,
    current_index: Option<usize>,
}

impl CommandEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(
        &mut self,
        workspace: &mut Workspace,
        registry: &CommandRegistry,
        invocation: CommandInvocation,
        runtime: &CommandRuntime,
    ) -> Result<CommandExecution, CommandError> {
        runtime.ensure_not_cancelled()?;
        let registered =
            registry
                .get(&invocation.id)
                .ok_or_else(|| CommandError::UnknownCommand {
                    id: invocation.id.clone(),
                })?;

        let before = workspace.clone();
        let effect = (registered.handler)(workspace, &invocation, runtime)?;
        workspace.validate().map_err(CommandError::Validation)?;

        if registered.definition.undoable {
            if let Some(index) = self.current_index {
                self.undo_entries.truncate(index + 1);
            } else {
                self.undo_entries.clear();
            }

            self.undo_entries.push(UndoEntry {
                command_id: invocation.id.clone(),
                operation_label: effect.operation_label.clone(),
                before,
                after: workspace.clone(),
            });
            self.current_index = Some(self.undo_entries.len() - 1);
            sync_history(workspace, &self.undo_entries, self.current_index);
        }

        Ok(CommandExecution {
            command_id: invocation.id,
            operation_label: effect.operation_label,
        })
    }

    pub fn undo(&mut self, workspace: &mut Workspace) -> Result<CommandExecution, CommandError> {
        let index = self.current_index.ok_or(CommandError::NothingToUndo)?;
        let entry = self.undo_entries[index].clone();
        *workspace = entry.before;
        self.current_index = index.checked_sub(1);
        sync_history(workspace, &self.undo_entries, self.current_index);
        Ok(CommandExecution {
            command_id: entry.command_id,
            operation_label: format!("Undo {}", entry.operation_label),
        })
    }

    pub fn redo(&mut self, workspace: &mut Workspace) -> Result<CommandExecution, CommandError> {
        let next_index = self.current_index.map_or(0, |index| index + 1);
        let entry = self
            .undo_entries
            .get(next_index)
            .cloned()
            .ok_or(CommandError::NothingToRedo)?;
        *workspace = entry.after;
        self.current_index = Some(next_index);
        sync_history(workspace, &self.undo_entries, self.current_index);
        Ok(CommandExecution {
            command_id: entry.command_id,
            operation_label: format!("Redo {}", entry.operation_label),
        })
    }

    pub fn history(&self) -> HistoryState {
        history_from_entries(&self.undo_entries, self.current_index)
    }
}

pub fn default_command_registry() -> Result<CommandRegistry, CommandError> {
    let mut registry = CommandRegistry::new();
    registry.register(
        CommandDefinition {
            id: CommandId::new("workspace.rename")?,
            label: "Rename Workspace".to_owned(),
            description: "Rename the current workspace.".to_owned(),
            group: CommandGroup::Workspace,
            aliases: vec![
                "set workspace name".to_owned(),
                "rename document".to_owned(),
            ],
            shortcut: None,
            undoable: true,
            parameter_prompts: vec![ParameterPrompt {
                key: "name".to_owned(),
                label: "Name".to_owned(),
                kind: ParameterKind::String,
                required: true,
            }],
        },
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            workspace.metadata.name = name.clone();
            Ok(CommandEffect::undoable(format!(
                "Rename Workspace to {name}"
            )))
        },
    )?;
    register_layer_commands(&mut registry)?;
    register_image_commands(&mut registry)?;
    register_export_commands(&mut registry)?;
    Ok(registry)
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("command id cannot be empty")]
    InvalidCommandId,
    #[error("command `{id}` is already registered")]
    DuplicateCommand { id: CommandId },
    #[error("unknown command `{id}`")]
    UnknownCommand { id: CommandId },
    #[error("missing command parameter `{key}`")]
    MissingParameter { key: &'static str },
    #[error("invalid command parameter `{key}`; expected {expected}")]
    InvalidParameter {
        key: &'static str,
        expected: &'static str,
    },
    #[error("command was cancelled")]
    Cancelled,
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
    #[error("command produced invalid workspace state")]
    Validation(#[from] ValidationError),
    #[error("layer operation failed")]
    Layer(#[from] LayerError),
    #[error("image import operation failed")]
    Image(#[from] ImageImportError),
    #[error("export operation failed")]
    Export(#[from] ExportError),
    #[error("invalid object id parameter `{key}`")]
    InvalidObjectId {
        key: &'static str,
        issue: crate::model::ValidationIssue,
    },
}

fn register_layer_commands(registry: &mut CommandRegistry) -> Result<(), CommandError> {
    register_layer_command(
        registry,
        "layer.create",
        "Create Layer",
        "Create a raster layer.",
        &["new layer", "add layer"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("name", "Name", ParameterKind::String, true),
            prompt("x", "X", ParameterKind::Number, false),
            prompt("y", "Y", ParameterKind::Number, false),
            prompt("width", "Width", ParameterKind::Number, false),
            prompt("height", "Height", ParameterKind::Number, false),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            let x = invocation.parameters.optional_f32("x", 0.0)?;
            let y = invocation.parameters.optional_f32("y", 0.0)?;
            let width = invocation.parameters.optional_f32("width", 64.0)?;
            let height = invocation.parameters.optional_f32("height", 64.0)?;
            layer::create_layer(
                workspace,
                NewLayer {
                    id,
                    name: name.clone(),
                    position: Point { x, y },
                    bounds: Rect {
                        x: 0.0,
                        y: 0.0,
                        width,
                        height,
                    },
                },
            )?;
            Ok(CommandEffect::undoable(format!("Create Layer {name}")))
        },
    )?;
    register_layer_command(
        registry,
        "layer.delete",
        "Delete Layer",
        "Delete an unlocked layer.",
        &["remove layer"],
        Some("Delete"),
        vec![prompt("id", "Layer ID", ParameterKind::ObjectId, true)],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let deleted = layer::delete_layer(workspace, &id)?;
            Ok(CommandEffect::undoable(format!(
                "Delete Layer {}",
                deleted.name
            )))
        },
    )?;
    register_layer_command(
        registry,
        "layer.duplicate",
        "Duplicate Layer",
        "Duplicate a layer above the source layer.",
        &["copy layer"],
        Some("Ctrl+D"),
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("new_id", "New Layer ID", ParameterKind::ObjectId, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let new_id = required_object_id(&invocation.parameters, "new_id")?;
            layer::duplicate_layer(workspace, &id, new_id)?;
            Ok(CommandEffect::undoable("Duplicate Layer"))
        },
    )?;
    register_layer_command(
        registry,
        "layer.rename",
        "Rename Layer",
        "Rename an unlocked layer.",
        &["name layer"],
        Some("F2"),
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("name", "Name", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            layer::rename_layer(workspace, &id, name.clone())?;
            Ok(CommandEffect::undoable(format!("Rename Layer to {name}")))
        },
    )?;
    register_layer_command(
        registry,
        "layer.reorder",
        "Reorder Layer",
        "Move an unlocked layer to a new stack index.",
        &["move layer"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("index", "Index", ParameterKind::Number, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let index = invocation
                .parameters
                .optional_usize("index")?
                .ok_or(CommandError::MissingParameter { key: "index" })?;
            layer::reorder_layer(workspace, &id, index)?;
            Ok(CommandEffect::undoable("Reorder Layer"))
        },
    )?;
    register_bool_layer_command(
        registry,
        "layer.set_visible",
        "Set Layer Visibility",
        "Show or hide a layer.",
        "visible",
        |workspace, id, value| layer::set_visibility(workspace, &id, value),
    )?;
    register_bool_layer_command(
        registry,
        "layer.set_locked",
        "Set Layer Lock",
        "Lock or unlock a layer.",
        "locked",
        |workspace, id, value| layer::set_locked(workspace, &id, value),
    )?;
    register_layer_command(
        registry,
        "layer.set_opacity",
        "Set Layer Opacity",
        "Set layer opacity.",
        &["opacity"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("opacity", "Opacity", ParameterKind::Number, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let opacity = invocation.parameters.optional_f32("opacity", 1.0)?;
            layer::set_opacity(workspace, &id, opacity)?;
            Ok(CommandEffect::undoable("Set Layer Opacity"))
        },
    )?;
    register_layer_command(
        registry,
        "layer.set_blend_mode",
        "Set Layer Blend Mode",
        "Set layer blend mode.",
        &["blend mode"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("blend_mode", "Blend Mode", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let blend_mode =
                parse_blend_mode(invocation.parameters.required_string("blend_mode")?)?;
            layer::set_blend_mode(workspace, &id, blend_mode)?;
            Ok(CommandEffect::undoable("Set Layer Blend Mode"))
        },
    )?;
    register_layer_command(
        registry,
        "layer.set_clipping",
        "Set Layer Clipping",
        "Set layer clipping behavior.",
        &["clipping"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("clipping", "Clipping", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let clipping = parse_clipping(invocation.parameters.required_string("clipping")?)?;
            layer::set_clipping(workspace, &id, clipping)?;
            Ok(CommandEffect::undoable("Set Layer Clipping"))
        },
    )?;
    register_layer_command(
        registry,
        "layer.set_mask",
        "Set Layer Mask",
        "Assign or clear a layer mask.",
        &["mask layer"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt(
                "mask_layer_id",
                "Mask Layer ID",
                ParameterKind::ObjectId,
                false,
            ),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let mask_layer_id = optional_object_id(&invocation.parameters, "mask_layer_id")?;
            layer::set_mask(workspace, &id, mask_layer_id)?;
            Ok(CommandEffect::undoable("Set Layer Mask"))
        },
    )?;
    register_layer_command(
        registry,
        "layer.set_group",
        "Set Layer Group",
        "Assign or clear a layer group.",
        &["group layer"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("group_id", "Group ID", ParameterKind::ObjectId, false),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let group_id = optional_object_id(&invocation.parameters, "group_id")?;
            layer::set_layer_group(workspace, &id, group_id)?;
            Ok(CommandEffect::undoable("Set Layer Group"))
        },
    )?;
    register_layer_command(
        registry,
        "layer.group",
        "Create Layer Group",
        "Create a group from a layer.",
        &["new group"],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt("group_id", "Group ID", ParameterKind::ObjectId, true),
            prompt("name", "Name", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let group_id = required_object_id(&invocation.parameters, "group_id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            layer::create_group(workspace, group_id.clone(), name.clone(), vec![id.clone()])?;
            layer::set_layer_group(workspace, &id, Some(group_id))?;
            Ok(CommandEffect::undoable(format!(
                "Create Layer Group {name}"
            )))
        },
    )?;
    register_simple_id_layer_command(
        registry,
        "layer.merge_down",
        "Merge Layer Down",
        "Merge an unlocked layer into the layer below.",
        &["merge layer"],
        |workspace, id| layer::merge_down(workspace, &id),
    )?;
    register_layer_command(
        registry,
        "layer.flatten",
        "Flatten Visible Layers",
        "Flatten visible unlocked layers into a single raster layer.",
        &["flatten image"],
        None,
        vec![prompt(
            "flattened_id",
            "Flattened Layer ID",
            ParameterKind::ObjectId,
            true,
        )],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let flattened_id = required_object_id(&invocation.parameters, "flattened_id")?;
            layer::flatten_visible_layers(workspace, flattened_id)?;
            Ok(CommandEffect::undoable("Flatten Visible Layers"))
        },
    )?;
    register_simple_id_layer_command(
        registry,
        "layer.rasterize",
        "Rasterize Layer",
        "Rasterize an unlocked layer into layer pixels.",
        &["raster layer"],
        |workspace, id| layer::rasterize_layer(workspace, &id),
    )?;
    register_simple_id_layer_command(
        registry,
        "layer.trim_to_visible_pixels",
        "Trim Layer To Visible Pixels",
        "Trim layer bounds to visible pixels.",
        &["trim layer"],
        |workspace, id| layer::trim_to_visible_pixels(workspace, &id),
    )?;
    Ok(())
}

fn register_image_commands(registry: &mut CommandRegistry) -> Result<(), CommandError> {
    register_image_command(
        registry,
        "image.import_linked",
        "Import Linked Image",
        "Decode a linked image file and place it as an image object.",
        &["open image", "drag image in"],
        None,
        image_import_prompts(true),
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let path = PathBuf::from(invocation.parameters.required_string("path")?);
            let bytes = fs::read(&path).map_err(ImageImportError::Io)?;
            let decoded = image_import::decode_image_bytes(&bytes)?;
            let asset_id = required_object_id(&invocation.parameters, "asset_id")?;
            let object_id = required_object_id(&invocation.parameters, "object_id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            let placement = image_placement_from_parameters(
                &invocation.parameters,
                object_id,
                &name,
                &decoded.metadata,
            )?;
            image_import::import_linked_image(
                workspace,
                LinkedImageImport {
                    asset_id,
                    name: name.clone(),
                    path,
                    placement,
                },
            )?;
            Ok(CommandEffect::undoable(format!("Import Image {name}")))
        },
    )?;
    register_image_command(
        registry,
        "image.import_clipboard",
        "Import Clipboard Image",
        "Place an image object from a clipboard-provided asset.",
        &["paste image"],
        Some("Ctrl+V"),
        image_place_existing_prompts(),
        place_existing_asset_handler("Import Clipboard Image"),
    )?;
    register_image_command(
        registry,
        "image.import_drag_drop",
        "Import Dropped Image",
        "Place an image object from a drag/drop-provided asset.",
        &["drop image"],
        None,
        image_place_existing_prompts(),
        place_existing_asset_handler("Import Dropped Image"),
    )?;
    register_image_command(
        registry,
        "image.place_asset",
        "Place Image Asset",
        "Place an existing image asset as an image object.",
        &["new image object"],
        None,
        image_place_existing_prompts(),
        place_existing_asset_handler("Place Image Asset"),
    )?;
    register_image_command(
        registry,
        "image.duplicate_object",
        "Duplicate Image Object",
        "Duplicate a placed image object.",
        &["copy image object"],
        Some("Ctrl+D"),
        vec![
            prompt(
                "object_id",
                "Image Object ID",
                ParameterKind::ObjectId,
                true,
            ),
            prompt(
                "new_object_id",
                "New Image Object ID",
                ParameterKind::ObjectId,
                true,
            ),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let object_id = required_object_id(&invocation.parameters, "object_id")?;
            let new_object_id = required_object_id(&invocation.parameters, "new_object_id")?;
            image_import::duplicate_image_object(workspace, &object_id, new_object_id)?;
            Ok(CommandEffect::undoable("Duplicate Image Object"))
        },
    )?;
    register_image_command(
        registry,
        "image.replace_source",
        "Replace Image Source",
        "Replace an image object's source asset while preserving object settings.",
        &["replace image"],
        None,
        vec![
            prompt(
                "object_id",
                "Image Object ID",
                ParameterKind::ObjectId,
                true,
            ),
            prompt(
                "asset_id",
                "Replacement Asset ID",
                ParameterKind::ObjectId,
                true,
            ),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let object_id = required_object_id(&invocation.parameters, "object_id")?;
            let asset_id = required_object_id(&invocation.parameters, "asset_id")?;
            image_import::replace_image_source(workspace, &object_id, asset_id)?;
            Ok(CommandEffect::undoable("Replace Image Source"))
        },
    )?;
    register_image_command(
        registry,
        "image.rasterize_object",
        "Rasterize Image Object",
        "Rasterize a placed image object into an editable layer.",
        &["rasterize image"],
        None,
        vec![
            prompt(
                "object_id",
                "Image Object ID",
                ParameterKind::ObjectId,
                true,
            ),
            prompt("layer_id", "Layer ID", ParameterKind::ObjectId, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let object_id = required_object_id(&invocation.parameters, "object_id")?;
            let layer_id = required_object_id(&invocation.parameters, "layer_id")?;
            image_import::rasterize_image_object(workspace, &object_id, layer_id)?;
            Ok(CommandEffect::undoable("Rasterize Image Object"))
        },
    )?;
    Ok(())
}

fn register_export_commands(registry: &mut CommandRegistry) -> Result<(), CommandError> {
    register_export_command(
        registry,
        "export_area.create",
        "Create Export Area",
        "Create a named export metadata region.",
        &["new export area", "mark export area"],
        None,
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("name", "Name", ParameterKind::String, true),
            prompt("x", "X", ParameterKind::Number, false),
            prompt("y", "Y", ParameterKind::Number, false),
            prompt("width", "Width", ParameterKind::Number, true),
            prompt("height", "Height", ParameterKind::Number, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            let area = NewExportArea {
                id,
                name: name.clone(),
                bounds: Rect {
                    x: invocation.parameters.optional_f32("x", 0.0)?,
                    y: invocation.parameters.optional_f32("y", 0.0)?,
                    width: required_f32(&invocation.parameters, "width")?,
                    height: required_f32(&invocation.parameters, "height")?,
                },
                padding: Padding::default(),
                background: ExportBackground::Transparent,
                trim: TrimBehavior::None,
                output_ids: Vec::new(),
                included_layer_ids: Vec::new(),
                excluded_layer_ids: Vec::new(),
                tags: Vec::new(),
                preset_id: None,
            };
            export::create_export_area(workspace, area)?;
            Ok(CommandEffect::undoable(format!(
                "Create Export Area {name}"
            )))
        },
    )?;
    register_export_command(
        registry,
        "export_area.rename",
        "Rename Export Area",
        "Rename an export area.",
        &["name export area"],
        Some("F2"),
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("name", "Name", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            export::rename_export_area(workspace, &id, name.clone())?;
            Ok(CommandEffect::undoable(format!(
                "Rename Export Area to {name}"
            )))
        },
    )?;
    register_export_command(
        registry,
        "export_area.move",
        "Move Export Area",
        "Move an export area.",
        &["position export area"],
        None,
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("x", "X", ParameterKind::Number, true),
            prompt("y", "Y", ParameterKind::Number, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let x = required_f32(&invocation.parameters, "x")?;
            let y = required_f32(&invocation.parameters, "y")?;
            export::move_export_area(workspace, &id, x, y)?;
            Ok(CommandEffect::undoable("Move Export Area"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.resize",
        "Resize Export Area",
        "Resize an export area.",
        &["size export area"],
        None,
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("width", "Width", ParameterKind::Number, true),
            prompt("height", "Height", ParameterKind::Number, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            export::resize_export_area(
                workspace,
                &id,
                required_f32(&invocation.parameters, "width")?,
                required_f32(&invocation.parameters, "height")?,
            )?;
            Ok(CommandEffect::undoable("Resize Export Area"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.duplicate",
        "Duplicate Export Area",
        "Duplicate an export area.",
        &["copy export area"],
        Some("Ctrl+D"),
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt(
                "new_id",
                "New Export Area ID",
                ParameterKind::ObjectId,
                true,
            ),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let new_id = required_object_id(&invocation.parameters, "new_id")?;
            export::duplicate_export_area(workspace, &id, new_id)?;
            Ok(CommandEffect::undoable("Duplicate Export Area"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.set_tags",
        "Set Export Area Tags",
        "Set comma-separated export area tags.",
        &["tag export area"],
        None,
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("tags", "Tags", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let tags = invocation
                .parameters
                .required_string("tags")?
                .split(',')
                .map(str::to_owned)
                .collect();
            export::set_export_area_tags(workspace, &id, tags)?;
            Ok(CommandEffect::undoable("Set Export Area Tags"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.group",
        "Create Export Area Group",
        "Create a group containing an export area.",
        &["group export area"],
        None,
        vec![
            prompt("id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("group_id", "Group ID", ParameterKind::ObjectId, true),
            prompt("name", "Name", ParameterKind::String, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let group_id = required_object_id(&invocation.parameters, "group_id")?;
            let name = invocation.parameters.required_string("name")?.to_owned();
            export::group_export_area(workspace, &id, group_id, name.clone())?;
            Ok(CommandEffect::undoable(format!(
                "Create Export Area Group {name}"
            )))
        },
    )?;
    register_export_command(
        registry,
        "export_area.delete",
        "Delete Export Area",
        "Delete an export area.",
        &["remove export area"],
        Some("Delete"),
        vec![prompt(
            "id",
            "Export Area ID",
            ParameterKind::ObjectId,
            true,
        )],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let deleted = export::delete_export_area(workspace, &id)?;
            Ok(CommandEffect::undoable(format!(
                "Delete Export Area {}",
                deleted.name
            )))
        },
    )?;
    register_export_command(
        registry,
        "output.add",
        "Add Output",
        "Add an output definition.",
        &["new output"],
        None,
        output_prompts(true),
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let output = new_output_from_parameters(&invocation.parameters)?;
            let filename = output.filename.clone();
            export::add_output(workspace, output)?;
            Ok(CommandEffect::undoable(format!("Add Output {filename}")))
        },
    )?;
    register_export_command(
        registry,
        "output.remove",
        "Remove Output",
        "Remove an output definition and detach it from export areas.",
        &["delete output"],
        Some("Delete"),
        vec![prompt("id", "Output ID", ParameterKind::ObjectId, true)],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let removed = export::remove_output(workspace, &id)?;
            Ok(CommandEffect::undoable(format!(
                "Remove Output {}",
                removed.filename
            )))
        },
    )?;
    register_export_command(
        registry,
        "output.duplicate",
        "Duplicate Output",
        "Duplicate an output definition.",
        &["copy output"],
        Some("Ctrl+D"),
        vec![
            prompt("id", "Output ID", ParameterKind::ObjectId, true),
            prompt("new_id", "New Output ID", ParameterKind::ObjectId, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let new_id = required_object_id(&invocation.parameters, "new_id")?;
            export::duplicate_output(workspace, &id, new_id)?;
            Ok(CommandEffect::undoable("Duplicate Output"))
        },
    )?;
    register_export_command(
        registry,
        "output.update",
        "Update Output",
        "Update output settings.",
        &["edit output"],
        None,
        output_prompts(false),
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            export::update_output(
                workspace,
                &id,
                output_update_from_parameters(&invocation.parameters)?,
            )?;
            Ok(CommandEffect::undoable("Update Output"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.attach_output",
        "Attach Output To Export Area",
        "Attach an output definition to an export area.",
        &["add output to export area"],
        None,
        vec![
            prompt("area_id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("output_id", "Output ID", ParameterKind::ObjectId, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let area_id = required_object_id(&invocation.parameters, "area_id")?;
            let output_id = required_object_id(&invocation.parameters, "output_id")?;
            export::attach_output_to_area(workspace, &area_id, output_id)?;
            Ok(CommandEffect::undoable("Attach Output To Export Area"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.detach_output",
        "Detach Output From Export Area",
        "Detach an output definition from an export area.",
        &["remove output from export area"],
        None,
        vec![
            prompt("area_id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("output_id", "Output ID", ParameterKind::ObjectId, true),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let area_id = required_object_id(&invocation.parameters, "area_id")?;
            let output_id = required_object_id(&invocation.parameters, "output_id")?;
            export::detach_output_from_area(workspace, &area_id, &output_id)?;
            Ok(CommandEffect::undoable("Detach Output From Export Area"))
        },
    )?;
    register_export_command(
        registry,
        "export_area.set_layer_inclusion",
        "Set Export Area Layer Inclusion",
        "Include, exclude, or inherit a layer for an export area.",
        &["export layer rule"],
        None,
        vec![
            prompt("area_id", "Export Area ID", ParameterKind::ObjectId, true),
            prompt("layer_id", "Layer ID", ParameterKind::ObjectId, true),
            prompt(
                "participation",
                "Participation",
                ParameterKind::String,
                true,
            ),
        ],
        |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let area_id = required_object_id(&invocation.parameters, "area_id")?;
            let layer_id = required_object_id(&invocation.parameters, "layer_id")?;
            let participation = parse_export_participation(
                invocation.parameters.required_string("participation")?,
            )?;
            export::set_layer_inclusion(workspace, &area_id, layer_id, participation)?;
            Ok(CommandEffect::undoable("Set Export Area Layer Inclusion"))
        },
    )?;
    Ok(())
}

fn register_layer_command(
    registry: &mut CommandRegistry,
    id: &str,
    label: &str,
    description: &str,
    aliases: &[&str],
    shortcut: Option<&str>,
    parameter_prompts: Vec<ParameterPrompt>,
    handler: impl Fn(&mut Workspace, &CommandInvocation, &CommandRuntime) -> CommandResult
        + Send
        + Sync
        + 'static,
) -> Result<(), CommandError> {
    registry.register(
        CommandDefinition {
            id: CommandId::new(id)?,
            label: label.to_owned(),
            description: description.to_owned(),
            group: CommandGroup::Layer,
            aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
            shortcut: shortcut.map(str::to_owned),
            undoable: true,
            parameter_prompts,
        },
        handler,
    )
}

fn register_export_command(
    registry: &mut CommandRegistry,
    id: &str,
    label: &str,
    description: &str,
    aliases: &[&str],
    shortcut: Option<&str>,
    parameter_prompts: Vec<ParameterPrompt>,
    handler: impl Fn(&mut Workspace, &CommandInvocation, &CommandRuntime) -> CommandResult
        + Send
        + Sync
        + 'static,
) -> Result<(), CommandError> {
    registry.register(
        CommandDefinition {
            id: CommandId::new(id)?,
            label: label.to_owned(),
            description: description.to_owned(),
            group: CommandGroup::Export,
            aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
            shortcut: shortcut.map(str::to_owned),
            undoable: true,
            parameter_prompts,
        },
        handler,
    )
}

fn register_image_command(
    registry: &mut CommandRegistry,
    id: &str,
    label: &str,
    description: &str,
    aliases: &[&str],
    shortcut: Option<&str>,
    parameter_prompts: Vec<ParameterPrompt>,
    handler: impl Fn(&mut Workspace, &CommandInvocation, &CommandRuntime) -> CommandResult
        + Send
        + Sync
        + 'static,
) -> Result<(), CommandError> {
    registry.register(
        CommandDefinition {
            id: CommandId::new(id)?,
            label: label.to_owned(),
            description: description.to_owned(),
            group: CommandGroup::ImageObject,
            aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
            shortcut: shortcut.map(str::to_owned),
            undoable: true,
            parameter_prompts,
        },
        handler,
    )
}

fn image_import_prompts(include_path: bool) -> Vec<ParameterPrompt> {
    let mut prompts = image_place_existing_prompts();
    if include_path {
        prompts.insert(0, prompt("path", "Path", ParameterKind::String, true));
    }
    prompts
}

fn image_place_existing_prompts() -> Vec<ParameterPrompt> {
    vec![
        prompt("asset_id", "Asset ID", ParameterKind::ObjectId, true),
        prompt(
            "object_id",
            "Image Object ID",
            ParameterKind::ObjectId,
            true,
        ),
        prompt("name", "Name", ParameterKind::String, true),
        prompt("x", "X", ParameterKind::Number, false),
        prompt("y", "Y", ParameterKind::Number, false),
        prompt("scale_width", "Scale Width", ParameterKind::Number, false),
        prompt("scale_height", "Scale Height", ParameterKind::Number, false),
        prompt("rotation_degrees", "Rotation", ParameterKind::Number, false),
        prompt("opacity", "Opacity", ParameterKind::Number, false),
    ]
}

fn place_existing_asset_handler(
    label: &'static str,
) -> impl Fn(&mut Workspace, &CommandInvocation, &CommandRuntime) -> CommandResult + Send + Sync {
    move |workspace, invocation, runtime| {
        runtime.ensure_not_cancelled()?;
        let asset_id = required_object_id(&invocation.parameters, "asset_id")?;
        let object_id = required_object_id(&invocation.parameters, "object_id")?;
        let name = invocation.parameters.required_string("name")?.to_owned();
        let metadata = workspace
            .assets
            .iter()
            .find(|asset| asset.id == asset_id)
            .and_then(|asset| asset.image_metadata.as_ref())
            .cloned()
            .ok_or_else(|| ImageImportError::AssetNotFound {
                id: asset_id.clone(),
            })?;
        let placement =
            image_placement_from_parameters(&invocation.parameters, object_id, &name, &metadata)?;
        image_import::place_existing_asset(workspace, asset_id, placement)?;
        Ok(CommandEffect::undoable(label))
    }
}

fn image_placement_from_parameters(
    parameters: &CommandParameters,
    object_id: ObjectId,
    name: &str,
    metadata: &crate::model::ImageAssetMetadata,
) -> Result<ImagePlacement, CommandError> {
    let mut placement = ImagePlacement::new(object_id, name.to_owned(), metadata);
    placement.position = Point {
        x: parameters.optional_f32("x", 0.0)?,
        y: parameters.optional_f32("y", 0.0)?,
    };
    placement.scale = crate::model::Size {
        width: parameters.optional_f32("scale_width", metadata.width as f32)?,
        height: parameters.optional_f32("scale_height", metadata.height as f32)?,
    };
    placement.rotation_degrees = parameters.optional_f32("rotation_degrees", 0.0)?;
    placement.opacity = parameters.optional_f32("opacity", 1.0)?;
    Ok(placement)
}

fn output_prompts(require_fields: bool) -> Vec<ParameterPrompt> {
    vec![
        prompt("id", "Output ID", ParameterKind::ObjectId, true),
        prompt(
            "filename",
            "Filename",
            ParameterKind::String,
            require_fields,
        ),
        prompt("folder", "Folder", ParameterKind::String, false),
        prompt("format", "Format", ParameterKind::String, require_fields),
        prompt("width", "Width", ParameterKind::Number, false),
        prompt("height", "Height", ParameterKind::Number, false),
        prompt("scale", "Scale", ParameterKind::Number, false),
        prompt("quality", "Quality", ParameterKind::Number, false),
        prompt("background", "Background", ParameterKind::String, false),
        prompt("transparency", "Transparency", ParameterKind::String, false),
        prompt("metadata", "Metadata", ParameterKind::String, false),
    ]
}

fn new_output_from_parameters(parameters: &CommandParameters) -> Result<NewOutput, CommandError> {
    Ok(NewOutput {
        id: required_object_id(parameters, "id")?,
        filename: parameters.required_string("filename")?.to_owned(),
        folder: parameters.optional_string("folder")?.map(str::to_owned),
        format: parse_output_format(parameters.required_string("format")?)?,
        width: optional_u32(parameters, "width")?,
        height: optional_u32(parameters, "height")?,
        scale: parameters.optional_f32("scale", 1.0)?,
        quality: optional_u8(parameters, "quality")?,
        compression: CompressionSettings::default(),
        background: optional_export_background(parameters, "background")?
            .unwrap_or(ExportBackground::Transparent),
        transparency: optional_transparency(parameters, "transparency")?
            .unwrap_or(TransparencyBehavior::Preserve),
        metadata: optional_metadata(parameters, "metadata")?.unwrap_or(MetadataBehavior::Strip),
    })
}

fn output_update_from_parameters(
    parameters: &CommandParameters,
) -> Result<OutputUpdate, CommandError> {
    Ok(OutputUpdate {
        filename: parameters.optional_string("filename")?.map(str::to_owned),
        folder: optional_nullable_string(parameters, "folder")?,
        format: optional_output_format(parameters, "format")?,
        width: optional_nullable_u32(parameters, "width")?,
        height: optional_nullable_u32(parameters, "height")?,
        scale: optional_present_f32(parameters, "scale")?,
        quality: optional_nullable_u8(parameters, "quality")?,
        background: optional_export_background(parameters, "background")?,
        transparency: optional_transparency(parameters, "transparency")?,
        metadata: optional_metadata(parameters, "metadata")?,
    })
}

fn register_bool_layer_command(
    registry: &mut CommandRegistry,
    id: &str,
    label: &str,
    description: &str,
    value_key: &'static str,
    operation: impl Fn(&mut Workspace, ObjectId, bool) -> layer::LayerResult<()> + Send + Sync + 'static,
) -> Result<(), CommandError> {
    let operation_label = label.to_owned();
    register_layer_command(
        registry,
        id,
        label,
        description,
        &[],
        None,
        vec![
            prompt("id", "Layer ID", ParameterKind::ObjectId, true),
            prompt(value_key, "Value", ParameterKind::Boolean, true),
        ],
        move |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            let value = invocation.parameters.required_bool(value_key)?;
            operation(workspace, id, value)?;
            Ok(CommandEffect::undoable(operation_label.clone()))
        },
    )
}

fn register_simple_id_layer_command(
    registry: &mut CommandRegistry,
    id: &str,
    label: &str,
    description: &str,
    aliases: &[&str],
    operation: impl Fn(&mut Workspace, ObjectId) -> layer::LayerResult<()> + Send + Sync + 'static,
) -> Result<(), CommandError> {
    let operation_label = label.to_owned();
    register_layer_command(
        registry,
        id,
        label,
        description,
        aliases,
        None,
        vec![prompt("id", "Layer ID", ParameterKind::ObjectId, true)],
        move |workspace, invocation, runtime| {
            runtime.ensure_not_cancelled()?;
            let id = required_object_id(&invocation.parameters, "id")?;
            operation(workspace, id)?;
            Ok(CommandEffect::undoable(operation_label.clone()))
        },
    )
}

fn prompt(key: &str, label: &str, kind: ParameterKind, required: bool) -> ParameterPrompt {
    ParameterPrompt {
        key: key.to_owned(),
        label: label.to_owned(),
        kind,
        required,
    }
}

fn required_object_id(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<ObjectId, CommandError> {
    let value = parameters.required_string(key)?;
    ObjectId::new(value).map_err(|issue| CommandError::InvalidObjectId { key, issue })
}

fn optional_object_id(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<ObjectId>, CommandError> {
    parameters
        .optional_string(key)?
        .map(|value| {
            ObjectId::new(value).map_err(|issue| CommandError::InvalidObjectId { key, issue })
        })
        .transpose()
}

fn required_f32(parameters: &CommandParameters, key: &'static str) -> Result<f32, CommandError> {
    match parameters.get(key) {
        Some(JsonValue::Number(value)) if value.is_finite() => Ok(*value as f32),
        Some(_) => Err(CommandError::InvalidParameter {
            key,
            expected: "finite number",
        }),
        None => Err(CommandError::MissingParameter { key }),
    }
}

fn optional_present_f32(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<f32>, CommandError> {
    match parameters.get(key) {
        Some(JsonValue::Number(value)) if value.is_finite() => Ok(Some(*value as f32)),
        Some(JsonValue::Null) | None => Ok(None),
        Some(_) => Err(CommandError::InvalidParameter {
            key,
            expected: "finite number or null",
        }),
    }
}

fn optional_u32(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<u32>, CommandError> {
    match parameters.get(key) {
        Some(JsonValue::Number(value))
            if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 =>
        {
            Ok(Some(*value as u32))
        }
        Some(JsonValue::Null) | None => Ok(None),
        Some(_) => Err(CommandError::InvalidParameter {
            key,
            expected: "non-negative integer or null",
        }),
    }
}

fn optional_nullable_u32(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<Option<u32>>, CommandError> {
    match parameters.get(key) {
        Some(JsonValue::Null) => Ok(Some(None)),
        Some(_) => optional_u32(parameters, key).map(Some),
        None => Ok(None),
    }
}

fn optional_u8(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<u8>, CommandError> {
    match optional_u32(parameters, key)? {
        Some(value) if value <= u8::MAX as u32 => Ok(Some(value as u8)),
        Some(_) => Err(CommandError::InvalidParameter {
            key,
            expected: "0..=255 integer or null",
        }),
        None => Ok(None),
    }
}

fn optional_nullable_u8(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<Option<u8>>, CommandError> {
    match parameters.get(key) {
        Some(JsonValue::Null) => Ok(Some(None)),
        Some(_) => optional_u8(parameters, key).map(Some),
        None => Ok(None),
    }
}

fn optional_nullable_string(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<Option<String>>, CommandError> {
    match parameters.get(key) {
        Some(JsonValue::String(value)) if !value.trim().is_empty() => Ok(Some(Some(value.clone()))),
        Some(JsonValue::Null) => Ok(Some(None)),
        None => Ok(None),
        Some(_) => Err(CommandError::InvalidParameter {
            key,
            expected: "non-empty string or null",
        }),
    }
}

fn parse_blend_mode(value: &str) -> Result<BlendMode, CommandError> {
    match value {
        "normal" => Ok(BlendMode::Normal),
        "multiply" => Ok(BlendMode::Multiply),
        "screen" => Ok(BlendMode::Screen),
        "overlay" => Ok(BlendMode::Overlay),
        "darken" => Ok(BlendMode::Darken),
        "lighten" => Ok(BlendMode::Lighten),
        "color_dodge" => Ok(BlendMode::ColorDodge),
        "color_burn" => Ok(BlendMode::ColorBurn),
        "hard_light" => Ok(BlendMode::HardLight),
        "soft_light" => Ok(BlendMode::SoftLight),
        "difference" => Ok(BlendMode::Difference),
        "exclusion" => Ok(BlendMode::Exclusion),
        "hue" => Ok(BlendMode::Hue),
        "saturation" => Ok(BlendMode::Saturation),
        "color" => Ok(BlendMode::Color),
        "luminosity" => Ok(BlendMode::Luminosity),
        _ => Err(CommandError::InvalidParameter {
            key: "blend_mode",
            expected: "known blend mode",
        }),
    }
}

fn parse_export_participation(value: &str) -> Result<ExportParticipation, CommandError> {
    match value {
        "included" => Ok(ExportParticipation::Included),
        "excluded" => Ok(ExportParticipation::Excluded),
        "inherit" => Ok(ExportParticipation::Inherit),
        _ => Err(CommandError::InvalidParameter {
            key: "participation",
            expected: "included, excluded, or inherit",
        }),
    }
}

fn optional_output_format(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<OutputFormat>, CommandError> {
    parameters
        .optional_string(key)?
        .map(parse_output_format)
        .transpose()
}

fn parse_output_format(value: &str) -> Result<OutputFormat, CommandError> {
    match value {
        "png" => Ok(OutputFormat::Png),
        "jpeg" | "jpg" => Ok(OutputFormat::Jpeg),
        "webp" => Ok(OutputFormat::WebP),
        "avif" => Ok(OutputFormat::Avif),
        "gif" => Ok(OutputFormat::Gif),
        "bmp" => Ok(OutputFormat::Bmp),
        "tiff" => Ok(OutputFormat::Tiff),
        "ico" => Ok(OutputFormat::Ico),
        "icns" => Ok(OutputFormat::Icns),
        "svg_rasterized" => Ok(OutputFormat::SvgRasterized),
        "pdf" => Ok(OutputFormat::Pdf),
        _ => Err(CommandError::InvalidParameter {
            key: "format",
            expected: "known output format",
        }),
    }
}

fn optional_export_background(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<ExportBackground>, CommandError> {
    parameters
        .optional_string(key)?
        .map(parse_export_background)
        .transpose()
}

fn parse_export_background(value: &str) -> Result<ExportBackground, CommandError> {
    match value {
        "transparent" => Ok(ExportBackground::Transparent),
        "checkerboard_preview" => Ok(ExportBackground::CheckerboardPreview),
        "white" => Ok(ExportBackground::Solid {
            color: crate::model::RgbaColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        }),
        "black" => Ok(ExportBackground::Solid {
            color: crate::model::RgbaColor {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        }),
        _ => Err(CommandError::InvalidParameter {
            key: "background",
            expected: "transparent, checkerboard_preview, white, or black",
        }),
    }
}

fn optional_transparency(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<TransparencyBehavior>, CommandError> {
    parameters
        .optional_string(key)?
        .map(parse_transparency)
        .transpose()
}

fn parse_transparency(value: &str) -> Result<TransparencyBehavior, CommandError> {
    match value {
        "preserve" => Ok(TransparencyBehavior::Preserve),
        "flatten" => Ok(TransparencyBehavior::Flatten),
        _ => Err(CommandError::InvalidParameter {
            key: "transparency",
            expected: "preserve or flatten",
        }),
    }
}

fn optional_metadata(
    parameters: &CommandParameters,
    key: &'static str,
) -> Result<Option<MetadataBehavior>, CommandError> {
    parameters
        .optional_string(key)?
        .map(parse_metadata)
        .transpose()
}

fn parse_metadata(value: &str) -> Result<MetadataBehavior, CommandError> {
    match value {
        "preserve" => Ok(MetadataBehavior::Preserve),
        "strip" => Ok(MetadataBehavior::Strip),
        _ => Err(CommandError::InvalidParameter {
            key: "metadata",
            expected: "preserve or strip",
        }),
    }
}

fn parse_clipping(value: &str) -> Result<ClippingBehavior, CommandError> {
    match value {
        "none" => Ok(ClippingBehavior::None),
        "clip_to_layer_below" => Ok(ClippingBehavior::ClipToLayerBelow),
        "clip_to_group" => Ok(ClippingBehavior::ClipToGroup),
        _ => Err(CommandError::InvalidParameter {
            key: "clipping",
            expected: "known clipping behavior",
        }),
    }
}

fn sync_history(workspace: &mut Workspace, entries: &[UndoEntry], current_index: Option<usize>) {
    workspace.history = history_from_entries(entries, current_index);
}

fn history_from_entries(entries: &[UndoEntry], current_index: Option<usize>) -> HistoryState {
    HistoryState {
        entries: entries
            .iter()
            .enumerate()
            .map(|(index, entry)| HistoryEntry {
                id: ObjectId::new(format!("history-{index}")).expect("generated id is valid"),
                command_id: entry.command_id.as_str().to_owned(),
                label: entry.operation_label.clone(),
            })
            .collect(),
        current_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Asset, AssetSource, ImageAssetMetadata, ImageFormat, Workspace};
    use image::{ImageBuffer, ImageFormat as CrateImageFormat, Rgba};
    use std::io::Cursor;

    #[test]
    fn registry_lists_command_definitions() {
        let registry = default_command_registry().expect("registry");
        let definitions = registry.definitions().collect::<Vec<_>>();

        assert!(definitions
            .iter()
            .any(|definition| definition.id.as_str() == "workspace.rename"));
        assert!(definitions
            .iter()
            .any(|definition| definition.id.as_str() == "layer.create"
                && definition.group == CommandGroup::Layer
                && definition.undoable));
        assert!(definitions
            .iter()
            .any(|definition| definition.id.as_str() == "layer.flatten"));
        assert!(definitions.iter().any(|definition| {
            definition.id.as_str() == "image.import_linked"
                && definition.group == CommandGroup::ImageObject
                && definition.undoable
        }));
    }

    #[test]
    fn execute_command_updates_workspace_and_history() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));

        let execution = engine
            .execute(
                &mut workspace,
                &registry,
                rename_invocation("Brand Assets"),
                &CommandRuntime::default(),
            )
            .expect("execute rename");

        assert_eq!(workspace.metadata.name, "Brand Assets");
        assert_eq!(execution.command_id.as_str(), "workspace.rename");
        assert_eq!(workspace.history.entries.len(), 1);
        assert_eq!(workspace.history.current_index, Some(0));
        assert_eq!(
            workspace.history.entries[0].label,
            execution.operation_label
        );
    }

    #[test]
    fn undo_and_redo_restore_workspace_snapshots() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));
        let original_name = workspace.metadata.name.clone();

        engine
            .execute(
                &mut workspace,
                &registry,
                rename_invocation("Brand Assets"),
                &CommandRuntime::default(),
            )
            .expect("execute rename");
        engine.undo(&mut workspace).expect("undo");

        assert_eq!(workspace.metadata.name, original_name);
        assert_eq!(workspace.history.current_index, None);

        engine.redo(&mut workspace).expect("redo");

        assert_eq!(workspace.metadata.name, "Brand Assets");
        assert_eq!(workspace.history.current_index, Some(0));
    }

    #[test]
    fn executing_after_undo_discards_redo_entries() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));

        engine
            .execute(
                &mut workspace,
                &registry,
                rename_invocation("One"),
                &CommandRuntime::default(),
            )
            .expect("first rename");
        engine.undo(&mut workspace).expect("undo first");
        engine
            .execute(
                &mut workspace,
                &registry,
                rename_invocation("Two"),
                &CommandRuntime::default(),
            )
            .expect("second rename");

        assert_eq!(workspace.metadata.name, "Two");
        assert!(matches!(
            engine.redo(&mut workspace),
            Err(CommandError::NothingToRedo)
        ));
        assert_eq!(workspace.history.entries.len(), 1);
    }

    #[test]
    fn cancellation_prevents_execution() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let runtime = CommandRuntime::new(cancellation, ProgressSink::default());

        let result = engine.execute(
            &mut workspace,
            &registry,
            rename_invocation("Never"),
            &runtime,
        );

        assert!(matches!(result, Err(CommandError::Cancelled)));
        assert_ne!(workspace.metadata.name, "Never");
    }

    #[test]
    fn progress_sink_records_reports() {
        let sink = ProgressSink::default();
        let runtime = CommandRuntime::new(CancellationToken::new(), sink.clone());

        runtime.report_progress(CommandProgress {
            completed_units: 1,
            total_units: Some(3),
        });

        assert_eq!(
            sink.reports(),
            vec![CommandProgress {
                completed_units: 1,
                total_units: Some(3)
            }]
        );
    }

    #[test]
    fn layer_commands_are_undoable() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));

        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "layer.create",
                    vec![
                        ("id", JsonValue::String("layer-a".to_owned())),
                        ("name", JsonValue::String("Layer A".to_owned())),
                        ("width", JsonValue::Number(32.0)),
                        ("height", JsonValue::Number(16.0)),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("create layer");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "layer.set_opacity",
                    vec![
                        ("id", JsonValue::String("layer-a".to_owned())),
                        ("opacity", JsonValue::Number(0.5)),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("set opacity");

        assert_eq!(workspace.layers.len(), 1);
        assert_eq!(workspace.layers[0].opacity, 0.5);
        assert_eq!(workspace.history.entries.len(), 2);

        engine.undo(&mut workspace).expect("undo opacity");
        assert_eq!(workspace.layers[0].opacity, 1.0);

        engine.undo(&mut workspace).expect("undo create");
        assert!(workspace.layers.is_empty());
    }

    #[test]
    fn locked_layer_command_rejects_mutation() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));

        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "layer.create",
                    vec![
                        ("id", JsonValue::String("locked".to_owned())),
                        ("name", JsonValue::String("Locked".to_owned())),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("create layer");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "layer.set_locked",
                    vec![
                        ("id", JsonValue::String("locked".to_owned())),
                        ("locked", JsonValue::Bool(true)),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("lock layer");

        let result = engine.execute(
            &mut workspace,
            &registry,
            invocation(
                "layer.rename",
                vec![
                    ("id", JsonValue::String("locked".to_owned())),
                    ("name", JsonValue::String("Nope".to_owned())),
                ],
            ),
            &CommandRuntime::default(),
        );

        assert!(matches!(
            result,
            Err(CommandError::Layer(LayerError::Locked { .. }))
        ));
        assert_eq!(workspace.layers[0].name, "Locked");
    }

    #[test]
    fn image_import_command_is_undoable() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));
        let path = temp_png_path();

        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "image.import_linked",
                    vec![
                        (
                            "path",
                            JsonValue::String(path.to_string_lossy().into_owned()),
                        ),
                        ("asset_id", JsonValue::String("asset".to_owned())),
                        ("object_id", JsonValue::String("object".to_owned())),
                        ("name", JsonValue::String("Imported".to_owned())),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("import image");

        assert_eq!(workspace.assets.len(), 1);
        assert_eq!(workspace.image_objects.len(), 1);
        assert_eq!(
            workspace.assets[0]
                .image_metadata
                .as_ref()
                .map(|metadata| (metadata.width, metadata.height)),
            Some((2, 1))
        );

        engine.undo(&mut workspace).expect("undo import");
        assert!(workspace.assets.is_empty());
        assert!(workspace.image_objects.is_empty());
    }

    #[test]
    fn image_object_commands_replace_duplicate_and_rasterize() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));
        workspace.assets.push(asset("asset-a"));
        workspace.assets.push(asset("asset-b"));

        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "image.place_asset",
                    vec![
                        ("asset_id", JsonValue::String("asset-a".to_owned())),
                        ("object_id", JsonValue::String("object".to_owned())),
                        ("name", JsonValue::String("Placed".to_owned())),
                        ("x", JsonValue::Number(4.0)),
                        ("y", JsonValue::Number(5.0)),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("place");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "image.duplicate_object",
                    vec![
                        ("object_id", JsonValue::String("object".to_owned())),
                        ("new_object_id", JsonValue::String("copy".to_owned())),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("duplicate");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "image.replace_source",
                    vec![
                        ("object_id", JsonValue::String("object".to_owned())),
                        ("asset_id", JsonValue::String("asset-b".to_owned())),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("replace");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "image.rasterize_object",
                    vec![
                        ("object_id", JsonValue::String("object".to_owned())),
                        ("layer_id", JsonValue::String("layer".to_owned())),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("rasterize");

        assert_eq!(workspace.image_objects.len(), 2);
        assert_eq!(workspace.layers.len(), 1);
        assert_eq!(workspace.image_objects[0].source_asset_id, id("asset-b"));
        assert_eq!(
            workspace.image_objects[0].position,
            Point { x: 4.0, y: 5.0 }
        );

        engine.undo(&mut workspace).expect("undo rasterize");
        assert!(workspace.layers.is_empty());
        assert_eq!(workspace.image_objects[0].rasterized_layer_id, None);
    }

    #[test]
    fn export_area_and_output_commands_are_undoable() {
        let registry = default_command_registry().expect("registry");
        let mut engine = CommandEngine::new();
        let mut workspace = Workspace::empty(id("workspace"));

        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "output.add",
                    vec![
                        ("id", JsonValue::String("output".to_owned())),
                        ("filename", JsonValue::String("icon.png".to_owned())),
                        ("format", JsonValue::String("png".to_owned())),
                        ("scale", JsonValue::Number(2.0)),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("add output");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "export_area.create",
                    vec![
                        ("id", JsonValue::String("area".to_owned())),
                        ("name", JsonValue::String("Icon".to_owned())),
                        ("width", JsonValue::Number(32.0)),
                        ("height", JsonValue::Number(32.0)),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("create area");
        engine
            .execute(
                &mut workspace,
                &registry,
                invocation(
                    "export_area.attach_output",
                    vec![
                        ("area_id", JsonValue::String("area".to_owned())),
                        ("output_id", JsonValue::String("output".to_owned())),
                    ],
                ),
                &CommandRuntime::default(),
            )
            .expect("attach output");

        assert_eq!(workspace.outputs.len(), 1);
        assert_eq!(workspace.export_areas.len(), 1);
        assert_eq!(workspace.export_areas[0].output_ids, vec![id("output")]);
        assert_eq!(workspace.history.entries.len(), 3);

        engine.undo(&mut workspace).expect("undo attach");
        assert!(workspace.export_areas[0].output_ids.is_empty());

        engine.undo(&mut workspace).expect("undo area");
        assert!(workspace.export_areas.is_empty());

        engine.undo(&mut workspace).expect("undo output");
        assert!(workspace.outputs.is_empty());
    }

    fn rename_invocation(name: &str) -> CommandInvocation {
        CommandInvocation {
            id: CommandId::new("workspace.rename").expect("command id"),
            parameters: CommandParameters::new(vec![(
                "name".to_owned(),
                JsonValue::String(name.to_owned()),
            )]),
            context: CommandContext::default(),
        }
    }

    fn invocation(id: &str, values: Vec<(&str, JsonValue)>) -> CommandInvocation {
        CommandInvocation {
            id: CommandId::new(id).expect("command id"),
            parameters: CommandParameters::new(
                values
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value)),
            ),
            context: CommandContext::default(),
        }
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }

    fn asset(value: &str) -> Asset {
        Asset {
            id: id(value),
            name: format!("{value}.png"),
            source: AssetSource::Embedded { digest: None },
            media_type: Some("image/png".to_owned()),
            color_profile: None,
            image_metadata: Some(ImageAssetMetadata {
                width: 2,
                height: 1,
                format: Some(ImageFormat::Png),
                color_type: "Rgba8".to_owned(),
                has_alpha: true,
            }),
        }
    }

    fn temp_png_path() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fleck-command-import-{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::write(&path, png_bytes()).expect("write png");
        path
    }

    fn png_bytes() -> Vec<u8> {
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(2, 1, |x, _| {
            if x == 0 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 128])
            }
        });
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, CrateImageFormat::Png)
            .expect("encode png");
        bytes.into_inner()
    }
}
