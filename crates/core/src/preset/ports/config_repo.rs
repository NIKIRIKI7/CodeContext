//! Порт репозитория пресетов и каскадной конфигурации (`ConfigRepositoryPort`).

use std::path::Path;
use thiserror::Error;

use crate::preset::models::preset::Preset;
use crate::preset::models::resolution::{ConfigWarning, GlobalConfig, ResolvedConfig};

/// Ошибки операций чтения, импорта и сохранения конфигурации.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigPortError {
    #[error("Ошибка ввода-вывода при доступе к файлу конфигурации `{path}`: {reason}")]
    IoError { path: String, reason: String },

    #[error(
        "Файл конфигурации `{path}` превышает допустимый размер ({size_bytes} > {max_bytes} байт)"
    )]
    FileTooLarge { path: String, size_bytes: u64, max_bytes: u64 },

    #[error("Некорректный формат JSON в файле `{path}`: {reason}")]
    InvalidFormat { path: String, reason: String },

    #[error("Ошибка миграции схемы в файле `{path}` (версия {version}): {reason}")]
    MigrationFailed { path: String, version: u32, reason: String },
}

/// Доменный порт каскадной загрузки и сохранения конфигурации.
pub trait ConfigRepositoryPort: Send + Sync {
    /// Разрешает каскад `Defaults -> Global -> Local (.ctxpack.json)` с плавной деградацией
    /// (Graceful Degradation: ошибки чтения локального или глобального файла превращаются в `ConfigWarning`).
    fn resolve_for_project(&self, project_root: Option<&Path>) -> ResolvedConfig;

    /// Быстрый путь (`Fast-Path`) для режима `--pack-headless`: возвращает только итоговый `Preset`,
    /// флаг `from_untrusted_project_config` и список предупреждений.
    fn resolve_headless_preset(&self, project_root: &Path) -> (Preset, bool, Vec<ConfigWarning>);

    /// Вычитывает глобальную конфигурацию пользователя.
    fn load_global(&self) -> Result<(GlobalConfig, Vec<ConfigWarning>), ConfigPortError>;

    /// Атомарно сохраняет глобальную конфигурацию пользователя на диск.
    fn save_global(&self, config: &GlobalConfig) -> Result<(), ConfigPortError>;

    /// Атомарно сохраняет пресет проекта в `<project_root>/.ctxpack.json`.
    fn save_project_preset(
        &self,
        project_root: &Path,
        preset: &Preset,
    ) -> Result<(), ConfigPortError>;

    /// Импортирует пресет из внешнего JSON-файла.
    fn import_preset(
        &self,
        file_path: &Path,
    ) -> Result<(Preset, Vec<ConfigWarning>), ConfigPortError>;

    /// Экспортирует пресет во внешний JSON-файл.
    fn export_preset(&self, preset: &Preset, file_path: &Path) -> Result<(), ConfigPortError>;
}
