use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use memchr::memchr;

use crate::discovery::binary_detector::BINARY_PROBE_BYTES;

/// Вычитывает текстовый файл с диска, отсекая бинарные файлы по первым 8 КБ.
pub(crate) fn read_text_file(path: &Path) -> Result<Option<String>> {
    let bytes = fs::read(path)
        .with_context(|| format!("Не удалось прочитать файл `{}`", path.display()))?;
    let probe_len = bytes.len().min(BINARY_PROBE_BYTES);
    let probe_slice = bytes.get(..probe_len).unwrap_or(&bytes);
    if memchr(0, probe_slice).is_some() {
        return Ok(None);
    }
    Ok(String::from_utf8(bytes).ok())
}
