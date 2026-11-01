use std::fmt::Write as _;

use core::applying::aggregates::apply_plan::FileOperation;
use core::applying::values::patch_status::{
    PatchResolutionStatus, SearchReplaceBlock, UnifiedDiffHunk,
};
use core::shared_kernel::fs::line_ending::LineEnding;
use similar::{ChangeTag, TextDiff};

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
    try_fuzzy_whitespace_replace,
    try_normalized_syntax_replace,
    try_comment_insensitive_replace,
    try_fuzzy_anchor_similarity_replace,
    try_anchor_bounds_replace,
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

fn try_fuzzy_whitespace_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let mut search_lines: Vec<&str> = Vec::with_capacity((search.len() / 24).max(4));
    for line in search.lines().map(str::trim) {
        if !line.is_empty() {
            search_lines.push(line);
        }
    }
    let first_search = search_lines.first().copied()?;

    let mut work_lines: Vec<&str> = Vec::with_capacity((working.len() / 20).max(16));
    work_lines.extend(working.lines());
    if work_lines.len() < search_lines.len() {
        return None;
    }

    for start_idx in 0..work_lines.len() {
        if work_lines.get(start_idx).copied().unwrap_or("").trim() != first_search {
            continue;
        }
        if let Some(end_idx) = match_trimmed_subsequence(&work_lines, start_idx, &search_lines) {
            let eol = target_ending.as_str();
            let mut out = String::with_capacity(working.len() + replace.len() + 16);
            let mut first = true;
            for &line in work_lines.get(..start_idx).unwrap_or(&[]) {
                if !first {
                    out.push_str(eol);
                }
                out.push_str(line);
                first = false;
            }
            for rep_line in replace.lines() {
                if !first {
                    out.push_str(eol);
                }
                out.push_str(rep_line);
                first = false;
            }
            for &line in work_lines.get(end_idx..).unwrap_or(&[]) {
                if !first {
                    out.push_str(eol);
                }
                out.push_str(line);
                first = false;
            }
            if working.ends_with('\n') {
                out.push_str(eol);
            }
            return Some(out);
        }
    }
    None
}

fn match_trimmed_subsequence(
    work_lines: &[&str],
    start_idx: usize,
    search_trimmed: &[&str],
) -> Option<usize> {
    let mut w_idx = start_idx;
    for (s_pos, &s_line) in search_trimmed.iter().enumerate() {
        while w_idx < work_lines.len() {
            let candidate = work_lines.get(w_idx).copied().unwrap_or("").trim();
            if candidate.is_empty() && s_pos > 0 {
                w_idx += 1;
                continue;
            }
            break;
        }
        let actual = work_lines.get(w_idx).copied()?.trim();
        if actual != s_line {
            return None;
        }
        w_idx += 1;
    }
    Some(w_idx)
}

fn normalize_loose_code_line(line: &str) -> String {
    let trimmed = line.trim().trim_end_matches([',', ';']);
    let replaced = trimmed.replace("Option::None", "None");
    let mut out = String::with_capacity(replaced.len());
    let mut prev_space = false;
    for ch in replaced.chars() {
        if ch == '"' || ch == '\'' {
            continue;
        }
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn replace_work_lines_range(
    working: &str,
    work_lines: &[&str],
    start_idx: usize,
    end_idx: usize,
    replace: &str,
    target_ending: LineEnding,
) -> String {
    let eol = target_ending.as_str();
    let mut out = String::with_capacity(working.len() + replace.len() + 16);
    let mut first = true;
    for &line in work_lines.get(..start_idx).unwrap_or(&[]) {
        if !first {
            out.push_str(eol);
        }
        out.push_str(line);
        first = false;
    }
    for rep_line in replace.lines() {
        if !first {
            out.push_str(eol);
        }
        out.push_str(rep_line);
        first = false;
    }
    for &line in work_lines.get(end_idx..).unwrap_or(&[]) {
        if !first {
            out.push_str(eol);
        }
        out.push_str(line);
        first = false;
    }
    if working.ends_with('\n') {
        out.push_str(eol);
    }
    out
}

fn try_normalized_syntax_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_norm: Vec<String> =
        search.lines().map(normalize_loose_code_line).filter(|l| !l.is_empty()).collect();
    let first_search = search_norm.first()?;
    let work_lines: Vec<&str> = working.lines().collect();
    if work_lines.len() < search_norm.len() {
        return None;
    }

    for start_idx in 0..work_lines.len() {
        let candidate_first =
            normalize_loose_code_line(work_lines.get(start_idx).copied().unwrap_or(""));
        if &candidate_first != first_search {
            continue;
        }
        let mut w_idx = start_idx;
        let mut matched_all = true;
        for (s_pos, s_norm) in search_norm.iter().enumerate() {
            while w_idx < work_lines.len() {
                let raw = work_lines.get(w_idx).copied().unwrap_or("").trim();
                if raw.is_empty() && s_pos > 0 {
                    w_idx += 1;
                    continue;
                }
                break;
            }
            let Some(actual_raw) = work_lines.get(w_idx).copied() else {
                matched_all = false;
                break;
            };
            if &normalize_loose_code_line(actual_raw) != s_norm {
                matched_all = false;
                break;
            }
            w_idx += 1;
        }
        if matched_all {
            return Some(replace_work_lines_range(
                working,
                &work_lines,
                start_idx,
                w_idx,
                replace,
                target_ending,
            ));
        }
    }
    None
}

fn try_fuzzy_anchor_similarity_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_lines: Vec<String> =
        search.lines().map(normalize_loose_code_line).filter(|l| !l.is_empty()).collect();
    let s_len = search_lines.len();
    if s_len < 2 {
        return None;
    }
    let work_lines: Vec<&str> = working.lines().collect();
    if work_lines.len() < s_len {
        return None;
    }
    let work_norm: Vec<String> =
        work_lines.iter().copied().map(normalize_loose_code_line).collect();

    let min_matches = if s_len == 2 { 2 } else { ((s_len * 3) / 4).max(2) };
    let mut best_match: Option<(usize, usize)> = None;

    for start_idx in 0..=(work_lines.len() - s_len) {
        let first_ok = work_norm.get(start_idx) == search_lines.first();
        let last_ok = work_norm.get(start_idx + s_len - 1) == search_lines.last();
        if !first_ok && !last_ok {
            continue;
        }
        let mut matched = 0_usize;
        for offset in 0..s_len {
            if work_norm.get(start_idx + offset) == search_lines.get(offset) {
                matched += 1;
            }
        }
        if matched >= min_matches && best_match.is_none_or(|(_, best_score)| matched > best_score) {
            best_match = Some((start_idx, matched));
        }
    }

    let (best_start, _) = best_match?;
    Some(replace_work_lines_range(
        working,
        &work_lines,
        best_start,
        best_start + s_len,
        replace,
        target_ending,
    ))
}

fn is_comment_or_placeholder_line(trimmed: &str) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed == "..."
        || (trimmed.starts_with("/*") && trimmed.ends_with("*/"))
}

fn try_comment_insensitive_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_code: Vec<String> = search
        .lines()
        .map(str::trim)
        .filter(|l| !is_comment_or_placeholder_line(l))
        .map(normalize_loose_code_line)
        .filter(|l| !l.is_empty())
        .collect();
    if search_code.len() < 2 {
        return None;
    }
    let first_code = search_code.first()?;
    let work_lines: Vec<&str> = working.lines().collect();

    for start_idx in 0..work_lines.len() {
        let candidate_raw = work_lines.get(start_idx).copied().unwrap_or("").trim();
        if is_comment_or_placeholder_line(candidate_raw)
            || &normalize_loose_code_line(candidate_raw) != first_code
        {
            continue;
        }
        let mut w_idx = start_idx;
        let mut matched_all = true;
        for (s_pos, expected) in search_code.iter().enumerate() {
            while w_idx < work_lines.len() {
                let raw = work_lines.get(w_idx).copied().unwrap_or("").trim();
                if is_comment_or_placeholder_line(raw) && s_pos > 0 {
                    w_idx += 1;
                    continue;
                }
                break;
            }
            let Some(actual_raw) = work_lines.get(w_idx).copied() else {
                matched_all = false;
                break;
            };
            if &normalize_loose_code_line(actual_raw) != expected {
                matched_all = false;
                break;
            }
            w_idx += 1;
        }
        if matched_all {
            return Some(replace_work_lines_range(
                working,
                &work_lines,
                start_idx,
                w_idx,
                replace,
                target_ending,
            ));
        }
    }
    None
}

fn try_anchor_bounds_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_norm: Vec<String> =
        search.lines().map(normalize_loose_code_line).filter(|l| !l.is_empty()).collect();
    if search_norm.len() < 2 {
        return None;
    }
    let first_anchor = search_norm.first()?;
    let last_anchor = search_norm.last()?;
    if first_anchor.len() < 4 || (last_anchor.len() < 4 && search_norm.len() < 3) {
        return None;
    }

    let work_lines: Vec<&str> = working.lines().collect();
    let work_norm: Vec<String> =
        work_lines.iter().copied().map(normalize_loose_code_line).collect();
    let max_span = search_norm.len() + 4;

    let mut found_span: Option<(usize, usize)> = None;
    for start_idx in 0..work_norm.len() {
        if work_norm.get(start_idx) != Some(first_anchor) {
            continue;
        }
        let max_end = (start_idx + max_span).min(work_norm.len());
        for end_idx in (start_idx + 1)..max_end {
            if work_norm.get(end_idx) == Some(last_anchor) {
                if found_span.is_some() {
                    return None;
                }
                found_span = Some((start_idx, end_idx + 1));
                break;
            }
        }
    }

    let (start_idx, end_idx) = found_span?;
    Some(replace_work_lines_range(working, &work_lines, start_idx, end_idx, replace, target_ending))
}

fn try_idempotent_already_applied(
    working: &str,
    _search: &str,
    replace: &str,
    _target_ending: LineEnding,
) -> Option<String> {
    let trimmed_rep = replace.trim();
    if trimmed_rep.is_empty() {
        return None;
    }
    if working.contains(trimmed_rep) {
        return Some(working.to_owned());
    }
    let norm_rep = trimmed_rep.replace("\r\n", "\n");
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
}
