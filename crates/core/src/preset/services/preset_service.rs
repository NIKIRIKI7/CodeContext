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

#[cfg(test)]
#[allow(clippy::assigning_clones, clippy::assert_is_empty)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;

    use super::PresetService;
    use crate::preset::models::preset::Preset;
    use crate::preset::models::resolution::{
        ConfigOrigin, ConfigWarning, GlobalConfig, ResolvedConfig,
    };
    use crate::preset::ports::config_repo::{ConfigPortError, ConfigRepositoryPort};

    #[derive(Debug, Default)]
    struct InMemoryConfigRepo {
        global: Mutex<GlobalConfig>,
        project_preset: Mutex<Option<Preset>>,
    }

    impl ConfigRepositoryPort for InMemoryConfigRepo {
        fn resolve_for_project(&self, _project_root: Option<&Path>) -> ResolvedConfig {
            let global_config = self.global.lock().unwrap().clone();
            if let Some(proj) = self.project_preset.lock().unwrap().clone() {
                ResolvedConfig {
                    effective_preset: proj.sanitized_from_untrusted_project_config(),
                    origin: ConfigOrigin::LocalProjectOverride,
                    global_config,
                    warnings: Vec::new(),
                }
            } else {
                ResolvedConfig {
                    effective_preset: global_config.active_preset(),
                    origin: ConfigOrigin::GlobalUserConfig,
                    global_config,
                    warnings: Vec::new(),
                }
            }
        }

        fn resolve_headless_preset(
            &self,
            project_root: &Path,
        ) -> (Preset, bool, Vec<ConfigWarning>) {
            let resolved = self.resolve_for_project(Some(project_root));
            let from_untrusted = resolved.from_untrusted_project_config();
            (resolved.effective_preset, from_untrusted, resolved.warnings)
        }

        fn load_global(&self) -> Result<(GlobalConfig, Vec<ConfigWarning>), ConfigPortError> {
            Ok((self.global.lock().unwrap().clone(), Vec::new()))
        }

        fn save_global(&self, config: &GlobalConfig) -> Result<(), ConfigPortError> {
            config.clone_into(&mut self.global.lock().unwrap());
            Ok(())
        }

        fn save_project_preset(
            &self,
            _project_root: &Path,
            preset: &Preset,
        ) -> Result<(), ConfigPortError> {
            *self.project_preset.lock().unwrap() = Some(preset.clone());
            Ok(())
        }

        fn import_preset(
            &self,
            _file_path: &Path,
        ) -> Result<(Preset, Vec<ConfigWarning>), ConfigPortError> {
            let imported = Preset { name: "Imported".to_owned(), ..Preset::default_code_preset() };
            Ok((imported, Vec::new()))
        }

        fn export_preset(&self, preset: &Preset, _file_path: &Path) -> Result<(), ConfigPortError> {
            *self.project_preset.lock().unwrap() = Some(preset.clone());
            Ok(())
        }
    }

    #[test]
    fn saving_project_preset_strips_session_secrets() {
        let service = PresetService::new(InMemoryConfigRepo::default());
        let mut preset = Preset::default_code_preset();
        preset.explicitly_allowed_secrets.push(".env".to_owned());

        service.save_preset_to_project(Path::new("/workspace"), &preset).unwrap();
        let saved = service.repo().project_preset.lock().unwrap().clone().unwrap();
        assert!(saved.explicitly_allowed_secrets.is_empty());
    }

    #[test]
    fn switching_and_deleting_presets_maintains_invariants() {
        let service = PresetService::new(InMemoryConfigRepo::default());
        let custom = Preset {
            name: "RustBackend".to_owned(),
            minify_code: true,
            ..Preset::default_code_preset()
        };
        let updated = service.save_preset_globally(custom, true).unwrap();
        assert_eq!(updated.active_preset_name, "RustBackend");
        assert_eq!(updated.presets.len(), 2);

        let after_delete = service.delete_global_preset("RustBackend").unwrap();
        assert_eq!(after_delete.presets.len(), 1);
        assert_eq!(after_delete.active_preset_name, "Default");
    }
}
