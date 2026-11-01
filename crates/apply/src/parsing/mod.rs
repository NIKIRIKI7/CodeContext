use std::fs;
use std::path::{Path, PathBuf};

use core::applying::aggregates::apply_plan::{ApplyPlan, FileOperation, PlannedFileItem};
use core::applying::aggregates::backup_manifest::{BackupFileEntry, BackupManifest};
use core::applying::ports::applier::{ApplierPort, ApplierPortError};
use core::applying::values::patch_status::PatchResolutionStatus;
use core::preset::models::template::{DEFAULT_FILE_HEADER_PREFIX, OutputTemplate, ParserConfig};

use crate::patching::patcher::apply_operation_to_text;
use crate::safety::git_status::check_dirty_git_files;
use crate::safety::sandbox::{
    SandboxError, validate_sandbox_path, validate_sandbox_path_with_fs_check,
};
use crate::storage::transaction::{
    ApplyTransactionError, ApplyTransactionReport, execute_apply_transaction,
    execute_apply_with_deletions, execute_undo_transaction,
};

pub(crate) mod autofix;
pub(crate) mod fenced_blocks;
pub(crate) mod search_replace;
pub(crate) mod unified_diff;

pub use autofix::AutofixResult;
pub use fenced_blocks::ParsedFile;
pub use search_replace::ParsedSearchReplaceFile;
pub use unified_diff::ParsedUnifiedDiffFile;

#[derive(Debug, Default)]
struct PlanOperationsCollector {
    ops: Vec<(String, FileOperation, bool)>,
}

impl PlanOperationsCollector {
    #[inline]
    fn with_capacity(cap: usize) -> Self {
        Self { ops: Vec::with_capacity(cap) }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[inline]
    fn contains_path(&self, rel_path: &str) -> bool {
        self.ops.iter().any(|(existing, _, _)| existing == rel_path)
    }

    #[inline]
    fn push_if_absent(&mut self, rel_path: String, op: FileOperation, is_truncated: bool) {
        if !self.contains_path(&rel_path) {
            self.ops.push((rel_path, op, is_truncated));
        }
    }

    #[inline]
    fn into_vec(self) -> Vec<(String, FileOperation, bool)> {
        self.ops
    }
}

type FormatExtractorFn =
    fn(&str, Option<&Path>, &OutputTemplate, &ParserConfig, &mut PlanOperationsCollector);

const FORMAT_EXTRACTOR_CHAIN: &[FormatExtractorFn] = &[
    extract_search_replace_step,
    extract_unified_diff_step,
    extract_full_files_step,
    extract_delete_directives_step,
];

fn extract_search_replace_step(
    raw_response: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
    collector: &mut PlanOperationsCollector,
) {
    let sr_files =
        search_replace::parse_search_replace_blocks(raw_response, project_root, template, config);
    for sr in sr_files {
        collector.push_if_absent(
            sr.rel_path,
            FileOperation::SearchReplace { blocks: sr.blocks },
            sr.is_truncated,
        );
    }
}

fn extract_unified_diff_step(
    raw_response: &str,
    project_root: Option<&Path>,
    _template: &OutputTemplate,
    config: &ParserConfig,
    collector: &mut PlanOperationsCollector,
) {
    let diff_files = unified_diff::parse_unified_diff_blocks(raw_response, project_root, config);
    for df in diff_files {
        let op = if df.is_deleted {
            FileOperation::DeleteFile
        } else {
            FileOperation::UnifiedDiff { hunks: df.hunks }
        };
        collector.push_if_absent(df.rel_path, op, false);
    }
}

fn extract_full_files_step(
    raw_response: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
    collector: &mut PlanOperationsCollector,
) {
    let should_extract = collector.is_empty() || {
        let header_prefix = if config.sync_with_output_template {
            template.file_header_prefix.as_str()
        } else {
            DEFAULT_FILE_HEADER_PREFIX
        };
        fenced_blocks::find_line_start_marker(raw_response, header_prefix, 0).is_some()
            || fenced_blocks::find_line_start_marker(
                raw_response,
                concat!("=== ", "CREATE FILE: "),
                0,
            )
            .is_some()
            || fenced_blocks::find_line_start_marker(raw_response, concat!("=== ", "NEW FILE: "), 0)
                .is_some()
    };

    if should_extract {
        let full_files = fenced_blocks::parse_ai_response_with_config(
            raw_response,
            project_root,
            template,
            config,
        );
        for pf in full_files {
            collector.push_if_absent(
                pf.rel_path,
                FileOperation::OverwriteFull { content: pf.content },
                pf.is_truncated,
            );
        }
    }
}

fn extract_delete_directives_step(
    raw_response: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
    collector: &mut PlanOperationsCollector,
) {
    let explicit_deletes =
        fenced_blocks::parse_delete_directives(raw_response, project_root, template, config);
    for del_path in explicit_deletes {
        collector.push_if_absent(del_path, FileOperation::DeleteFile, false);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ResponseApplier;

impl ResponseApplier {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    #[inline]
    pub fn validate_path(
        project_root: &Path,
        candidate_rel_path: &str,
    ) -> Result<PathBuf, SandboxError> {
        validate_sandbox_path(project_root, candidate_rel_path)
    }

    #[inline]
    pub fn parse(response_text: &str) -> Vec<ParsedFile> {
        fenced_blocks::parse_ai_response(response_text)
    }

    #[inline]
    pub fn parse_with_config(
        response_text: &str,
        project_root: Option<&Path>,
        template: &OutputTemplate,
        config: &ParserConfig,
    ) -> Vec<ParsedFile> {
        let plan = Self.build_apply_plan(project_root, response_text, template, config);
        plan.items
            .into_iter()
            .map(|item| ParsedFile {
                rel_path: item.rel_path,
                content: item.resolved_content,
                is_truncated: item.is_truncated,
            })
            .collect()
    }

    #[inline]
    pub fn autofix(corrupted_text: &str) -> AutofixResult {
        autofix::autofix_response(corrupted_text)
    }

    #[inline]
    pub fn apply_files(
        project_root: &Path,
        files: &[ParsedFile],
        create_backup: bool,
    ) -> Result<ApplyTransactionReport, ApplyTransactionError> {
        execute_apply_transaction(project_root, files, create_backup)
    }

    #[inline]
    pub fn apply_files_and_deletions(
        project_root: &Path,
        files_to_write: &[ParsedFile],
        files_to_delete: &[String],
        create_backup: bool,
    ) -> Result<ApplyTransactionReport, ApplyTransactionError> {
        execute_apply_with_deletions(project_root, files_to_write, files_to_delete, create_backup)
    }

    #[inline]
    pub fn undo_last_backup(project_root: &Path) -> Result<usize, String> {
        execute_undo_transaction(project_root)
    }
}

impl ApplierPort for ResponseApplier {
    fn autofix_with_config(
        &self,
        raw_response: &str,
        template: &OutputTemplate,
        config: &ParserConfig,
    ) -> (String, bool, bool) {
        let res = autofix::autofix_response_with_config(raw_response, template, config);
        (res.fixed_text, res.preamble_stripped, res.closed_truncated_block)
    }

    fn build_apply_plan(
        &self,
        project_root: Option<&Path>,
        raw_response: &str,
        template: &OutputTemplate,
        config: &ParserConfig,
    ) -> ApplyPlan {
        let estimated_ops = (raw_response.len() / 128).clamp(8, 512);
        let mut collector = PlanOperationsCollector::with_capacity(estimated_ops);
        for &extractor in FORMAT_EXTRACTOR_CHAIN {
            extractor(raw_response, project_root, template, config, &mut collector);
        }
        let raw_ops = collector.into_vec();

        let fallback_root = Path::new(".");
        let validation_root = project_root.unwrap_or(fallback_root);
        let check_disk_symlinks = project_root.is_some_and(Path::exists);

        let mut items = Vec::with_capacity(raw_ops.len());
        let mut valid_rel_paths = Vec::with_capacity(raw_ops.len());

        for (rel_path, operation, is_truncated) in raw_ops {
            let sandbox_check = validate_sandbox_path_with_fs_check(
                validation_root,
                &rel_path,
                check_disk_symlinks,
            );
            let is_sandbox_safe = sandbox_check.is_ok();

            let existing_content = if is_sandbox_safe && check_disk_symlinks {
                valid_rel_paths.push(rel_path.clone());
                project_root.and_then(|root| read_existing_disk_file(root, &rel_path))
            } else {
                None
            };

            let outcome = apply_operation_to_text(
                &rel_path,
                existing_content.as_deref(),
                &operation,
                is_truncated,
            );

            let status = match sandbox_check {
                Ok(_) => outcome.status,
                Err(err) => PatchResolutionStatus::BlockedBySandbox { reason: err.to_string() },
            };

            items.push(PlannedFileItem {
                rel_path,
                operation,
                status,
                is_truncated,
                is_sandbox_safe,
                resolved_content: outcome.resolved_content,
                diff_preview: outcome.diff.unified_diff_text,
                added_lines: outcome.diff.added_lines,
                removed_lines: outcome.diff.removed_lines,
                line_ending: outcome.diff.line_ending,
            });
        }

        let dirty_git_files = if check_disk_symlinks {
            project_root
                .map(|root| check_dirty_git_files(root, &valid_rel_paths))
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        ApplyPlan::new(items, dirty_git_files, false, false)
    }

    fn execute_apply_plan(
        &self,
        project_root: &Path,
        plan: &ApplyPlan,
        create_backup: bool,
    ) -> Result<BackupManifest, ApplierPortError> {
        if plan.items.is_empty() {
            return Err(ApplierPortError::EmptyPlan);
        }
        for item in &plan.items {
            if !item.is_sandbox_safe {
                return Err(ApplierPortError::SandboxViolation(item.rel_path.clone()));
            }
            if let PatchResolutionStatus::SearchBlockConflict { unmatched_count } = item.status {
                return Err(ApplierPortError::PatchConflict {
                    path: item.rel_path.clone(),
                    unmatched: unmatched_count,
                });
            }
        }

        let files_to_write: Vec<ParsedFile> = plan
            .items
            .iter()
            .filter(|i| !matches!(i.operation, FileOperation::DeleteFile))
            .map(|i| ParsedFile {
                rel_path: i.rel_path.clone(),
                content: i.resolved_content.clone(),
                is_truncated: i.is_truncated,
            })
            .collect();

        let files_to_delete: Vec<String> = plan
            .items
            .iter()
            .filter(|i| matches!(i.operation, FileOperation::DeleteFile))
            .map(|i| i.rel_path.clone())
            .collect();

        let entries: Vec<BackupFileEntry> = plan
            .items
            .iter()
            .map(|i| BackupFileEntry {
                rel_path: i.rel_path.clone(),
                existed_before: project_root.join(&i.rel_path).exists(),
            })
            .collect();

        let report = execute_apply_with_deletions(
            project_root,
            &files_to_write,
            &files_to_delete,
            create_backup,
        )
        .map_err(|err| match err {
            ApplyTransactionError::EmptyPlan => ApplierPortError::EmptyPlan,
            ApplyTransactionError::Sandbox(s) => ApplierPortError::SandboxViolation(s.to_string()),
            ApplyTransactionError::Storage(st) => ApplierPortError::StorageError(st.to_string()),
        })?;

        Ok(BackupManifest::new(0, report.backup_dir, report.applied_files_count, entries))
    }

    fn undo_last_backup(&self, project_root: &Path) -> Result<usize, ApplierPortError> {
        execute_undo_transaction(project_root).map_err(ApplierPortError::StorageError)
    }
}

fn read_existing_disk_file(root: &Path, rel_path: &str) -> Option<String> {
    let direct = root.join(rel_path);
    if let Ok(content) = fs::read_to_string(&direct) {
        return Some(content);
    }
    if let Some((_, after_first)) = rel_path.split_once(['/', '\\'])
        && let Ok(content) = fs::read_to_string(root.join(after_first))
    {
        return Some(content);
    }
    let mut parent_opt = root.parent();
    for _ in 0..3 {
        let parent = parent_opt?;
        if let Ok(content) = fs::read_to_string(parent.join(rel_path)) {
            return Some(content);
        }
        parent_opt = parent.parent();
    }
    None
}
