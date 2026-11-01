//! Value Object сегмента упакованного буфера (`PackChunk`).

use serde::{Deserialize, Serialize};

/// Неделимый сегмент упакованного контекста, готовый к копированию в буфер обмена.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackChunk {
    /// Порядковый индекс сегмента (начиная с `0`).
    pub index: usize,
    /// Общее количество сегментов в результате упаковки.
    pub total: usize,
    /// Сформированный текст сегмента.
    pub content: String,
    /// Количество файлов, вошедших в данный сегмент.
    pub file_count: usize,
}

impl PackChunk {
    /// Создаёт новый неизменяемый сегмент упакованного контекста.
    #[inline]
    pub const fn new(index: usize, total: usize, content: String, file_count: usize) -> Self {
        Self { index, total, content, file_count }
    }

    /// Возвращает точный размер сегмента в байтах.
    #[inline]
    pub const fn size_bytes(&self) -> usize {
        self.content.len()
    }

    /// Возвращает `true`, если сегмент не содержит текста.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}
