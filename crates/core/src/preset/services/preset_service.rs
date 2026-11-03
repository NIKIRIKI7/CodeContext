use std::path::Path;

use crate::preset::models::preset::Preset;
use crate::preset::models::resolution::{ConfigWarning, GlobalConfig, ResolvedConfig};
use crate::preset::ports::config_repo::{ConfigPortError, ConfigRepositoryPort};

#[derive(Debug, Clone)]
pub struct PresetService<R> {
    repo: R,
}

impl<R> PresetService<R>
where
    R: ConfigRepositoryPort,
{
    #[inline]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    #[inline]
    pub const fn repo(&self) -> &R {
        &self.repo
    }

    #[inline]
    pub fn resolve_project_config(&self, project_root: Option<&Path>) -> ResolvedConfig {
        self.repo.resolve_for_project(project_root)
    }

    #[inline]
    pub fn resolve_headless(&self, project_root: &Path) -> (Preset, bool, Vec<ConfigWarning>) {
        self.repo.resolve_headless_preset(project_root)
    }

    #[inline]
    pub fn load_global_config(&self) -> GlobalConfig {
        self.repo.load_global().map_or_else(|_| GlobalConfig::default(), |(cfg, _)| cfg)
    }

    #[inline]
    pub fn save_global_config(&self, config: &GlobalConfig) -> Result<(), ConfigPortError> {
        self.repo.save_global(config)
    }

    pub fn toggle_auto_read_clipboard(&self) -> Result<GlobalConfig, ConfigPortError> {
        let mut global = self.load_global_config();
        global.auto_read_clipboard_on_apply = !global.auto_read_clipboard_on_apply;
        self.repo.save_global(&global)?;
        Ok(global)
    }

    pub fn toggle_create_backups_by_default(&self) -> Result<GlobalConfig, ConfigPortError> {
        let mut global = self.load_global_config();
        global.create_backups_by_default = !global.create_backups_by_default;
        self.repo.save_global(&global)?;
        Ok(global)
    }

    pub fn set_update_channel(&self, channel: &str) -> Result<GlobalConfig, ConfigPortError> {
        let mut global = self.load_global_config();
        channel.trim().clone_into(&mut global.update_channel);
        self.repo.save_global(&global)?;
        Ok(global)
    }

    pub fn record_recent_project(
        &self,
        project_root: &Path,
    ) -> Result<GlobalConfig, ConfigPortError> {
        let mut global = self.load_global_config();
        let path_str = project_root.to_string_lossy();
        if global.record_recent_project(&path_str) {
            self.repo.save_global(&global)?;
        }
        Ok(global)
    }

    pub fn switch_active_preset(&self, preset_name: &str) -> Result<GlobalConfig, ConfigPortError> {
        let (mut global, _) = self.repo.load_global().unwrap_or_default();
        if let Some(found_name) = global.find_preset(preset_name).map(|f| f.name.clone()) {
            global.active_preset_name = found_name;
        } else {
            preset_name.trim().clone_into(&mut global.active_preset_name);
        }
        self.repo.save_global(&global)?;
        Ok(global)
    }

    pub fn save_preset_globally(
        &self,
        preset: Preset,
        make_active: bool,
    ) -> Result<GlobalConfig, ConfigPortError> {
        let (mut global, _) = self.repo.load_global().unwrap_or_default();
        if make_active {
            global.active_preset_name.clone_from(&preset.name);
        }
        global.upsert_preset(preset);
        self.repo.save_global(&global)?;
        Ok(global)
    }

    pub fn delete_global_preset(&self, preset_name: &str) -> Result<GlobalConfig, ConfigPortError> {
        let (mut global, _) = self.repo.load_global().unwrap_or_default();
        global.remove_preset(preset_name);
        self.repo.save_global(&global)?;
        Ok(global)
    }

    pub fn save_preset_to_project(
        &self,
        project_root: &Path,
        preset: &Preset,
    ) -> Result<(), ConfigPortError> {
        let sanitized = preset.clone().sanitized_from_untrusted_project_config();
        self.repo.save_project_preset(project_root, &sanitized)
    }

    pub fn import_and_register_preset(
        &self,
        file_path: &Path,
        make_active: bool,
    ) -> Result<(Preset, GlobalConfig, Vec<ConfigWarning>), ConfigPortError> {
        let (preset, warnings) = self.repo.import_preset(file_path)?;
        let updated_global = self.save_preset_globally(preset.clone(), make_active)?;
        Ok((preset, updated_global, warnings))
    }

    pub fn export_preset_to_file(
        &self,
        preset: &Preset,
        file_path: &Path,
    ) -> Result<(), ConfigPortError> {
        let sanitized = preset.clone().sanitized_from_untrusted_project_config();
        self.repo.export_preset(&sanitized, file_path)
    }
}
