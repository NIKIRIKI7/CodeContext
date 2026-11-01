//! Диагностические аномалии обхода ФС и разбора ответов.

use serde::{Deserialize, Serialize};

/// Нефатальная аномалия, обнаруженная при сканировании или упаковке проекта.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anomaly {
    /// Файл заблокирован защитником секретов (`SecretGuard`).
    SecretBlocked { rel_path: String, reason: String },
    /// Размер файла превышает настроенный порог предупреждения.
    LargeFileWarning { rel_path: String, size_bytes: u64, threshold_bytes: u64 },
    /// Превышена максимальная глубина вложенности директорий.
    MaxDepthExceeded { rel_path: String, depth: usize, max_depth: usize },
    /// Превышен лимит общего количества файлов в проекте.
    MaxFilesExceeded { discovered_files: usize, max_files: usize },
    /// Пропущена символическая ссылка или обнаружен цикл симлинков.
    CyclicOrSkippedSymlink { rel_path: String },
    /// Файл не удалось прочитать или он содержит невалидный UTF-8.
    UnreadableOrInvalidUtf8 { rel_path: String, reason: String },
}

impl Anomaly {
    /// Возвращает относительный путь файла, связанного с аномалией (если применимо).
    #[inline]
    pub fn rel_path(&self) -> Option<&str> {
        match self {
            Self::SecretBlocked { rel_path, .. }
            | Self::LargeFileWarning { rel_path, .. }
            | Self::MaxDepthExceeded { rel_path, .. }
            | Self::CyclicOrSkippedSymlink { rel_path }
            | Self::UnreadableOrInvalidUtf8 { rel_path, .. } => Some(rel_path.as_str()),
            Self::MaxFilesExceeded { .. } => None,
        }
    }

    /// Возвращает `true`, если аномалия связана с безопасностью (`SecretGuard`).
    #[inline]
    pub const fn is_security_warning(&self) -> bool {
        matches!(self, Self::SecretBlocked { .. })
    }
}
