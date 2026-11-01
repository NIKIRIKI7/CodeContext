use std::path::Path;

use core::applying::values::patch_status::SearchReplaceBlock;
use core::preset::models::template::{
    DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX, DEFAULT_FILE_HEADER_SUFFIX, OutputTemplate,
    ParserConfig,
};

use super::fenced_blocks::{
    extract_markdown_heading_or_label_path, find_line_start_marker, looks_like_file_path,
    normalize_extracted_path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSearchReplaceFile {
    pub rel_path: String,
    pub blocks: Vec<SearchReplaceBlock>,
    pub is_truncated: bool,
}

const CODE_FENCE: &str = "\x60\x60\x60";

pub(crate) fn parse_search_replace_blocks(
    input: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
) -> Vec<ParsedSearchReplaceFile> {
    if !config.enable_search_replace {
        return Vec::new();
    }

    let search_marker = config.search_replace_markers.search_header.as_str();
    let divider_marker = config.search_replace_markers.divider.as_str();
    let replace_marker = config.search_replace_markers.replace_footer.as_str();

    if find_line_start_marker(input, search_marker, 0).is_none()
        && find_line_start_marker(input, "<<<<<", 0).is_none()
    {
        return Vec::new();
    }

    let header_prefix = if config.sync_with_output_template {
        template.file_header_prefix.as_str()
    } else {
        DEFAULT_FILE_HEADER_PREFIX
    };
    let header_suffix = if config.sync_with_output_template {
        template.header_suffix_trimmed()
    } else {
        DEFAULT_FILE_HEADER_SUFFIX.trim_end_matches(['\r', '\n'])
    };
    let footer_marker = if config.sync_with_output_template {
        template.footer_trimmed()
    } else {
        DEFAULT_FILE_FOOTER.trim_matches(['\r', '\n'])
    };

    let estimated_files = (input.len() / 128).clamp(4, 512);
    let mut files: Vec<ParsedSearchReplaceFile> = Vec::with_capacity(estimated_files);
    let mut current_path: Option<String> = None;
    let mut state = BlockParseState::Idle;
    let mut search_buf = String::with_capacity(256);
    let mut replace_buf = String::with_capacity(256);

    for line in input.split_inclusive('\n') {
        let trimmed = line.trim();

        match state {
            BlockParseState::Idle => {
                if is_search_open_line(trimmed, search_marker) {
                    state = BlockParseState::InSearch;
                    search_buf.clear();
                    replace_buf.clear();
                } else if trimmed == footer_marker || trimmed.starts_with("=== END FILE") {
                    current_path = None;
                } else if let Some(path) = extract_candidate_path_line(
                    trimmed,
                    header_prefix,
                    header_suffix,
                    project_root,
                    config,
                ) {
                    current_path = Some(path);
                } else if trimmed.starts_with("=== ") && trimmed.ends_with("===") {
                    current_path = None;
                }
            }
            BlockParseState::InSearch => {
                if is_divider_line(trimmed, divider_marker) {
                    state = BlockParseState::InReplace;
                } else {
                    search_buf.push_str(line);
                }
            }
            BlockParseState::InReplace => {
                if is_replace_close_line(trimmed, replace_marker) {
                    if let Some(ref rel_path) = current_path {
                        push_search_replace_block(
                            &mut files,
                            rel_path,
                            strip_trailing_newline(&search_buf),
                            strip_trailing_newline(&replace_buf),
                            false,
                        );
                    }
                    state = BlockParseState::Idle;
                } else {
                    replace_buf.push_str(line);
                }
            }
        }
    }

    if state != BlockParseState::Idle
        && let Some(ref rel_path) = current_path
    {
        push_search_replace_block(
            &mut files,
            rel_path,
            strip_trailing_newline(&search_buf),
            strip_trailing_newline(&replace_buf),
            true,
        );
    }

    files
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockParseState {
    Idle,
    InSearch,
    InReplace,
}

fn extract_candidate_path_line(
    trimmed: &str,
    header_prefix: &str,
    header_suffix: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Option<String> {
    if trimmed.is_empty() || trimmed.starts_with(CODE_FENCE) {
        return None;
    }

    for prefix in [
        header_prefix,
        "=== FILE: ",
        "=== CREATE FILE: ",
        "=== NEW FILE: ",
        "=== UPDATE FILE: ",
        "=== MODIFY FILE: ",
    ] {
        if let Some(after_prefix) = trimmed.strip_prefix(prefix) {
            let raw_path = after_prefix
                .strip_suffix(header_suffix)
                .unwrap_or_else(|| after_prefix.trim_end_matches('=').trim());
            let clean = normalize_extracted_path(raw_path, project_root, config);
            if looks_like_file_path(&clean) {
                return Some(clean);
            }
        }
    }

    if let Some(h_path) = extract_markdown_heading_or_label_path(trimmed, project_root, config) {
        return Some(h_path);
    }

    if let Some(comment_stripped) = trimmed.strip_prefix("//").or_else(|| trimmed.strip_prefix('#'))
    {
        let clean = normalize_extracted_path(comment_stripped.trim(), project_root, config);
        if looks_like_file_path(&clean) && !contains_code_syntax_chars(&clean) {
            return Some(clean);
        }
    }

    None
}

#[inline]
fn contains_code_syntax_chars(s: &str) -> bool {
    s.contains(['(', ')', '{', '}', '[', ']', ';', '=', ',', '<', '>', '|'])
}

#[inline]
fn is_search_open_line(trimmed: &str, search_marker: &str) -> bool {
    if trimmed.starts_with(search_marker) {
        return true;
    }
    if let Some(rest) = trimmed.strip_prefix("<<<<<") {
        let after_angles = rest.trim_start_matches('<').trim();
        return after_angles.is_empty()
            || after_angles.eq_ignore_ascii_case("SEARCH")
            || after_angles.eq_ignore_ascii_case("HEAD")
            || after_angles.eq_ignore_ascii_case("ORIGINAL")
            || after_angles.eq_ignore_ascii_case("OLD");
    }
    false
}

#[inline]
fn is_divider_line(trimmed: &str, divider_marker: &str) -> bool {
    if trimmed.starts_with(divider_marker) {
        return true;
    }
    trimmed.starts_with("=====") && trimmed.trim_start_matches('=').trim().is_empty()
}

#[inline]
fn is_replace_close_line(trimmed: &str, replace_marker: &str) -> bool {
    if trimmed.starts_with(replace_marker) {
        return true;
    }
    if let Some(rest) = trimmed.strip_prefix(">>>>>") {
        let after_angles = rest.trim_start_matches('>').trim();
        return after_angles.is_empty()
            || after_angles.eq_ignore_ascii_case("REPLACE")
            || after_angles.eq_ignore_ascii_case("UPDATED")
            || after_angles.eq_ignore_ascii_case("NEW")
            || after_angles.eq_ignore_ascii_case("MODIFIED");
    }
    false
}

fn push_search_replace_block(
    files: &mut Vec<ParsedSearchReplaceFile>,
    rel_path: &str,
    search: &str,
    replace: &str,
    is_truncated: bool,
) {
    let block = SearchReplaceBlock::new(search.to_owned(), replace.to_owned());
    if let Some(existing) = files.iter_mut().find(|f| f.rel_path == rel_path) {
        existing.blocks.push(block);
        existing.is_truncated |= is_truncated;
    } else {
        files.push(ParsedSearchReplaceFile {
            rel_path: rel_path.to_owned(),
            blocks: vec![block],
            is_truncated,
        });
    }
}

#[inline]
fn strip_trailing_newline(s: &str) -> &str {
    if let Some(stripped) = s.strip_suffix("\r\n") {
        stripped
    } else if let Some(stripped) = s.strip_suffix('\n') {
        stripped
    } else {
        s
    }
}
