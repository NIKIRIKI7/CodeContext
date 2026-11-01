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

    let normalized: Cow<'_, str> = if trimmed.contains('\\') {
        Cow::Owned(trimmed.replace('\\', "/"))
    } else {
        Cow::Borrowed(trimmed)
    };

    let path = Path::new(normalized.as_ref());
    if path.is_absolute() {
        return Err(SandboxError::AbsoluteOrUncPath(candidate_rel_path.to_owned()));
    }

    let mut clean_rel = PathBuf::with_capacity(
        project_root.as_os_str().len().saturating_add(normalized.len()).saturating_add(16),
    );
    clean_rel.push(project_root);

    let mut has_normal_segment = false;
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                if part.is_empty() || part.to_str().is_some_and(|s| s.trim().is_empty()) {
                    return Err(SandboxError::EmptyOrNulPath);
                }
                clean_rel.push(part);
                has_normal_segment = true;
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(SandboxError::PathTraversalDetected(candidate_rel_path.to_owned()));
            }
        }
    }

    if !has_normal_segment {
        return Err(SandboxError::EmptyOrNulPath);
    }

    let is_synthetic_unix_root_on_windows =
        cfg!(windows) && project_root.as_os_str().as_encoded_bytes().first().copied() == Some(b'/');

    if check_symlinks_on_disk
        && !is_synthetic_unix_root_on_windows
        && clean_rel.symlink_metadata().is_ok()
        && let Ok(canon_target) = clean_rel.canonicalize()
        && let Ok(canon_root) = project_root.canonicalize()
        && !canon_target.starts_with(&canon_root)
    {
        return Err(SandboxError::SymlinkEscapeDetected(candidate_rel_path.to_owned()));
    }

    Ok(clean_rel)
}
