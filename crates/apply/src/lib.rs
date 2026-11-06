mod parsing;
mod patching;
mod safety;
mod storage;

pub use parsing::{AutofixResult, ParsedFile, ResponseApplier};
pub use patching::patcher::{
    DiffSummary, PatchedOutcome, apply_operation_to_text, build_unified_diff_preview,
};
pub use safety::sandbox::SandboxError;
pub use storage::backup::{
    BackupSnapshotInfo, compute_dir_size_bytes, list_backup_snapshots, purge_all_backups,
    restore_backup_by_id,
};
pub use storage::transaction::{ApplyTransactionError, ApplyTransactionReport};
