use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_SHELL_ROOT_LABEL: &str = "CodeContext";
pub const DEFAULT_SHELL_PACK_LABEL_RU: &str = "Скопировать (с .gitignore)";
pub const DEFAULT_SHELL_APPLY_LABEL_RU: &str = "Применить патч (автофикс)";
pub const DEFAULT_SHELL_OPEN_LABEL_RU: &str = "Открыть в GUI";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShellRegistrationStatus {
    #[default]
    NotRegistered,
    RegisteredHealthy,
    StaleExecutablePath {
        registered_command: String,
    },
}

impl ShellRegistrationStatus {
    #[inline]
    pub const fn is_registered(&self) -> bool {
        matches!(self, Self::RegisteredHealthy)
    }

    #[inline]
    pub const fn is_stale(&self) -> bool {
        matches!(self, Self::StaleExecutablePath { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellMenuConfig {
    pub executable_path: PathBuf,
    pub root_menu_label: String,
    pub pack_menu_label: String,
    pub apply_menu_label: String,
    pub open_gui_label: String,
}

impl Default for ShellMenuConfig {
    fn default() -> Self {
        let executable_path =
            std::env::current_exe().unwrap_or_else(|_| PathBuf::from("codecontext"));
        Self {
            executable_path,
            root_menu_label: DEFAULT_SHELL_ROOT_LABEL.to_owned(),
            pack_menu_label: DEFAULT_SHELL_PACK_LABEL_RU.to_owned(),
            apply_menu_label: DEFAULT_SHELL_APPLY_LABEL_RU.to_owned(),
            open_gui_label: DEFAULT_SHELL_OPEN_LABEL_RU.to_owned(),
        }
    }
}

impl ShellMenuConfig {
    #[inline]
    pub fn with_executable(executable_path: PathBuf) -> Self {
        Self { executable_path, ..Self::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ShellIntegrationError {
    #[error("Не удалось определить путь к исполняемому файлу приложения: {0}")]
    ExecutableResolutionFailed(String),

    #[error("Не удалось зарегистрировать контекстное меню ОС: {0}")]
    RegistrationFailed(String),

    #[error("Не удалось удалить контекстное меню ОС: {0}")]
    UnregistrationFailed(String),
}

pub trait ShellIntegrationPort: Send + Sync {
    fn status(&self) -> ShellRegistrationStatus {
        if self.is_registered() {
            ShellRegistrationStatus::RegisteredHealthy
        } else {
            ShellRegistrationStatus::NotRegistered
        }
    }

    fn status_for_config(&self, _config: &ShellMenuConfig) -> ShellRegistrationStatus {
        self.status()
    }

    fn is_registered(&self) -> bool;

    fn register_context_menu(&self) -> Result<(), ShellIntegrationError>;

    fn register_with_config(&self, _config: &ShellMenuConfig) -> Result<(), ShellIntegrationError> {
        self.register_context_menu()
    }

    fn unregister_context_menu(&self) -> Result<(), ShellIntegrationError>;
}
