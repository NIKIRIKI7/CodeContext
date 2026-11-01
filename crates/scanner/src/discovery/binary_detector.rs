use core::shared_kernel::fs::file_kind::FileKind;
use memchr::memchr;

/// Стандартный размер префикса файла для быстрой проверки на бинарность при чтении с диска (8 КБ).
pub(crate) const BINARY_PROBE_BYTES: usize = 8192;

/// Проверяет произвольный срез байтов целиком на наличие `NUL`-байта (`\0`) через SIMD (`memchr`).
#[inline]
pub(crate) fn is_binary_slice(bytes: &[u8]) -> bool {
    memchr(0, bytes).is_some()
}

/// Проверяет только первые `BINARY_PROBE_BYTES` (8 КБ) файла.
#[inline]
pub(crate) fn is_binary_probe(bytes: &[u8]) -> bool {
    let probe_len = bytes.len().min(BINARY_PROBE_BYTES);
    let probe_slice = bytes.get(..probe_len).unwrap_or(bytes);
    is_binary_slice(probe_slice)
}

/// Быстрая проверка бинарного расширения файла без чтения с диска (0 аллокаций).
#[inline]
pub(crate) fn is_binary_extension(rel_path: &str) -> bool {
    FileKind::is_known_binary_extension(rel_path)
}
