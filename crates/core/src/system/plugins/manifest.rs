use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    WorkspaceSelection,
    FsReadProject,
    FsProposePatch,
    ClipboardAccess,
    TerminalPropose,
    NetworkOutbound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationEvent {
    OnStartup,
    OnPackContext,
    OnParseResponse,
    OnAfterApply,
    OnCommand(String),
    OnView(String),
    OnProjectOpened(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCommandSpec {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub default_shortcut: Option<String>,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginViewContribution {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub location: PluginViewLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginViewLocation {
    #[default]
    ActivityRail,
    BottomDock,
    ModalOverlay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginContextProviderSpec {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub header_tag: String,
    #[serde(default)]
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCodeTransformSpec {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub target_extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributionPoints {
    #[serde(default)]
    pub commands: Vec<PluginCommandSpec>,
    #[serde(default)]
    pub views: Vec<PluginViewContribution>,
    #[serde(default)]
    pub context_providers: Vec<PluginContextProviderSpec>,
    #[serde(default)]
    pub code_transforms: Vec<PluginCodeTransformSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub activation_events: Vec<ActivationEvent>,
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub contributes: ContributionPoints,
}

impl PluginManifest {
    #[inline]
    pub fn has_permission(&self, permission: PluginPermission) -> bool {
        self.permissions.contains(&permission)
    }

    #[inline]
    pub fn activates_on(&self, event: &ActivationEvent) -> bool {
        self.activation_events.iter().any(|e| e == event || matches!(e, ActivationEvent::OnStartup))
    }
}
