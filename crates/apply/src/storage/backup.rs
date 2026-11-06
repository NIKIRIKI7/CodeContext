use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::atomic_writer::{AtomicWriteError, write_file_atomically};

const BACKUPS_REL_DIR: &str = ".ctxpack/backups";
const MANIFEST_FILE_NAME: &str = "manifest.json";
const SNAPSHOT_FILES_SUBDIR: &str = "files";

static BACKUP_SEQ_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupSnapshotInfo {
    pub snapshot_id: String,
    pub timestamp_ms: u128,
    pub files_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BackupFileEntry {
    rel_path: String,
    existed_before: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BackupSnapshotManifest {
    timestamp_ms: u128,
    entries: Vec<BackupFileEntry>,
}

pub(crate) fn create_backup_snapshot(
    project_root: &Path,
    rel_paths: &[String],
) -> Result<PathBuf, AtomicWriteError> {
    let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let seq = BACKUP_SEQ_COUNTER.fetch_add(1, Ordering::Relaxed);
    let folder_name = format!("{timestamp_ms:015}_{seq:04}");
    let backup_dir = project_root.join(BACKUPS_REL_DIR).join(folder_name);
    let files_dir = backup_dir.join(SNAPSHOT_FILES_SUBDIR);

    fs::create_dir_all(&files_dir).map_err(|err| AtomicWriteError::CreateDirectory {
        path: files_dir.display().to_string(),
        reason: err.to_string(),
    })?;

    let mut entries = Vec::with_capacity(rel_paths.len());
    for rel_path in rel_paths {
        let orig_path = project_root.join(rel_path);
        let existed_before = orig_path.exists() && orig_path.is_file();
        if existed_before && let Ok(orig_content) = fs::read_to_string(&orig_path) {
            let backup_file_path = files_dir.join(rel_path);
            if let Some(parent) = backup_file_path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent).map_err(|err| AtomicWriteError::CreateDirectory {
                    path: parent.display().to_string(),
                    reason: err.to_string(),
                })?;
            }
            fs::write(&backup_file_path, orig_content.as_bytes()).map_err(|err| {
                AtomicWriteError::TempWrite {
                    path: backup_file_path.display().to_string(),
                    reason: err.to_string(),
                }
            })?;
        }
        entries.push(BackupFileEntry { rel_path: rel_path.clone(), existed_before });
    }

    let manifest = BackupSnapshotManifest { timestamp_ms, entries };
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap_or_default();
    let manifest_path = backup_dir.join(MANIFEST_FILE_NAME);
    fs::write(&manifest_path, manifest_json.as_bytes()).map_err(|err| {
        AtomicWriteError::TempWrite {
            path: manifest_path.display().to_string(),
            reason: err.to_string(),
        }
    })?;

    Ok(backup_dir)
}

pub(crate) fn restore_latest_backup(project_root: &Path) -> Result<usize, String> {
    let backups_root = project_root.join(BACKUPS_REL_DIR);
    if !backups_root.exists() {
        return Err("Папка бэкапов `.ctxpack/backups` не найдена".to_owned());
    }

    let mut subdirs: Vec<PathBuf> = fs::read_dir(&backups_root)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join(MANIFEST_FILE_NAME).exists())
        .collect();

    subdirs.sort();
    let Some(latest_dir) = subdirs.pop() else {
        return Err("Нет доступных снимков бэкапа для отката".to_owned());
    };

    let manifest_raw =
        fs::read_to_string(latest_dir.join(MANIFEST_FILE_NAME)).map_err(|e| e.to_string())?;
    let manifest: BackupSnapshotManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    let files_dir = latest_dir.join(SNAPSHOT_FILES_SUBDIR);

    let mut restored_count = 0_usize;
    for entry in &manifest.entries {
        let target_path = project_root.join(&entry.rel_path);
        if entry.existed_before {
            let snap_file = files_dir.join(&entry.rel_path);
            if let Ok(orig_content) = fs::read_to_string(&snap_file) {
                write_file_atomically(&target_path, &orig_content).map_err(|e| e.to_string())?;
                restored_count += 1;
            }
        } else if target_path.exists() {
            let _ = fs::remove_file(&target_path);
            restored_count += 1;
        }
    }

    let _ = fs::remove_dir_all(&latest_dir);
    Ok(restored_count)
}

pub fn list_backup_snapshots(project_root: &Path) -> Vec<BackupSnapshotInfo> {
    let backups_root = project_root.join(BACKUPS_REL_DIR);
    let Ok(read_dir) = fs::read_dir(&backups_root) else {
        return Vec::new();
    };
    let mut subdirs: Vec<PathBuf> = read_dir
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join(MANIFEST_FILE_NAME).exists())
        .collect();
    subdirs.sort();
    subdirs.reverse();

    let mut out = Vec::with_capacity(subdirs.len().min(8));
    for dir in subdirs.into_iter().take(8) {
        let snapshot_id = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_owned();
        if let Ok(raw) = fs::read_to_string(dir.join(MANIFEST_FILE_NAME))
            && let Ok(manifest) = serde_json::from_str::<BackupSnapshotManifest>(&raw)
        {
            let size_bytes = compute_dir_size_bytes(&dir);
            out.push(BackupSnapshotInfo {
                snapshot_id,
                timestamp_ms: manifest.timestamp_ms,
                files_count: manifest.entries.len(),
                size_bytes,
            });
        }
    }
    out
}

pub fn restore_backup_by_id(project_root: &Path, snapshot_id: &str) -> Result<usize, String> {
    if snapshot_id.is_empty() || snapshot_id.contains("..") || snapshot_id.contains(['/', '\\']) {
        return Err("Некорректный идентификатор снимка бэкапа".to_owned());
    }
    let target_dir = project_root.join(BACKUPS_REL_DIR).join(snapshot_id);
    if !target_dir.exists() {
        return Err("Снимок бэкапа не найден".to_owned());
    }
    let manifest_raw =
        fs::read_to_string(target_dir.join(MANIFEST_FILE_NAME)).map_err(|e| e.to_string())?;
    let manifest: BackupSnapshotManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    let files_dir = target_dir.join(SNAPSHOT_FILES_SUBDIR);

    let mut restored_count = 0_usize;
    for entry in &manifest.entries {
        let target_path = project_root.join(&entry.rel_path);
        if entry.existed_before {
            let snap_file = files_dir.join(&entry.rel_path);
            if let Ok(orig_content) = fs::read_to_string(&snap_file) {
                write_file_atomically(&target_path, &orig_content).map_err(|e| e.to_string())?;
                restored_count += 1;
            }
        } else if target_path.exists() {
            let _ = fs::remove_file(&target_path);
            restored_count += 1;
        }
    }
    let _ = fs::remove_dir_all(&target_dir);
    Ok(restored_count)
}

pub fn purge_all_backups(project_root: &Path) -> u64 {
    let backups_root = project_root.join(BACKUPS_REL_DIR);
    if !backups_root.exists() {
        return 0;
    }
    let freed = compute_dir_size_bytes(&backups_root);
    let _ = fs::remove_dir_all(&backups_root);
    freed
}

pub fn compute_dir_size_bytes(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0_u64;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            total = total.saturating_add(compute_dir_size_bytes(&p));
        } else if let Ok(meta) = entry.metadata() {
            total = total.saturating_add(meta.len());
        }
    }
    total
}
