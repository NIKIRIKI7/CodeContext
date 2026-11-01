use std::fmt::Write as _;

use core::applying::aggregates::apply_plan::FileOperation;
use core::applying::values::patch_status::{
    PatchResolutionStatus, SearchReplaceBlock, UnifiedDiffHunk,
};
use core::shared_kernel::fs::line_ending::LineEnding;
use similar::{ChangeTag, TextDiff};

use super::fuzzy_matcher::{
    try_ellipsis_and_comment_tolerant_replace, try_syntax_normalized_replace,
    try_trimmed_subsequence_replace, try_zed_fuzzy_similarity_replace,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSummary {
    pub unified_diff_text: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub line_ending: LineEnding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchedOutcome {
    pub resolved_content: String,
    pub status: PatchResolutionStatus,
    pub diff: DiffSummary,
}

type BlockMatcherFn = fn(&str, &str, &str, LineEnding) -> Option<String>;

const SEARCH_REPLACE_MATCHER_CHAIN: &[BlockMatcherFn] = &[
    try_exact_replace,
    try_line_ending_normalized_replace,
    try_trimmed_subsequence_replace,
    try_syntax_normalized_replace,
    try_ellipsis_and_comment_tolerant_replace,
    try_zed_fuzzy_similarity_replace,
    try_idempotent_already_applied,
];

pub fn apply_operation_to_text(
    rel_path: &str,
    existing_disk_content: Option<&str>,
    operation: &FileOperation,
    is_truncated: bool,
) -> PatchedOutcome {
    let base_content = existing_disk_content.unwrap_or("");
    let detected_ending = existing_disk_content.map_or_else(
        || match operation {
            FileOperation::OverwriteFull { content } => LineEnding::detect(content),
            _ => LineEnding::Lf,
        },
        LineEnding::detect,
    );

    if matches!(operation, FileOperation::DeleteFile) {
        let removed_lines = existing_disk_content.map_or(0, |s| s.lines().count());
        let mut preview = String::with_capacity(base_content.len() + 128);
        let _ = writeln!(
            preview,
            "{}{removed_lines}{}{rel_path})",
            concat!("@", "@ -1,"),
            concat!(" +0,0 @", "@ (удаление файла: ")
        );
        for line in base_content.lines() {
            preview.push('-');
            preview.push_str(line);
            preview.push('\n');
        }
        return PatchedOutcome {
            resolved_content: String::new(),
            status: PatchResolutionStatus::Ready,
            diff: DiffSummary {
                unified_diff_text: preview,
                added_lines: 0,
                removed_lines,
                line_ending: detected_ending,
            },
        };
    }

    let (resolved_content, mut status) = match operation {
        FileOperation::OverwriteFull { content } => (
            normalize_overwrite_for_existing(content, existing_disk_content, detected_ending),
            PatchResolutionStatus::Ready,
        ),
        FileOperation::SearchReplace { blocks } => {
            apply_search_replace_blocks(base_content, blocks, detected_ending)
        }
        FileOperation::UnifiedDiff { hunks } => {
            apply_unified_diff_hunks(base_content, hunks, detected_ending)
        }
        FileOperation::DeleteFile => (String::new(), PatchResolutionStatus::Ready),
    };

    if is_truncated && status == PatchResolutionStatus::Ready {
        status = PatchResolutionStatus::TruncatedWarning;
    }

    let diff = build_unified_diff_preview(
        rel_path,
        existing_disk_content,
        &resolved_content,
        detected_ending,
    );

    PatchedOutcome { resolved_content, status, diff }
}

fn normalize_overwrite_for_existing(
    content: &str,
    existing_disk_content: Option<&str>,
    target_ending: LineEnding,
) -> String {
    let Some(existing) = existing_disk_content else {
        return content.to_owned();
    };

    let mut out = if target_ending == LineEnding::Lf && content.contains('\r') {
        content.replace("\r\n", "\n")
    } else {
        content.to_owned()
    };

    if existing.ends_with('\n') && !out.is_empty() && !out.ends_with('\n') {
        out.push_str(target_ending.as_str());
    }

    out
}

pub fn build_unified_diff_preview(
    rel_path: &str,
    old_content_opt: Option<&str>,
    new_content: &str,
    line_ending: LineEnding,
) -> DiffSummary {
    let Some(old_content) = old_content_opt else {
        let added_lines = new_content.lines().count();
        let mut preview =
            String::with_capacity(new_content.len() + added_lines.saturating_mul(2) + 128);
        let _ = writeln!(
            preview,
            "{}{added_lines}{}{rel_path})",
            concat!("@", "@ -0,0 +1,"),
            concat!(" @", "@ (новый файл: ")
        );
        for line in new_content.lines() {
            preview.push('+');
            preview.push_str(line);
            preview.push('\n');
        }
        return DiffSummary {
            unified_diff_text: preview,
            added_lines,
            removed_lines: 0,
            line_ending,
        };
    };

    let text_diff = TextDiff::from_lines(old_content, new_content);
    let mut added_lines = 0_usize;
    let mut removed_lines = 0_usize;
    for change in text_diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => added_lines += 1,
            ChangeTag::Delete => removed_lines += 1,
            ChangeTag::Equal => {}
        }
    }

    if added_lines == 0 && removed_lines == 0 {
        let header_open = concat!("@", "@ Без изменений (");
        let header_close = concat!(") @", "@\n");
        return DiffSummary {
            unified_diff_text: format!("{header_open}{rel_path}{header_close}{new_content}"),
            added_lines: 0,
            removed_lines: 0,
            line_ending,
        };
    }

    let unified_diff_text = text_diff
        .unified_diff()
        .context_radius(3)
        .header(&format!("a/{rel_path}"), &format!("b/{rel_path}"))
        .to_string();

    DiffSummary { unified_diff_text, added_lines, removed_lines, line_ending }
}

fn apply_search_replace_blocks(
    original: &str,
    blocks: &[SearchReplaceBlock],
    line_ending: LineEnding,
) -> (String, PatchResolutionStatus) {
    apply_search_replace_with_chain(original, blocks, line_ending, SEARCH_REPLACE_MATCHER_CHAIN)
}

fn apply_search_replace_with_chain(
    original: &str,
    blocks: &[SearchReplaceBlock],
    line_ending: LineEnding,
    matcher_chain: &[BlockMatcherFn],
) -> (String, PatchResolutionStatus) {
    let mut working = original.to_owned();
    let mut unmatched_count = 0_usize;

    for block in blocks {
        if block.search.is_empty() {
            if working.is_empty() || blocks.len() == 1 {
                working.clone_from(&block.replace);
            } else if !working.contains(&block.replace) {
                working.push_str(&block.replace);
            }
            continue;
        }
        let mut matched = false;
        for &matcher in matcher_chain {
            if let Some(updated) = matcher(&working, &block.search, &block.replace, line_ending) {
                working = updated;
                matched = true;
                break;
            }
        }
        if !matched {
            unmatched_count += 1;
        }
    }

    let status = if unmatched_count > 0 {
        PatchResolutionStatus::SearchBlockConflict { unmatched_count }
    } else {
        PatchResolutionStatus::Ready
    };
    (working, status)
}

fn try_exact_replace(
    working: &str,
    search: &str,
    replace: &str,
    _target_ending: LineEnding,
) -> Option<String> {
    let pos = working.find(search)?;
    let mut updated = String::with_capacity(
        working.len().saturating_sub(search.len()).saturating_add(replace.len()),
    );
    updated.push_str(working.get(..pos).unwrap_or(""));
    updated.push_str(replace);
    updated.push_str(working.get(pos + search.len()..).unwrap_or(""));
    Some(updated)
}

fn try_line_ending_normalized_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    if !working.contains('\r') && !search.contains('\r') {
        return None;
    }
    let norm_working = working.replace("\r\n", "\n");
    let norm_search = search.replace("\r\n", "\n");
    let norm_replace = replace.replace("\r\n", "\n");
    let pos = norm_working.find(&norm_search)?;
    let mut updated = norm_working;
    updated.replace_range(pos..pos + norm_search.len(), &norm_replace);
    if target_ending == LineEnding::Crlf {
        updated = updated.replace('\n', "\r\n");
    }
    Some(updated)
}

fn try_idempotent_already_applied(
    working: &str,
    search: &str,
    replace: &str,
    _target_ending: LineEnding,
) -> Option<String> {
    let trimmed_rep = replace.trim();
    if trimmed_rep.is_empty() {
        return None;
    }
    let norm_rep = trimmed_rep.replace("\r\n", "\n");
    if norm_rep.lines().count() < 2 && norm_rep.len() < 24 {
        return None;
    }
    let norm_search = search.trim().replace("\r\n", "\n");
    if norm_search.contains(&norm_rep) {
        return None;
    }
    let norm_work = working.replace("\r\n", "\n");
    if norm_work.contains(&norm_rep) {
        return Some(working.to_owned());
    }
    None
}

fn apply_unified_diff_hunks(
    original: &str,
    hunks: &[UnifiedDiffHunk],
    target_ending: LineEnding,
) -> (String, PatchResolutionStatus) {
    let mut blocks = Vec::with_capacity(hunks.len());
    let eol = target_ending.as_str();
    for hunk in hunks {
        let mut search_lines = Vec::new();
        let mut replace_lines = Vec::new();
        for line in &hunk.lines {
            if let Some(removed) = line.strip_prefix('-') {
                search_lines.push(removed);
            } else if let Some(added) = line.strip_prefix('+') {
                replace_lines.push(added);
            } else {
                let ctx = line.strip_prefix(' ').unwrap_or(line.as_str());
                search_lines.push(ctx);
                replace_lines.push(ctx);
            }
        }
        blocks.push(SearchReplaceBlock::new(search_lines.join(eol), replace_lines.join(eol)));
    }
    apply_search_replace_blocks(original, &blocks, target_ending)
}

#[cfg(test)]
mod tests {
    use core::applying::aggregates::apply_plan::FileOperation;
    use core::applying::values::patch_status::{PatchResolutionStatus, SearchReplaceBlock};

    use super::apply_operation_to_text;

    #[test]
    fn applies_search_replace_exact_and_fuzzy_indentation() {
        let orig = "pub fn add(a: i32, b: i32) -> i32 {\n    let res = a + b;\n    res\n}\n";
        let op = FileOperation::SearchReplace {
            blocks: vec![SearchReplaceBlock::new(
                "  let res = a + b;\n  res".to_owned(),
                "    a.saturating_add(b)".to_owned(),
            )],
        };
        let out = apply_operation_to_text("src/math.rs", Some(orig), &op, false);
        assert_eq!(out.status, PatchResolutionStatus::Ready);
        assert!(out.resolved_content.contains("saturating_add"));
        assert_eq!(out.diff.added_lines, 1);
        assert_eq!(out.diff.removed_lines, 2);
    }

    #[test]
    fn zed_fuzzy_matcher_never_leaves_dangling_closing_braces_with_blank_lines() {
        let orig = concat!(
            "use std::path::PathBuf;\n\n",
            "use core::system::ports::updater::{\n",
            "    SemanticVersion, UpdateChannel, UpdateStage, UpdaterPort as _,\n",
            "};\n",
            "use di::AppContainer;\n",
            "use gpui::{\n",
            "    App, Context, Window,\n",
            "};\n\n",
            "use crate::features::change_applier::state::ChangeApplierState;\n"
        );
        let op = FileOperation::SearchReplace {
            blocks: vec![SearchReplaceBlock::new(
                concat!(
                    "use std::path::PathBuf;\n",
                    "use core::system::ports::updater::{\n",
                    "    SemanticVersion, UpdateChannel,\n",
                    "};\n",
                    "    Window, div, px,\n",
                    "};\n",
                    "use crate::features::change_applier::state::ChangeApplierState;"
                )
                .to_owned(),
                concat!(
                    "use std::path::PathBuf;\n",
                    "use std::time::Duration;\n\n",
                    "use di::AppContainer;\n",
                    "use crate::features::change_applier::state::ChangeApplierState;"
                )
                .to_owned(),
            )],
        };
        let out = apply_operation_to_text("src/view.rs", Some(orig), &op, false);
        assert_eq!(out.status, PatchResolutionStatus::Ready);
        assert!(!out.resolved_content.contains("\n};\n"));
        assert!(out.resolved_content.contains("use std::time::Duration;"));
    }

    #[test]
    fn zed_fuzzy_matcher_supports_ellipsis_placeholders_in_search_block() {
        let orig = concat!(
            "pub fn process_items(items: &[u32]) -> u32 {\n",
            "    let mut sum = 0;\n",
            "    for &item in items {\n",
            "        sum += item;\n",
            "    }\n",
            "    sum\n",
            "}\n"
        );
        let op = FileOperation::SearchReplace {
            blocks: vec![SearchReplaceBlock::new(
                "pub fn process_items(items: &[u32]) -> u32 {\n    // ... existing code ...\n    sum\n}"
                    .to_owned(),
                "pub fn process_items(items: &[u32]) -> u32 {\n    items.iter().sum()\n}"
                    .to_owned(),
            )],
        };
        let out = apply_operation_to_text("src/calc.rs", Some(orig), &op, false);
        assert_eq!(out.status, PatchResolutionStatus::Ready);
        assert_eq!(
            out.resolved_content,
            "pub fn process_items(items: &[u32]) -> u32 {\n    items.iter().sum()\n}\n"
        );
    }

    #[test]
    fn fuzzy_matcher_does_not_corrupt_similar_struct_fields_or_duplicate_lines() {
        let orig = concat!(
            "pub struct UiLabels {\n",
            "    pub update_modal_title: &'static str,\n",
            "    pub update_modal_subtitle: &'static str,\n",
            "    pub update_modal_current_label: &'static str,\n",
            "    pub btn_update_now: &'static str,\n",
            "    pub meta_modal_title: &'static str,\n",
            "    pub meta_section_metrics: &'static str,\n",
            "}\n"
        );
        let op = FileOperation::SearchReplace {
            blocks: vec![SearchReplaceBlock::new(
                concat!(
                    "    pub update_modal_title: &'static str,\n",
                    "    pub update_modal_subtitle: &'static str,\n",
                    "    pub update_modal_current_label: &'static str,\n",
                    "    pub btn_update_now: &'static str,\n",
                    "    pub meta_modal_title: &'static str,"
                )
                .to_owned(),
                "    pub meta_modal_title: &'static str,".to_owned(),
            )],
        };
        let out = apply_operation_to_text("src/locale.rs", Some(orig), &op, false);
        assert_eq!(out.status, PatchResolutionStatus::Ready);
        assert!(!out.resolved_content.contains("update_modal_subtitle"));
        assert_eq!(out.resolved_content.matches("meta_modal_title").count(), 1);
    }

    #[test]
    fn idempotent_check_does_not_falsely_succeed_when_replace_is_subset_of_search() {
        let orig = "pub struct S {\n    pub a: u32,\n    pub b: u32,\n}\n";
        let op = FileOperation::SearchReplace {
            blocks: vec![SearchReplaceBlock::new(
                "    pub non_existent_field: u32,\n    pub b: u32,".to_owned(),
                "    pub b: u32,".to_owned(),
            )],
        };
        let out = apply_operation_to_text("src/s.rs", Some(orig), &op, false);
        assert_eq!(out.status, PatchResolutionStatus::SearchBlockConflict { unmatched_count: 1 });
    }
}
