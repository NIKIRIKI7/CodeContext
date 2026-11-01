//! Проекция (Read Model) результата упаковки контекста (`PackResult`).

use serde::{Deserialize, Serialize};

use crate::packing::values::pack_chunk::PackChunk;
use crate::shared_kernel::execution::anomaly::Anomaly;

/// Итоговая проекция завершённой упаковки контекста.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackResult {
    /// Сформированные сегменты упакованного буфера (минимум 1 сегмент).
    pub chunks: Vec<PackChunk>,
    /// Общий размер всех сегментов в байтах.
    pub total_bytes: usize,
    /// Общее количество упакованных файлов.
    pub packed_files_count: usize,
    /// Количество заблокированных файлов с секретами.
    pub blocked_secrets_count: usize,
    /// Количество отсечённых бинарных файлов.
    pub skipped_binary_count: usize,
    /// Все диагностические предупреждения и аномалии.
    pub anomalies: Vec<Anomaly>,
}

impl PackResult {
    /// Создаёт новую проекцию результата упаковки.
    #[inline]
    pub fn new(
        chunks: Vec<PackChunk>,
        packed_files_count: usize,
        blocked_secrets_count: usize,
        skipped_binary_count: usize,
        anomalies: Vec<Anomaly>,
    ) -> Self {
        let total_bytes = chunks.iter().map(PackChunk::size_bytes).sum();
        Self {
            chunks,
            total_bytes,
            packed_files_count,
            blocked_secrets_count,
            skipped_binary_count,
            anomalies,
        }
    }

    /// Возвращает текст первого (или единственного) сегмента для быстрого копирования и предпросмотра.
    #[inline]
    pub fn primary_text(&self) -> &str {
        self.chunks.first().map_or("", |c| c.content.as_str())
    }

    /// Возвращает `true`, если результат был разбит на несколько томов.
    #[inline]
    pub fn is_chunked(&self) -> bool {
        self.chunks.len() > 1
    }
}
