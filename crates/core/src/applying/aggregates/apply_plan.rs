use serde::{Deserialize, Serialize};

use crate::applying::values::patch_status::{
    PatchOperationKind, PatchResolutionStatus, SearchReplaceBlock, UnifiedDiffHunk,
};
use crate::shared_kernel::fs::line_ending::LineEnding;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    OverwriteFull { content: String },
    SearchReplace { blocks: Vec<SearchReplaceBlock> },
    UnifiedDiff { hunks: Vec<UnifiedDiffHunk> },
    DeleteFile,
}

impl FileOperation {
    #[inline]
    pub const fn kind(&self) -> PatchOperationKind {
        match self {
            Self::OverwriteFull { .. } => PatchOperationKind::OverwriteFull,
            Self::SearchReplace { .. } => PatchOperationKind::SearchReplace,
            Self::UnifiedDiff { .. } => PatchOperationKind::UnifiedDiff,
            Self::DeleteFile => PatchOperationKind::DeleteFile,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedFileItem {
    pub rel_path: String,
    pub operation: FileOperation,
    pub status: PatchResolutionStatus,
    pub is_truncated: bool,
    pub is_sandbox_safe: bool,
    pub resolved_content: String,
    pub diff_preview: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub line_ending: LineEnding,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyPlan {
    pub items: Vec<PlannedFileItem>,
    pub dirty_git_files: Vec<String>,
    pub preamble_stripped: bool,
    pub closed_truncated_block: bool,
}

impl ApplyPlan {
    #[inline]
    pub const fn new(
        items: Vec<PlannedFileItem>,
        dirty_git_files: Vec<String>,
        preamble_stripped: bool,
        closed_truncated_block: bool,
    ) -> Self {
        Self { items, dirty_git_files, preamble_stripped, closed_truncated_block }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    pub fn has_blocking_conflicts(&self) -> bool {
        self.items.iter().any(|i| !i.is_sandbox_safe || i.status.is_blocking_error())
    }
}
