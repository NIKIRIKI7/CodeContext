use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::manifest::PluginManifest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "widget", rename_all = "snake_case")]
pub enum PluginWidgetNode {
    Section {
        title: String,
        children: Vec<Self>,
    },
    Row {
        children: Vec<Self>,
    },
    Column {
        children: Vec<Self>,
    },
    Label {
        text: String,
        #[serde(default)]
        monospace: bool,
        #[serde(default)]
        muted: bool,
    },
    Badge {
        text: String,
        #[serde(default)]
        tone: String,
    },
    Button {
        id: String,
        label: String,
        #[serde(default)]
        primary: bool,
    },
    Switch {
        id: String,
        label: String,
        checked: bool,
    },
    CodePreview {
        id: String,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginViewSpec {
    pub plugin_id: String,
    pub view_id: String,
    pub title: String,
    pub root: PluginWidgetNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginUiEvent {
    pub plugin_id: String,
    pub view_id: String,
    pub widget_id: String,
    pub action: String,
    pub payload: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub is_built_in: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PluginPortError {
    #[error("Плагин `{0}` не найден или отключён")]
    NotFoundOrDisabled(String),
    #[error("Плагин `{plugin_id}` не имеет разрешения `{permission}`")]
    PermissionDenied { plugin_id: String, permission: String },
    #[error("Ошибка выполнения плагина `{plugin_id}`: {reason}")]
    ExecutionFailed { plugin_id: String, reason: String },
}

pub trait PluginHostPort: Send + Sync {
    fn list_plugins(&self) -> Vec<PluginDescriptor>;

    fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> Result<(), PluginPortError>;

    fn collect_dynamic_context_sections(
        &self,
        project_root: Option<&Path>,
    ) -> Vec<(String, String)>;

    fn transform_file_content(&self, rel_path: &str, original_content: &str) -> Option<String>;

    fn execute_command(
        &self,
        command_id: &str,
        project_root: Option<&Path>,
    ) -> Result<String, PluginPortError>;

    fn render_plugin_view(
        &self,
        plugin_id: &str,
        view_id: &str,
        project_root: Option<&Path>,
    ) -> Result<PluginViewSpec, PluginPortError>;

    fn dispatch_ui_event(
        &self,
        event: &PluginUiEvent,
        project_root: Option<&Path>,
    ) -> Result<PluginViewSpec, PluginPortError>;
}
