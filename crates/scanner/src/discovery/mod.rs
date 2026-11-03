pub(crate) mod binary_detector;
pub(crate) mod filter_plan;
pub(crate) mod parallel_walker;

pub use filter_plan::{CompiledFilterPlan, FolderWalkVerdict};
pub use parallel_walker::{IgnoreFileScanner, ScannedFile, ScannerError};
