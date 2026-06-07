use crate::model::{HistoryEntry, HistoryState, JsonValue, ObjectId, ValidationError, Workspace};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    use crate::model::Workspace;

    #[test]
    fn registry_lists_command_definitions() {
        let registry = default_command_registry().expect("registry");
        let definitions = registry.definitions().collect::<Vec<_>>();

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].id.as_str(), "workspace.rename");
        assert!(definitions[0].undoable);
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

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }
}
