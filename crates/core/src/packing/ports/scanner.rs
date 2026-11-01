//! Доменный порт сканера файловой системы (`FileScannerPort`).

use std::path::Path;

use thiserror::Error;

use crate::preset::models::preset::Preset;
use crate::shared_kernel::execution::anomaly::Anomaly;
use crate::shared_kernel::execution::cancel_token::CancelToken;

/// Ошибки инфраструктурного адаптера сканирования файловой системы.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScannerPortError {
    #[error("Корневая директория проекта не существует или недоступна: `{0}`")]
    InvalidProjectRoot(String),

    #[error("Ошибка компиляции glob-правила фильтрации `{pattern}`: {reason}")]
    InvalidGlobPattern { pattern: String, reason: String },

    #[error("Операция сканирования была отменена пользователем")]
    Cancelled,

    #[error("Ошибка ввода-вывода при чтении проекта: {0}")]
    IoError(String),
}

/// Сырые метаданные файла, обнаруженного обходчиком ФС (до доменной классификации `SecretGuard`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawScannedEntry {
    /// Нормализованный относительный путь (разделитель `/`).
    pub rel_path: String,
    /// Размер файла в байтах.
    pub size_bytes: u64,
    /// Глубина вложенности относительно корня проекта.
    pub depth: usize,
}

/// Результат Фазы 1 (параллельный обход индекса ФС).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanWalkReport {
    /// Отсортированный список найденных кандидатов.
    pub entries: Vec<RawScannedEntry>,
    /// Аномалии обхода (симлинки, превышение глубины или лимита файлов).
    pub anomalies: Vec<Anomaly>,
}

/// Вычитанный с диска текстовый файл.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedTextFile {
    /// Относительный путь файла.
    pub rel_path: String,
    /// Проверенное UTF-8 содержимое файла.
    pub content: String,
}

/// Результат Фазы 2 (параллельное чтение и фильтрация бинарников по 8 КБ `NUL`-пробе).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HydrateReport {
    /// Успешно прочитанные текстовые файлы.
    pub files: Vec<LoadedTextFile>,
    /// Число файлов, отсечённых как бинарные при проверке первых 8 КБ.
    pub skipped_binaries: usize,
    /// Аномалии чтения файлов.
    pub anomalies: Vec<Anomaly>,
}

/// Доменный порт параллельного сканера файловой системы.
pub trait FileScannerPort: Send + Sync {
    /// Фаза 1: Параллельно обходит дерево проекта с учётом `.gitignore` и возвращает индекс метаданных.
    fn scan_index(
        &self,
        project_root: &Path,
        preset: &Preset,
        cancel_token: &CancelToken,
    ) -> Result<ScanWalkReport, ScannerPortError>;

    /// Фаза 2: Параллельно вычитывает содержимое выбранных текстовых файлов, отсекая бинарные по 8 КБ пробе.
    fn read_selected_files(
        &self,
        project_root: &Path,
        selected_rel_paths: &[String],
        cancel_token: &CancelToken,
    ) -> Result<HydrateReport, ScannerPortError>;
}
