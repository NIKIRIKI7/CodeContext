use std::path::Path;

use core::applying::values::patch_status::UnifiedDiffHunk;
use core::preset::models::template::ParserConfig;

use super::fenced_blocks::{looks_like_file_path, normalize_extracted_path};

const CODE_FENCE: &str = "\x60\x60\x60";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUnifiedDiffFile {
    pub rel_path: String,
    pub is_deleted: bool,
    pub hunks: Vec<UnifiedDiffHunk>,
}

pub(crate) fn parse_unified_diff_blocks(
    input: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Vec<ParsedUnifiedDiffFile> {
    if !config.enable_unified_diff || (!input.contains("@@ ") && !input.contains("+++ /dev/null")) {
        return Vec::new();
    }

    let estimated_files = (input.len() / 128).clamp(4, 512);
    let mut files: Vec<ParsedUnifiedDiffFile> = Vec::with_capacity(estimated_files);
    let mut current_old_path: Option<String> = None;
    let mut current_new_path: Option<String> = None;
    let mut current_hunk: Option<UnifiedDiffHunk> = None;

    for raw_line in input.lines() {
        let line = raw_line.trim_end_matches('\r');

        if let Some(old_raw) = line.strip_prefix("--- ") {
            flush_hunk(
                &mut files,
                current_old_path.as_deref(),
                current_new_path.as_deref(),
                &mut current_hunk,
            );
            current_old_path = Some(clean_diff_header_path(old_raw, project_root, config));
            continue;
        }

        if let Some(new_raw) = line.strip_prefix("+++ ") {
            let parsed_new = clean_diff_header_path(new_raw, project_root, config);
            if parsed_new == "/dev/null"
                && let Some(ref old_p) = current_old_path
                && old_p != "/dev/null"
                && looks_like_file_path(old_p)
                && !files.iter().any(|f| &f.rel_path == old_p)
            {
                files.push(ParsedUnifiedDiffFile {
                    rel_path: old_p.clone(),
                    is_deleted: true,
                    hunks: Vec::new(),
                });
            }
            current_new_path = Some(parsed_new);
            continue;
        }

        if let Some((old_start, old_count, new_start, new_count)) = parse_hunk_header(line) {
            flush_hunk(
                &mut files,
                current_old_path.as_deref(),
                current_new_path.as_deref(),
                &mut current_hunk,
            );
            let cap = old_count.saturating_add(new_count).max(4);
            current_hunk = Some(UnifiedDiffHunk::new(
                old_start,
                old_count,
                new_start,
                new_count,
                Vec::with_capacity(cap),
            ));
            continue;
        }

        if let Some(ref mut hunk) = current_hunk {
            if line.starts_with('+') || line.starts_with('-') || line.starts_with(' ') {
                hunk.lines.push(line.to_owned());
            } else if line.is_empty() {
                hunk.lines.push(" ".to_owned());
            } else if line.starts_with(CODE_FENCE) {
                flush_hunk(
                    &mut files,
                    current_old_path.as_deref(),
                    current_new_path.as_deref(),
                    &mut current_hunk,
                );
            }
        }
    }

    flush_hunk(
        &mut files,
        current_old_path.as_deref(),
        current_new_path.as_deref(),
        &mut current_hunk,
    );
    files
}

fn flush_hunk(
    files: &mut Vec<ParsedUnifiedDiffFile>,
    old_path: Option<&str>,
    new_path: Option<&str>,
    current_hunk: &mut Option<UnifiedDiffHunk>,
) {
    let Some(hunk) = current_hunk.take() else {
        return;
    };
    let is_deleted = new_path == Some("/dev/null");
    let target_path = if is_deleted {
        old_path.unwrap_or("")
    } else {
        new_path
            .filter(|&p| p != "/dev/null")
            .or_else(|| old_path.filter(|&p| p != "/dev/null"))
            .unwrap_or("")
    };

    if target_path.is_empty() || !looks_like_file_path(target_path) {
        return;
    }

    if let Some(existing) = files.iter_mut().find(|f| f.rel_path == target_path) {
        existing.is_deleted |= is_deleted;
        existing.hunks.push(hunk);
    } else {
        files.push(ParsedUnifiedDiffFile {
            rel_path: target_path.to_owned(),
            is_deleted,
            hunks: vec![hunk],
        });
    }
}

fn clean_diff_header_path(raw: &str, project_root: Option<&Path>, config: &ParserConfig) -> String {
    let token = raw.split('\t').next().unwrap_or(raw).trim();
    if token == "/dev/null" {
        return "/dev/null".to_owned();
    }
    let without_ab = token.strip_prefix("a/").or_else(|| token.strip_prefix("b/")).unwrap_or(token);
    normalize_extracted_path(without_ab, project_root, config)
}

fn parse_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    let after_open = line.strip_prefix("@@ -")?;
    let end_markers = after_open.find(" @@")?;
    let range_part = after_open.get(..end_markers)?;
    let (old_part, new_part) = range_part.split_once(" +")?;
    let (old_start, old_count) = parse_range_pair(old_part)?;
    let (new_start, new_count) = parse_range_pair(new_part)?;
    Some((old_start, old_count, new_start, new_count))
}

fn parse_range_pair(s: &str) -> Option<(usize, usize)> {
    if let Some((start_s, count_s)) = s.split_once(',') {
        Some((start_s.parse().ok()?, count_s.parse().ok()?))
    } else {
        Some((s.parse().ok()?, 1))
    }
}
