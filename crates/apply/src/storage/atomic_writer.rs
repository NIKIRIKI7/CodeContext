use std::fs::{self, File};
use std::io::Write as _;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AtomicWriteError {
    #[error("Не удалось создать родительскую директорию `{path}`: {reason}")]
    CreateDirectory { path: String, reason: String },

    #[error("Не удалось записать временный файл `{path}`: {reason}")]
    TempWrite { path: String, reason: String },

    #[error("Не удалось атомарно переименовать `{from}` -> `{to}`: {reason}")]
    Rename { from: String, to: String, reason: String },
}

pub(crate) fn write_file_atomically(
    target_path: &Path,
    content: &str,
) -> Result<(), AtomicWriteError> {
    if let Some(parent) = target_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| AtomicWriteError::CreateDirectory {
            path: parent.display().to_string(),
            reason: err.to_string(),
        })?;
    }

    let file_name = target_path.file_name().and_then(|n| n.to_str()).unwrap_or("patch_target");
    let pid = std::process::id();
    let tmp_path = target_path.with_file_name(format!(".{file_name}.{pid}.tmp"));

    {
        let mut tmp_file = File::create(&tmp_path).map_err(|err| AtomicWriteError::TempWrite {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
        tmp_file.write_all(content.as_bytes()).map_err(|err| AtomicWriteError::TempWrite {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
        tmp_file.sync_all().map_err(|err| AtomicWriteError::TempWrite {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
    }

    if let Err(err) = fs::rename(&tmp_path, target_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(AtomicWriteError::Rename {
            from: tmp_path.display().to_string(),
            to: target_path.display().to_string(),
            reason: err.to_string(),
        });
    }

    Ok(())
}
