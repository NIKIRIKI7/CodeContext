//! Модели каскадного разрешения конфигурации (`ResolvedConfig`, `GlobalConfig`, `ConfigWarning`).

use serde::{Deserialize, Serialize};

use super::preset::Preset;

/// Глобальное сочетание клавиш по умолчанию.
pub const DEFAULT_GLOBAL_HOTKEY: &str = "Ctrl+Shift+C";
/// Канал обновлений по умолчанию.
pub const DEFAULT_UPDATE_CHANNEL: &str = "stable";
/// Канонический репозиторий синхронизации релизов по умолчанию.
pub const DEFAULT_UPDATE_REPO_URL: &str = "https://github.com/NIKIRIKI7/CodeContext.git";

/// Источник происхождения итогового пресета после прохождения каскада.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConfigOrigin {
    /// Использованы встроенные настройки по умолчанию (`Level 0`).
    #[default]
    BuiltInDefault,
    /// Загружен профиль из глобального файла настроек пользователя (`Level 1`).
    GlobalUserConfig,
    /// Наложен локальный файл `.ctxpack.json` из корня проекта (`Level 2`).
    LocalProjectOverride,
}

impl ConfigOrigin {
    /// Возвращает `true`, если конфигурация содержит правила из локального `.ctxpack.json` репозитория.
    #[inline]
    pub const fn is_from_untrusted_project_config(self) -> bool {
        matches!(self, Self::LocalProjectOverride)
    }
}

/// Диагностическое предупреждение, возникшее при чтении, миграции или валидации конфигурации.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigWarning {
    /// Глобальный файл конфигурации повреждён; выполнен откат к встроенным настройкам.
    CorruptedGlobalConfig { path: String, reason: String },
    /// Локальный файл `.ctxpack.json` повреждён; выполнен откат к глобальному профилю.
    CorruptedProjectConfig { path: String, reason: String },
    /// Файл конфигурации превышает допустимый размер (1 МБ) и был отклонён.
    OversizedConfigFile { path: String, size_bytes: u64, max_bytes: u64 },
    /// В `.ctxpack.json` обнаружена попытка разблокировать секреты (заблокировано по Сценарию A3).
    UntrustedSecretUnlockIgnored { blocked_paths: Vec<String> },
    /// Числовой параметр выходил за безопасные границы и был принудительно скорректирован.
    ParameterClamped { field: String, original: String, clamped_to: String },
    /// Базовый пресет, запрошенный в `.ctxpack.json`, не найден в глобальном списке.
    UnknownBasePreset { requested: String, fallback: String },
    /// Файл создан в более новой версии программы (Forward Compatibility).
    NewerSchemaVersion { file_version: u32, supported_version: u32 },
}

impl ConfigWarning {
    /// Возвращает `true`, если предупреждение связано с попыткой обхода защиты секретов.
    #[inline]
    pub const fn is_security_warning(&self) -> bool {
        matches!(self, Self::UntrustedSecretUnlockIgnored { .. })
    }
}

/// Глобальная конфигурация приложения на уровне профиля ОС пользователя (`Level 1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Имя активного пресета по умолчанию.
    pub active_preset_name: String,
    /// Список сохранённых глобальных пресетов пользователя.
    pub presets: Vec<Preset>,
    /// Глобальное сочетание клавиш вызова окна.
    pub global_hotkey: String,
    /// Автоматически ли читать буфер обмена при переходе на вкладку «Применить».
    pub auto_read_clipboard_on_apply: bool,
    /// Создавать ли резервные копии перед применением изменений по умолчанию.
    pub create_backups_by_default: bool,
    /// Выбранный канал обновлений (`stable`, `beta`, `alpha`).
    pub update_channel: String,
    /// URL репозитория для синхронизации обновлений.
    pub update_repo_url: String,
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
        }
    }
}

impl GlobalConfig {
    /// Возвращает активный пресет (или первый доступный / стандартный, если имя не найдено).
    #[inline]
    pub fn active_preset(&self) -> Preset {
        self.find_preset(&self.active_preset_name)
            .or_else(|| self.presets.first())
            .cloned()
            .unwrap_or_else(Preset::default_code_preset)
    }

    /// Ищет пресет по имени без учёта регистра ASCII или по точному совпадению.
    #[inline]
    pub fn find_preset(&self, name: &str) -> Option<&Preset> {
        let trimmed = name.trim();
        self.presets.iter().find(|p| p.name == trimmed || p.name.eq_ignore_ascii_case(trimmed))
    }

    /// Добавляет новый или обновляет существующий пресет с тем же именем.
    pub fn upsert_preset(&mut self, preset: Preset) {
        if let Some(existing) =
            self.presets.iter_mut().find(|p| p.name.eq_ignore_ascii_case(&preset.name))
        {
            *existing = preset;
        } else {
            self.presets.push(preset);
        }
    }

    /// Удаляет пресет по имени, гарантируя, что в списке всегда остаётся минимум один пресет.
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

/// Итоговая проекция каскадного разрешения конфигурации (`Defaults -> Global -> Local`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedConfig {
    /// Эффективный пресет для текущей сессии работы с проектом.
    pub effective_preset: Preset,
    /// Источник итогового пресета.
    pub origin: ConfigOrigin,
    /// Глобальная конфигурация пользователя.
    pub global_config: GlobalConfig,
    /// Все нефатальные предупреждения, собранные при прохождении каскада.
    pub warnings: Vec<ConfigWarning>,
}

impl ResolvedConfig {
    /// Возвращает `true`, если пресет был модифицирован локальным файлом `.ctxpack.json`.
    #[inline]
    pub const fn from_untrusted_project_config(&self) -> bool {
        self.origin.is_from_untrusted_project_config()
    }

    /// Возвращает `true`, если при загрузке были перехвачены попытки разблокировки секретов.
    #[inline]
    pub fn has_security_warnings(&self) -> bool {
        self.warnings.iter().any(ConfigWarning::is_security_warning)
    }
}
