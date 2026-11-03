use serde::{Deserialize, Serialize};

use super::preset::Preset;

pub const DEFAULT_GLOBAL_HOTKEY: &str = "Ctrl+K";
pub const DEFAULT_UPDATE_CHANNEL: &str = "stable";
pub const DEFAULT_UPDATE_REPO_URL: &str = "https://github.com/NIKIRIKI7/CodeContext.git";
pub const MAX_RECENT_PROJECTS: usize = 8;

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
}

impl KeymapAction {
    pub const ALL: [Self; 8] = [
        Self::GlobalHotkey,
        Self::OpenProject,
        Self::SearchFiles,
        Self::RescanProject,
        Self::CopyContext,
        Self::PasteResponse,
        Self::ApplyChanges,
        Self::UndoChanges,
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
    pub update_channel: String,
    pub update_repo_url: String,
    #[serde(default)]
    pub recent_projects: Vec<String>,
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
            update_channel: DEFAULT_UPDATE_CHANNEL.to_owned(),
            update_repo_url: DEFAULT_UPDATE_REPO_URL.to_owned(),
            recent_projects: Vec::new(),
            keymap: KeymapConfig::default(),
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
