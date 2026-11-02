use std::fs::File;
use std::io::Read as _;
use std::path::Path;

use anyhow::{Context, Result};
use memchr::memchr;

use crate::discovery::binary_detector::BINARY_PROBE_BYTES;

pub(crate) fn read_text_file(path: &Path) -> Result<Option<String>> {
    let mut file = File::open(path).with_context(|| path.display().to_string())?;
    let meta = file.metadata().with_context(|| path.display().to_string())?;
    let Ok(file_len) = usize::try_from(meta.len()) else {
        return Ok(None);
    };
    if file_len == 0 {
        return Ok(Some(String::new()));
    }

    let mut probe = [0_u8; BINARY_PROBE_BYTES];
    let probe_to_read = file_len.min(BINARY_PROBE_BYTES);
    let probe_buf = probe
        .get_mut(..probe_to_read)
        .context("Invalid probe buffer slice")?;
    file.read_exact(probe_buf)
        .with_context(|| path.display().to_string())?;

    if memchr(0, probe_buf).is_some() {
        return Ok(None);
    }

    let mut bytes = Vec::with_capacity(file_len);
    bytes.extend_from_slice(probe_buf);
    file.read_to_end(&mut bytes)
        .with_context(|| path.display().to_string())?;

    Ok(String::from_utf8(bytes).ok())
}
