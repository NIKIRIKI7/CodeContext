use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::parsing::ParsedFile;
use crate::safety::sandbox::{SandboxError, validate_sandbox_path};
use crate::storage::atomic_writer::{AtomicWriteError, write_file_atomically};
use crate::storage::backup::{create_backup_snapshot, restore_latest_backup};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyTransactionReport {
    pub applied_files_count: usize,
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApplyTransactionError {
    #[error("Нет файлов для применения")]
    EmptyPlan,
    #[error(transparent)]
    Sandbox(#[from] SandboxError),
    #[error(transparent)]
    Storage(#[from] AtomicWriteError),
}

pub(crate) fn execute_apply_transaction(
    project_root: &Path,
    files: &[ParsedFile],
    create_backup: bool,
) -> Result<ApplyTransactionReport, ApplyTransactionError> {
    execute_apply_with_deletions(project_root, files, &[], create_backup)
}

pub(crate) fn execute_apply_with_deletions(
    project_root: &Path,
    files_to_write: &[ParsedFile],
    files_to_delete: &[String],
    create_backup: bool,
) -> Result<ApplyTransactionReport, ApplyTransactionError> {
    if files_to_write.is_empty() && files_to_delete.is_empty() {
        return Err(ApplyTransactionError::EmptyPlan);
    }

    let mut validated_writes: Vec<(PathBuf, String, &str)> =
        Vec::with_capacity(files_to_write.len());
    for file in files_to_write {
        let abs_target = validate_sandbox_path(project_root, &file.rel_path)?;
        validated_writes.push((abs_target, file.rel_path.clone(), file.content.as_str()));
    }

    let mut validated_deletes: Vec<(PathBuf, String)> = Vec::with_capacity(files_to_delete.len());
    for del_rel in files_to_delete {
        let abs_target = validate_sandbox_path(project_root, del_rel)?;
        validated_deletes.push((abs_target, del_rel.clone()));
    }

    let backup_dir = if create_backup {
        let mut all_rel_paths: Vec<String> =
            Vec::with_capacity(validated_writes.len() + validated_deletes.len());
        for (_, rel, _) in &validated_writes {
            all_rel_paths.push(rel.clone());
        }
        for (_, rel) in &validated_deletes {
            if !all_rel_paths.contains(rel) {
                all_rel_paths.push(rel.clone());
            }
        }
        Some(create_backup_snapshot(project_root, &all_rel_paths)?)
    } else {
        None
    };

    for (abs_target, _, content) in &validated_writes {
        write_file_atomically(abs_target, content)?;
    }

    for (abs_target, _) in &validated_deletes {
        if abs_target.exists() && abs_target.is_file() {
            let _ = fs::remove_file(abs_target);
        }
    }

    Ok(ApplyTransactionReport {
        applied_files_count: validated_writes.len() + validated_deletes.len(),
        backup_dir,
    })
}

#[inline]
pub(crate) fn execute_undo_transaction(project_root: &Path) -> Result<usize, String> {
    restore_latest_backup(project_root)
}
