mod parsing;
mod patching;
mod safety;
mod storage;

pub use parsing::{
    AutofixResult, ParsedFile, ParsedSearchReplaceFile, ParsedUnifiedDiffFile, ResponseApplier,
};
pub use patching::patcher::{
    DiffSummary, PatchedOutcome, apply_operation_to_text, build_unified_diff_preview,
};
pub use safety::sandbox::SandboxError;
pub use storage::transaction::{ApplyTransactionError, ApplyTransactionReport};
