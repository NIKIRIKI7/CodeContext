use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PatchOperationKind {
    #[default]
    OverwriteFull,
    SearchReplace,
    UnifiedDiff,
    DeleteFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PatchResolutionStatus {
    #[default]
    Ready,
    TruncatedWarning,
    SearchBlockConflict {
        unmatched_count: usize,
    },
    BlockedBySandbox {
        reason: String,
    },
}

impl PatchResolutionStatus {
    #[inline]
    pub const fn is_blocking_error(&self) -> bool {
        matches!(self, Self::SearchBlockConflict { .. } | Self::BlockedBySandbox { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchReplaceBlock {
    pub search: String,
    pub replace: String,
}

impl SearchReplaceBlock {
    #[inline]
    pub const fn new(search: String, replace: String) -> Self {
        Self { search, replace }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedDiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<String>,
}

impl UnifiedDiffHunk {
    #[inline]
    pub const fn new(
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
        lines: Vec<String>,
    ) -> Self {
        Self { old_start, old_count, new_start, new_count, lines }
    }
}
