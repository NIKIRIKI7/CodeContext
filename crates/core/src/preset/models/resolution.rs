use serde::{Deserialize, Serialize};

use super::preset::Preset;

pub const DEFAULT_GLOBAL_HOTKEY: &str = "Ctrl+K";
pub const DEFAULT_UPDATE_CHANNEL: &str = "stable";
pub const DEFAULT_UPDATE_REPO_URL: &str = "https://github.com/NIKIRIKI7/CodeContext.git";
pub const MAX_RECENT_PROJECTS: usize = 8;

fn default_toggle_terminal_shortcut() -> String {
    "Ctrl+Backquote".to_owned()
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalTaskCategory {
    #[default]
    Git,
    Build,
    Test,
    Custom,
}

impl TerminalTaskCategory {
    pub const ALL: [Self; 4] = [Self::Git, Self::Build, Self::Test, Self::Custom];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCommandPreset {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub category: TerminalTaskCategory,
    #[serde(default)]
    pub is_custom: bool,
}

pub fn default_terminal_command_presets() -> Vec<TerminalCommandPreset> {
    vec![
        TerminalCommandPreset {
            id: "git_status".to_owned(),
            name: "git status -s".to_owned(),
            command: "git status -s".to_owned(),
            category: TerminalTaskCategory::Git,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "git_diff_stat".to_owned(),
            name: "git diff --stat".to_owned(),
            command: "git diff --stat".to_owned(),
            category: TerminalTaskCategory::Git,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "git_log_5".to_owned(),
            name: "git log -n 5".to_owned(),
            command: "git log -n 5 --oneline".to_owned(),
            category: TerminalTaskCategory::Git,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "cargo_check".to_owned(),
            name: "cargo check".to_owned(),
            command: "cargo check --workspace".to_owned(),
            category: TerminalTaskCategory::Build,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "cargo_build_rel".to_owned(),
            name: "cargo build --release".to_owned(),
            command: "cargo build --release".to_owned(),
            category: TerminalTaskCategory::Build,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "cargo_clippy".to_owned(),
            name: "cargo clippy".to_owned(),
            command: "cargo clippy --workspace --all-targets -- -D warnings".to_owned(),
            category: TerminalTaskCategory::Test,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "cargo_test".to_owned(),
            name: "cargo test".to_owned(),
            command: "cargo test --workspace".to_owned(),
            category: TerminalTaskCategory::Test,
            is_custom: false,
        },
        TerminalCommandPreset {
            id: "cargo_fmt_check".to_owned(),
            name: "cargo fmt --check".to_owned(),
            command: "cargo fmt --all -- --check".to_owned(),
            category: TerminalTaskCategory::Test,
            is_custom: false,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditionProfile {
    #[default]
    Full,
    Lite,
    Custom,
}

impl EditionProfile {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Lite => "lite",
            Self::Custom => "custom",
        }
    }

    #[inline]
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.eq_ignore_ascii_case("lite") {
            Self::Lite
        } else if trimmed.eq_ignore_ascii_case("custom") {
            Self::Custom
        } else {
            Self::Full
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlag {
    ChangeApplier,
    SystemPrompts,
    AdvancedFilters,
    TerminalRunner,
    TelemetryLogs,
    Plugins,
}

impl FeatureFlag {
    pub const ALL: [Self; 6] = [
        Self::ChangeApplier,
        Self::SystemPrompts,
        Self::AdvancedFilters,
        Self::TerminalRunner,
        Self::TelemetryLogs,
        Self::Plugins,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub enable_change_applier: bool,
    #[serde(default = "default_true")]
    pub enable_system_prompts: bool,
    #[serde(default = "default_true")]
    pub enable_advanced_filters: bool,
    #[serde(default = "default_true")]
    pub enable_terminal_runner: bool,
    #[serde(default = "default_true")]
    pub enable_telemetry_logs: bool,
    #[serde(default = "default_true")]
    pub enable_plugins: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::full()
    }
}

impl FeatureFlags {
    #[inline]
    pub const fn full() -> Self {
        Self {
            enable_change_applier: true,
            enable_system_prompts: true,
            enable_advanced_filters: true,
            enable_terminal_runner: true,
            enable_telemetry_logs: true,
            enable_plugins: true,
        }
    }

    #[inline]
    pub const fn lite() -> Self {
        Self {
            enable_change_applier: true,
            enable_system_prompts: true,
            enable_advanced_filters: true,
            enable_terminal_runner: false,
            enable_telemetry_logs: false,
            enable_plugins: false,
        }
    }

    #[inline]
    pub const fn is_enabled(&self, flag: FeatureFlag) -> bool {
        match flag {
            FeatureFlag::ChangeApplier => self.enable_change_applier,
            FeatureFlag::SystemPrompts => self.enable_system_prompts,
            FeatureFlag::AdvancedFilters => self.enable_advanced_filters,
            FeatureFlag::TerminalRunner => self.enable_terminal_runner,
            FeatureFlag::TelemetryLogs => self.enable_telemetry_logs,
            FeatureFlag::Plugins => self.enable_plugins,
        }
    }

    #[inline]
    pub const fn toggle(&mut self, flag: FeatureFlag) -> bool {
        match flag {
            FeatureFlag::ChangeApplier => {
                self.enable_change_applier = !self.enable_change_applier;
                self.enable_change_applier
            }
            FeatureFlag::SystemPrompts => {
                self.enable_system_prompts = !self.enable_system_prompts;
                self.enable_system_prompts
            }
            FeatureFlag::AdvancedFilters => {
                self.enable_advanced_filters = !self.enable_advanced_filters;
                self.enable_advanced_filters
            }
            FeatureFlag::TerminalRunner => {
                self.enable_terminal_runner = !self.enable_terminal_runner;
                self.enable_terminal_runner
            }
            FeatureFlag::TelemetryLogs => {
                self.enable_telemetry_logs = !self.enable_telemetry_logs;
                self.enable_telemetry_logs
            }
            FeatureFlag::Plugins => {
                self.enable_plugins = !self.enable_plugins;
                self.enable_plugins
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceChromeConfig {
    #[serde(default = "default_true")]
    pub show_activity_rail: bool,
    #[serde(default = "default_true")]
    pub show_top_bar: bool,
    #[serde(default = "default_true")]
    pub show_options_footer: bool,
    #[serde(default = "default_true")]
    pub show_file_sizes: bool,
    #[serde(default = "default_true")]
    pub show_size_heatmap: bool,
}

impl Default for WorkspaceChromeConfig {
    fn default() -> Self {
        Self {
            show_activity_rail: true,
            show_top_bar: true,
            show_options_footer: true,
            show_file_sizes: true,
            show_size_heatmap: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UiChromeSetting {
    ActivityRail,
    TopBar,
    OptionsFooter,
    FileSizes,
    SizeHeatmap,
    BottomBar,
    TerminalButton,
    LogsButton,
    StatusMetrics,
    TerminalPresets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiChromeConfig {
    #[serde(default = "default_true")]
    pub show_bottom_bar: bool,
    #[serde(default = "default_true")]
    pub show_terminal_button: bool,
    #[serde(default = "default_true")]
    pub show_logs_button: bool,
    #[serde(default = "default_true")]
    pub show_status_metrics: bool,
    #[serde(default = "default_true")]
    pub show_terminal_presets: bool,
    #[serde(default)]
    pub workspace: WorkspaceChromeConfig,
}

impl Default for UiChromeConfig {
    fn default() -> Self {
        Self {
            show_bottom_bar: true,
            show_terminal_button: true,
            show_logs_button: true,
            show_status_metrics: true,
            show_terminal_presets: true,
            workspace: WorkspaceChromeConfig::default(),
        }
    }
}

impl UiChromeConfig {
    #[inline]
    pub const fn lite() -> Self {
        Self {
            show_bottom_bar: false,
            show_terminal_button: false,
            show_logs_button: false,
            show_status_metrics: true,
            show_terminal_presets: false,
            workspace: WorkspaceChromeConfig {
                show_activity_rail: true,
                show_top_bar: true,
                show_options_footer: true,
                show_file_sizes: true,
                show_size_heatmap: true,
            },
        }
    }

    #[inline]
    pub const fn is_enabled(&self, setting: UiChromeSetting) -> bool {
        match setting {
            UiChromeSetting::ActivityRail => self.workspace.show_activity_rail,
            UiChromeSetting::TopBar => self.workspace.show_top_bar,
            UiChromeSetting::OptionsFooter => self.workspace.show_options_footer,
            UiChromeSetting::FileSizes => self.workspace.show_file_sizes,
            UiChromeSetting::SizeHeatmap => self.workspace.show_size_heatmap,
            UiChromeSetting::BottomBar => self.show_bottom_bar,
            UiChromeSetting::TerminalButton => self.show_terminal_button,
            UiChromeSetting::LogsButton => self.show_logs_button,
            UiChromeSetting::StatusMetrics => self.show_status_metrics,
            UiChromeSetting::TerminalPresets => self.show_terminal_presets,
        }
    }

    #[inline]
    pub const fn toggle(&mut self, setting: UiChromeSetting) -> bool {
        match setting {
            UiChromeSetting::ActivityRail => {
                self.workspace.show_activity_rail = !self.workspace.show_activity_rail;
                self.workspace.show_activity_rail
            }
            UiChromeSetting::TopBar => {
                self.workspace.show_top_bar = !self.workspace.show_top_bar;
                self.workspace.show_top_bar
            }
            UiChromeSetting::OptionsFooter => {
                self.workspace.show_options_footer = !self.workspace.show_options_footer;
                self.workspace.show_options_footer
            }
            UiChromeSetting::FileSizes => {
                self.workspace.show_file_sizes = !self.workspace.show_file_sizes;
                self.workspace.show_file_sizes
            }
            UiChromeSetting::SizeHeatmap => {
                self.workspace.show_size_heatmap = !self.workspace.show_size_heatmap;
                self.workspace.show_size_heatmap
            }
            UiChromeSetting::BottomBar => {
                self.show_bottom_bar = !self.show_bottom_bar;
                self.show_bottom_bar
            }
            UiChromeSetting::TerminalButton => {
                self.show_terminal_button = !self.show_terminal_button;
                self.show_terminal_button
            }
            UiChromeSetting::LogsButton => {
                self.show_logs_button = !self.show_logs_button;
                self.show_logs_button
            }
            UiChromeSetting::StatusMetrics => {
                self.show_status_metrics = !self.show_status_metrics;
                self.show_status_metrics
            }
            UiChromeSetting::TerminalPresets => {
                self.show_terminal_presets = !self.show_terminal_presets;
                self.show_terminal_presets
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeymapAction {
    GlobalHotkey,
    OpenProject,
    SearchFiles,
    RescanProject,
    CopyContext,
    PasteResponse,
    ApplyChanges,
    UndoChanges,
    ToggleTerminal,
}

impl KeymapAction {
    pub const ALL: [Self; 9] = [
        Self::GlobalHotkey,
        Self::OpenProject,
        Self::SearchFiles,
        Self::RescanProject,
        Self::CopyContext,
        Self::PasteResponse,
        Self::ApplyChanges,
        Self::UndoChanges,
        Self::ToggleTerminal,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeymapConfig {
    pub open_project: String,
    pub search_files: String,
    pub rescan_project: String,
    pub copy_context: String,
    pub paste_response: String,
    pub apply_changes: String,
    pub undo_changes: String,
    #[serde(default = "default_toggle_terminal_shortcut")]
    pub toggle_terminal: String,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            open_project: "Ctrl+O".to_owned(),
            search_files: "Ctrl+F".to_owned(),
            rescan_project: "Ctrl+R".to_owned(),
            copy_context: "Ctrl+C".to_owned(),
            paste_response: "Ctrl+V".to_owned(),
            apply_changes: "Ctrl+Enter".to_owned(),
            undo_changes: "Ctrl+Z".to_owned(),
            toggle_terminal: default_toggle_terminal_shortcut(),
        }
    }
}

impl KeymapConfig {
    #[inline]
    pub fn shortcut_for<'a>(&'a self, action: KeymapAction, global_hotkey: &'a str) -> &'a str {
        match action {
            KeymapAction::GlobalHotkey => global_hotkey,
            KeymapAction::OpenProject => &self.open_project,
            KeymapAction::SearchFiles => &self.search_files,
            KeymapAction::RescanProject => &self.rescan_project,
            KeymapAction::CopyContext => &self.copy_context,
            KeymapAction::PasteResponse => &self.paste_response,
            KeymapAction::ApplyChanges => &self.apply_changes,
            KeymapAction::UndoChanges => &self.undo_changes,
            KeymapAction::ToggleTerminal => &self.toggle_terminal,
        }
    }

    #[inline]
    pub fn set_shortcut(&mut self, action: KeymapAction, canonical: String) {
        match action {
            KeymapAction::GlobalHotkey => {}
            KeymapAction::OpenProject => self.open_project = canonical,
            KeymapAction::SearchFiles => self.search_files = canonical,
            KeymapAction::RescanProject => self.rescan_project = canonical,
            KeymapAction::CopyContext => self.copy_context = canonical,
            KeymapAction::PasteResponse => self.paste_response = canonical,
            KeymapAction::ApplyChanges => self.apply_changes = canonical,
            KeymapAction::UndoChanges => self.undo_changes = canonical,
            KeymapAction::ToggleTerminal => self.toggle_terminal = canonical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConfigOrigin {
    #[default]
    BuiltInDefault,
    GlobalUserConfig,
    LocalProjectOverride,
}

impl ConfigOrigin {
    #[inline]
    pub const fn is_from_untrusted_project_config(self) -> bool {
        matches!(self, Self::LocalProjectOverride)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigWarning {
    CorruptedGlobalConfig { path: String, reason: String },
    CorruptedProjectConfig { path: String, reason: String },
    OversizedConfigFile { path: String, size_bytes: u64, max_bytes: u64 },
    UntrustedSecretUnlockIgnored { blocked_paths: Vec<String> },
    ParameterClamped { field: String, original: String, clamped_to: String },
    UnknownBasePreset { requested: String, fallback: String },
    NewerSchemaVersion { file_version: u32, supported_version: u32 },
}

impl ConfigWarning {
    #[inline]
    pub const fn is_security_warning(&self) -> bool {
        matches!(self, Self::UntrustedSecretUnlockIgnored { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub active_preset_name: String,
    pub presets: Vec<Preset>,
    pub global_hotkey: String,
    pub auto_read_clipboard_on_apply: bool,
    pub create_backups_by_default: bool,
    #[serde(default = "default_true")]
    pub show_bottom_bar: bool,
    #[serde(default)]
    pub edition: EditionProfile,
    #[serde(default)]
    pub features: FeatureFlags,
    #[serde(default)]
    pub ui_chrome: UiChromeConfig,
    #[serde(default = "default_terminal_command_presets")]
    pub terminal_presets: Vec<TerminalCommandPreset>,
    pub update_channel: String,
    pub update_repo_url: String,
    #[serde(default)]
    pub recent_projects: Vec<String>,
    #[serde(default)]
    pub enabled_plugins: Vec<String>,
    #[serde(default)]
    pub keymap: KeymapConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let default_preset = Preset::default_code_preset();
        Self {
            active_preset_name: default_preset.name.clone(),
            presets: vec![default_preset],
            global_hotkey: DEFAULT_GLOBAL_HOTKEY.to_owned(),
            auto_read_clipboard_on_apply: false,
            create_backups_by_default: true,
            show_bottom_bar: true,
            edition: EditionProfile::Full,
            features: FeatureFlags::full(),
            ui_chrome: UiChromeConfig::default(),
            terminal_presets: default_terminal_command_presets(),
            update_channel: DEFAULT_UPDATE_CHANNEL.to_owned(),
            update_repo_url: DEFAULT_UPDATE_REPO_URL.to_owned(),
            recent_projects: Vec::new(),
            enabled_plugins: Vec::new(),
            keymap: KeymapConfig::default(),
        }
    }
}

impl GlobalConfig {
    pub fn apply_edition_profile(&mut self, profile: EditionProfile) {
        self.edition = profile;
        match profile {
            EditionProfile::Full => {
                self.features = FeatureFlags::full();
                self.ui_chrome = UiChromeConfig::default();
                self.show_bottom_bar = true;
            }
            EditionProfile::Lite => {
                self.features = FeatureFlags::lite();
                self.ui_chrome = UiChromeConfig::lite();
                self.show_bottom_bar = false;
            }
            EditionProfile::Custom => {}
        }
    }
}

impl GlobalConfig {
    #[inline]
    pub fn active_preset(&self) -> Preset {
        self.find_preset(&self.active_preset_name)
            .or_else(|| self.presets.first())
            .cloned()
            .unwrap_or_else(Preset::default_code_preset)
    }

    #[inline]
    pub fn find_preset(&self, name: &str) -> Option<&Preset> {
        let trimmed = name.trim();
        self.presets.iter().find(|p| p.name == trimmed || p.name.eq_ignore_ascii_case(trimmed))
    }

    pub fn record_recent_project(&mut self, project_path: &str) -> bool {
        let normalized = normalize_recent_project_path(project_path);
        if normalized.is_empty() {
            return false;
        }
        if self
            .recent_projects
            .first()
            .is_some_and(|first| paths_are_equivalent(first, normalized) && first == normalized)
        {
            return false;
        }
        self.recent_projects.retain(|existing| !paths_are_equivalent(existing, normalized));
        self.recent_projects.insert(0, normalized.to_owned());
        self.recent_projects.truncate(MAX_RECENT_PROJECTS);
        true
    }

    pub fn upsert_preset(&mut self, preset: Preset) {
        if let Some(existing) =
            self.presets.iter_mut().find(|p| p.name.eq_ignore_ascii_case(&preset.name))
        {
            *existing = preset;
        } else {
            self.presets.push(preset);
        }
    }

    pub fn remove_preset(&mut self, name: &str) -> bool {
        let before = self.presets.len();
        self.presets.retain(|p| !p.name.eq_ignore_ascii_case(name));
        let removed = self.presets.len() < before;

        if self.presets.is_empty() {
            let fallback = Preset::default_code_preset();
            self.active_preset_name.clone_from(&fallback.name);
            self.presets.push(fallback);
        } else if self.find_preset(&self.active_preset_name).is_none()
            && let Some(first) = self.presets.first()
        {
            self.active_preset_name.clone_from(&first.name);
        }
        removed
    }
}

#[inline]
pub fn normalize_recent_project_path(raw: &str) -> &str {
    let trimmed = raw.trim();
    let without_verbatim = trimmed.strip_prefix(r"\\?\").unwrap_or(trimmed);
    let stripped = without_verbatim.trim_end_matches(['/', '\\']);
    if stripped.is_empty() { without_verbatim } else { stripped }
}

#[inline]
pub fn paths_are_equivalent(a: &str, b: &str) -> bool {
    let na = normalize_recent_project_path(a);
    let nb = normalize_recent_project_path(b);
    let mut chars_a = na.chars();
    let mut chars_b = nb.chars();
    loop {
        match (chars_a.next(), chars_b.next()) {
            (None, None) => return true,
            (Some(ca), Some(cb)) => {
                if (ca == '/' || ca == '\\') && (cb == '/' || cb == '\\') {
                    continue;
                }
                let same = if cfg!(windows) { ca.eq_ignore_ascii_case(&cb) } else { ca == cb };
                if !same {
                    return false;
                }
            }
            _ => return false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedConfig {
    pub effective_preset: Preset,
    pub origin: ConfigOrigin,
    pub global_config: GlobalConfig,
    pub warnings: Vec<ConfigWarning>,
}

impl ResolvedConfig {
    #[inline]
    pub const fn from_untrusted_project_config(&self) -> bool {
        self.origin.is_from_untrusted_project_config()
    }

    #[inline]
    pub fn has_security_warnings(&self) -> bool {
        self.warnings.iter().any(ConfigWarning::is_security_warning)
    }
}
