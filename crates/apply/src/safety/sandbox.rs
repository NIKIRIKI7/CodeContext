use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SandboxError {
    #[error("Путь пуст или содержит запрещённый NUL-байт")]
    EmptyOrNulPath,
    #[error("Абсолютные и сетевые UNC-пути запрещены в песочнице: `{0}`")]
    AbsoluteOrUncPath(String),
    #[error("Префиксы дисков Windows и альтернативные потоки NTFS (ADS) запрещены: `{0}`")]
    DriveOrAdsForbidden(String),
    #[error("Обнаружена попытка выхода за корень проекта (Path Traversal `..`): `{0}`")]
    PathTraversalDetected(String),
    #[error("Симлинк ведёт за пределы корня проекта: `{0}`")]
    SymlinkEscapeDetected(String),
}

pub(crate) fn validate_sandbox_path(
    project_root: &Path,
    candidate_rel_path: &str,
) -> Result<PathBuf, SandboxError> {
    validate_sandbox_path_with_fs_check(project_root, candidate_rel_path, true)
}

pub(crate) fn validate_sandbox_path_with_fs_check(
    project_root: &Path,
    candidate_rel_path: &str,
    check_symlinks_on_disk: bool,
) -> Result<PathBuf, SandboxError> {
    let trimmed = candidate_rel_path.trim();
    if trimmed.is_empty() || candidate_rel_path.contains('\0') {
        return Err(SandboxError::EmptyOrNulPath);
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return Err(SandboxError::AbsoluteOrUncPath(candidate_rel_path.to_owned()));
    }
    if trimmed.contains(':') {
        return Err(SandboxError::DriveOrAdsForbidden(candidate_rel_path.to_owned()));
    }

    let mut has_normal_segment = false;
    let mut has_cur_dir_or_empty = false;
    for seg in trimmed.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            has_cur_dir_or_empty = true;
            continue;
        }
        if seg == ".." {
            return Err(SandboxError::PathTraversalDetected(candidate_rel_path.to_owned()));
        }
        if seg.trim().is_empty() {
            return Err(SandboxError::EmptyOrNulPath);
        }
        has_normal_segment = true;
    }
    if !has_normal_segment {
        return Err(SandboxError::EmptyOrNulPath);
    }
    let clean_rel = if !has_cur_dir_or_empty && !trimmed.contains('\\') {
        let path = Path::new(trimmed);
        if path.is_absolute() {
            return Err(SandboxError::AbsoluteOrUncPath(candidate_rel_path.to_owned()));
        }
        let mut buf = PathBuf::with_capacity(
            project_root.as_os_str().len().saturating_add(trimmed.len()).saturating_add(2),
        );
        buf.push(project_root);
        buf.push(trimmed);
        buf
    } else {
        let normalized: Cow<'_, str> = if trimmed.contains('\\') {
            Cow::Owned(trimmed.replace('\\', "/"))
        } else {
            Cow::Borrowed(trimmed)
        };
        let path = Path::new(normalized.as_ref());
        if path.is_absolute() {
            return Err(SandboxError::AbsoluteOrUncPath(candidate_rel_path.to_owned()));
        }
        let mut buf = PathBuf::with_capacity(
            project_root.as_os_str().len().saturating_add(normalized.len()).saturating_add(16),
        );
        buf.push(project_root);
        for component in path.components() {
            match component {
                Component::Normal(part) => buf.push(part),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(SandboxError::PathTraversalDetected(candidate_rel_path.to_owned()));
                }
            }
        }
        buf
    };

    let is_synthetic_unix_root_on_windows =
        cfg!(windows) && project_root.as_os_str().as_encoded_bytes().first().copied() == Some(b'/');

    if check_symlinks_on_disk
        && !is_synthetic_unix_root_on_windows
        && let Ok(meta) = clean_rel.symlink_metadata()
        && meta.file_type().is_symlink()
        && let Ok(canon_target) = clean_rel.canonicalize()
        && let Ok(canon_root) = project_root.canonicalize()
        && !canon_target.starts_with(&canon_root)
    {
        return Err(SandboxError::SymlinkEscapeDetected(candidate_rel_path.to_owned()));
    }

    Ok(clean_rel)
}
