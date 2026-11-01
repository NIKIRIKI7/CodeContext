use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupFileEntry {
    pub rel_path: String,
    pub existed_before: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    pub timestamp_ms: u128,
    pub backup_dir: Option<PathBuf>,
    pub applied_files_count: usize,
    pub entries: Vec<BackupFileEntry>,
}

impl BackupManifest {
    #[inline]
    pub const fn new(
        timestamp_ms: u128,
        backup_dir: Option<PathBuf>,
        applied_files_count: usize,
        entries: Vec<BackupFileEntry>,
    ) -> Self {
        Self { timestamp_ms, backup_dir, applied_files_count, entries }
    }
}
