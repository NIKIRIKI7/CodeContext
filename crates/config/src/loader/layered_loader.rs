//! Каскадная загрузка: `Defaults -> Global Config -> Project .ctxpack.json`.

use std::fs::{self, File};
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};

use core::preset::models::preset::{
    GLOBAL_CONFIG_FILE_NAME, MAX_CONFIG_FILE_BYTES, PROJECT_CONFIG_FILE_NAME, Preset,
};
use core::preset::models::resolution::{ConfigOrigin, ConfigWarning, GlobalConfig, ResolvedConfig};
use core::preset::ports::config_repo::{ConfigPortError, ConfigRepositoryPort};

use crate::schema::dtos::{GlobalConfigDto, ProjectOverlayDto};
use crate::schema::mappers::{
    apply_project_overlay, global_domain_to_dto, global_dto_to_domain,
    preset_domain_to_project_overlay_dto,
};
use crate::schema::migration::parse_and_migrate_json;

const APP_CONFIG_DIR_NAME: &str = "CodeContext";

/// Инфраструктурный адаптер каскадного чтения и атомарного сохранения конфигурации.
#[derive(Debug, Clone)]
pub struct LayeredConfigLoader {
    global_config_path: PathBuf,
}

impl Default for LayeredConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl LayeredConfigLoader {
    /// Создаёт загрузчик со стандартным путём глобальной конфигурации для текущей ОС (`dirs`).
    #[inline]
    pub fn new() -> Self {
        let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let global_config_path = base_dir.join(APP_CONFIG_DIR_NAME).join(GLOBAL_CONFIG_FILE_NAME);
        Self { global_config_path }
    }

    /// Создаёт загрузчик с явно заданным путём глобального конфига
    /// (для изоляции в тестах и бенчмарках без использования запрещённого `std::env::set_var`).
    #[inline]
    pub const fn with_global_config_path(global_config_path: PathBuf) -> Self {
        Self { global_config_path }
    }

    /// Возвращает путь к глобальному файлу настроек пользователя.
    #[inline]
    pub fn global_config_path(&self) -> &Path {
        &self.global_config_path
    }

    /// Вычисляет путь к локальному файлу `.ctxpack.json` внутри корня проекта.
    #[inline]
    pub fn project_config_path(project_root: &Path) -> PathBuf {
        project_root.join(PROJECT_CONFIG_FILE_NAME)
    }

    fn load_global_layer(&self, warnings: &mut Vec<ConfigWarning>) -> (GlobalConfig, ConfigOrigin) {
        match read_bounded_file(&self.global_config_path) {
            Ok(None) => (GlobalConfig::default(), ConfigOrigin::BuiltInDefault),
            Ok(Some(raw_json)) => {
                match parse_and_migrate_json::<GlobalConfigDto>(&raw_json, warnings) {
                    Ok(dto) => {
                        (global_dto_to_domain(dto, warnings), ConfigOrigin::GlobalUserConfig)
                    }
                    Err(reason) => {
                        warnings.push(ConfigWarning::CorruptedGlobalConfig {
                            path: self.global_config_path.display().to_string(),
                            reason,
                        });
                        (GlobalConfig::default(), ConfigOrigin::BuiltInDefault)
                    }
                }
            }
            Err(ConfigPortError::FileTooLarge { path, size_bytes, max_bytes }) => {
                warnings.push(ConfigWarning::OversizedConfigFile { path, size_bytes, max_bytes });
                (GlobalConfig::default(), ConfigOrigin::BuiltInDefault)
            }
            Err(err) => {
                warnings.push(ConfigWarning::CorruptedGlobalConfig {
                    path: self.global_config_path.display().to_string(),
                    reason: err.to_string(),
                });
                (GlobalConfig::default(), ConfigOrigin::BuiltInDefault)
            }
        }
    }
}

impl ConfigRepositoryPort for LayeredConfigLoader {
    fn resolve_for_project(&self, project_root: Option<&Path>) -> ResolvedConfig {
        let mut warnings = Vec::new();
        let (global_config, base_origin) = self.load_global_layer(&mut warnings);

        let (effective_preset, origin) = if let Some(root) = project_root {
            apply_project_layer(root, &global_config, base_origin, &mut warnings)
        } else {
            (global_config.active_preset(), base_origin)
        };

        ResolvedConfig { effective_preset, origin, global_config, warnings }
    }

    fn resolve_headless_preset(&self, project_root: &Path) -> (Preset, bool, Vec<ConfigWarning>) {
        let resolved = self.resolve_for_project(Some(project_root));
        let from_untrusted = resolved.from_untrusted_project_config();
        (resolved.effective_preset, from_untrusted, resolved.warnings)
    }

    fn load_global(&self) -> Result<(GlobalConfig, Vec<ConfigWarning>), ConfigPortError> {
        let Some(raw_json) = read_bounded_file(&self.global_config_path)? else {
            return Ok((GlobalConfig::default(), Vec::new()));
        };
        let mut warnings = Vec::new();
        let dto: GlobalConfigDto =
            parse_and_migrate_json(&raw_json, &mut warnings).map_err(|reason| {
                ConfigPortError::InvalidFormat {
                    path: self.global_config_path.display().to_string(),
                    reason,
                }
            })?;
        Ok((global_dto_to_domain(dto, &mut warnings), warnings))
    }

    fn save_global(&self, config: &GlobalConfig) -> Result<(), ConfigPortError> {
        let dto = global_domain_to_dto(config);
        write_json_atomically(&self.global_config_path, &dto)
    }

    fn save_project_preset(
        &self,
        project_root: &Path,
        preset: &Preset,
    ) -> Result<(), ConfigPortError> {
        let target_path = Self::project_config_path(project_root);
        let sanitized = preset.clone().sanitized_from_untrusted_project_config();
        let dto = preset_domain_to_project_overlay_dto(&sanitized);
        write_json_atomically(&target_path, &dto)
    }

    fn import_preset(
        &self,
        file_path: &Path,
    ) -> Result<(Preset, Vec<ConfigWarning>), ConfigPortError> {
        let raw_json = read_bounded_file(file_path)?.ok_or_else(|| ConfigPortError::IoError {
            path: file_path.display().to_string(),
            reason: "Файл пресета не найден".to_owned(),
        })?;
        let mut warnings = Vec::new();
        let overlay: ProjectOverlayDto =
            parse_and_migrate_json(&raw_json, &mut warnings).map_err(|reason| {
                ConfigPortError::InvalidFormat { path: file_path.display().to_string(), reason }
            })?;
        let base = Preset::default_code_preset();
        let preset = apply_project_overlay(&base, overlay, &mut warnings);
        Ok((preset, warnings))
    }

    fn export_preset(&self, preset: &Preset, file_path: &Path) -> Result<(), ConfigPortError> {
        let sanitized = preset.clone().sanitized_from_untrusted_project_config();
        let dto = preset_domain_to_project_overlay_dto(&sanitized);
        write_json_atomically(file_path, &dto)
    }
}

fn apply_project_layer(
    project_root: &Path,
    global_config: &GlobalConfig,
    base_origin: ConfigOrigin,
    warnings: &mut Vec<ConfigWarning>,
) -> (Preset, ConfigOrigin) {
    let local_path = LayeredConfigLoader::project_config_path(project_root);
    let raw_opt = match read_bounded_file(&local_path) {
        Ok(opt) => opt,
        Err(ConfigPortError::FileTooLarge { path, size_bytes, max_bytes }) => {
            warnings.push(ConfigWarning::OversizedConfigFile { path, size_bytes, max_bytes });
            return (global_config.active_preset(), base_origin);
        }
        Err(err) => {
            warnings.push(ConfigWarning::CorruptedProjectConfig {
                path: local_path.display().to_string(),
                reason: err.to_string(),
            });
            return (global_config.active_preset(), base_origin);
        }
    };

    let Some(raw_json) = raw_opt else {
        return (global_config.active_preset(), base_origin);
    };

    match parse_and_migrate_json::<ProjectOverlayDto>(&raw_json, warnings) {
        Ok(overlay) => {
            let base_preset = resolve_base_preset_for_overlay(
                global_config,
                overlay.base_preset.as_deref(),
                warnings,
            );
            let effective = apply_project_overlay(&base_preset, overlay, warnings);
            (effective, ConfigOrigin::LocalProjectOverride)
        }
        Err(reason) => {
            warnings.push(ConfigWarning::CorruptedProjectConfig {
                path: local_path.display().to_string(),
                reason,
            });
            (global_config.active_preset(), base_origin)
        }
    }
}

fn resolve_base_preset_for_overlay(
    global_config: &GlobalConfig,
    requested_base: Option<&str>,
    warnings: &mut Vec<ConfigWarning>,
) -> Preset {
    let Some(base_name) = requested_base.map(str::trim).filter(|s| !s.is_empty()) else {
        return global_config.active_preset();
    };

    if let Some(found) = global_config.find_preset(base_name) {
        found.clone()
    } else {
        let fallback = global_config.active_preset();
        warnings.push(ConfigWarning::UnknownBasePreset {
            requested: base_name.to_owned(),
            fallback: fallback.name.clone(),
        });
        fallback
    }
}

/// Читает файл конфигурации с проверкой верхнего лимита `MAX_CONFIG_FILE_BYTES` (1 МБ)
/// до выделения памяти под буфер строки.
fn read_bounded_file(path: &Path) -> Result<Option<String>, ConfigPortError> {
    let metadata = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(ConfigPortError::IoError {
                path: path.display().to_string(),
                reason: err.to_string(),
            });
        }
    };

    if !metadata.is_file() {
        return Err(ConfigPortError::IoError {
            path: path.display().to_string(),
            reason: "Ожидался обычный файл, но обнаружена директория".to_owned(),
        });
    }

    let size_bytes = metadata.len();
    if size_bytes > MAX_CONFIG_FILE_BYTES {
        return Err(ConfigPortError::FileTooLarge {
            path: path.display().to_string(),
            size_bytes,
            max_bytes: MAX_CONFIG_FILE_BYTES,
        });
    }

    fs::read_to_string(path).map(Some).map_err(|err| ConfigPortError::IoError {
        path: path.display().to_string(),
        reason: err.to_string(),
    })
}

/// Атомарно сериализует и записывает структуру в JSON-файл через временный файл и `fs::rename`.
fn write_json_atomically<T>(target_path: &Path, value: &T) -> Result<(), ConfigPortError>
where
    T: serde::Serialize,
{
    if let Some(parent) = target_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| ConfigPortError::IoError {
            path: parent.display().to_string(),
            reason: err.to_string(),
        })?;
    }

    let mut json_bytes =
        serde_json::to_vec_pretty(value).map_err(|err| ConfigPortError::InvalidFormat {
            path: target_path.display().to_string(),
            reason: err.to_string(),
        })?;
    json_bytes.push(b'\n');

    let file_name = target_path.file_name().and_then(|n| n.to_str()).unwrap_or("config.json");
    let tmp_path = target_path.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));

    {
        let mut tmp_file = File::create(&tmp_path).map_err(|err| ConfigPortError::IoError {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
        tmp_file.write_all(&json_bytes).map_err(|err| ConfigPortError::IoError {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
        tmp_file.sync_all().map_err(|err| ConfigPortError::IoError {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
    }

    if let Err(err) = fs::rename(&tmp_path, target_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(ConfigPortError::IoError {
            path: target_path.display().to_string(),
            reason: err.to_string(),
        });
    }

    Ok(())
}
