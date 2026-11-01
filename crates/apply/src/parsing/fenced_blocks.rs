use std::path::Path;

use core::preset::models::template::{
    DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX, DEFAULT_FILE_HEADER_SUFFIX, OutputTemplate,
    ParserConfig,
};
use core::shared_kernel::fs::file_kind::{
    contains_ignore_ascii_case, starts_with_ignore_ascii_case,
};

use super::autofix::{is_clean_line_tail, is_offset_inside_rust_string};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile {
    pub rel_path: String,
    pub content: String,
    pub is_truncated: bool,
}

const CODE_FENCE: &str = "\x60\x60\x60";

const CREATE_HEADER_PREFIXES: &[&str] = &[
    "=== CREATE FILE: ",
    "=== NEW FILE: ",
    "=== CREATE: ",
    "=== UPDATE FILE: ",
    "=== MODIFY FILE: ",
];

const EXTENSIONLESS_ROOT_FILENAMES: &[&str] = &[
    "justfile",
    "makefile",
    "gnumakefile",
    "dockerfile",
    "containerfile",
    "vagrantfile",
    "procfile",
    "gemfile",
    "rakefile",
    "brewfile",
    "podfile",
    "fastfile",
    "appfile",
    "snapfile",
    "matchfile",
    "cartfile",
    "guardfile",
    "cakefile",
    "workspace",
    "build",
    "kbuild",
    "kconfig",
    "version",
    "license",
    "licence",
    "copying",
    "notice",
    "authors",
    "contributors",
    "codeowners",
    "patents",
    "readme",
    "changelog",
    "changes",
    "news",
    "history",
    "todo",
    "roadmap",
    "maintainers",
    "security",
    "dslfile",
];

const DELETE_HEADER_PREFIXES: &[&str] = &[
    "=== DELETE FILE: ",
    "=== DELETE: ",
    "=== REMOVE FILE: ",
    "=== REMOVE: ",
    "DELETE FILE:",
    "REMOVE FILE:",
    "УДАЛИТЬ ФАЙЛ:",
];

const ACTION_WORDS_TO_STRIP: &[&str] = &[
    "create file:",
    "create file",
    "new file:",
    "new file",
    "create:",
    "create",
    "создать файл:",
    "создать файл",
    "новый файл:",
    "новый файл",
    "создать:",
    "обновить файл:",
    "обновить файл",
    "modify file:",
    "update file:",
];

const DELETE_ACTION_PREFIXES: &[&str] = &[
    "delete file:",
    "delete file",
    "remove file:",
    "remove file",
    "delete:",
    "delete ",
    "remove:",
    "remove ",
    "удалить файл:",
    "удалить файл",
    "удалить:",
    "удалить ",
    "удаление файла:",
    "удаление файла",
    "git rm -f ",
    "git rm ",
    "rm -f ",
    "rm ",
];

pub(crate) fn parse_ai_response(input: &str) -> Vec<ParsedFile> {
    parse_ai_response_with_config(
        input,
        None,
        &OutputTemplate::canonical(),
        &ParserConfig::canonical(),
    )
}

pub(crate) fn parse_ai_response_with_config(
    input: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
) -> Vec<ParsedFile> {
    let mut results = parse_canonical_blocks(input, project_root, template, config);
    if results.is_empty() && config.enable_markdown_fences {
        results.extend(parse_markdown_blocks(input, project_root, config));
    }
    results
}

pub(crate) fn parse_delete_directives(
    input: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
) -> Vec<String> {
    if !may_contain_delete_directives(input) {
        return Vec::new();
    }

    let header_suffix = if config.sync_with_output_template {
        template.header_suffix_trimmed()
    } else {
        DEFAULT_FILE_HEADER_SUFFIX.trim_end_matches(['\r', '\n'])
    };

    let mut deleted_paths: Vec<String> = Vec::new();
    for raw_line in input.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(path) =
            try_extract_canonical_delete(trimmed, header_suffix, project_root, config)
            && !deleted_paths.contains(&path)
        {
            deleted_paths.push(path);
            continue;
        }

        if let Some(path) = try_extract_markdown_delete(trimmed, project_root, config)
            && !deleted_paths.contains(&path)
        {
            deleted_paths.push(path);
        }
    }
    deleted_paths
}

#[inline]
fn may_contain_delete_directives(input: &str) -> bool {
    contains_ignore_ascii_case(input, "delete")
        || contains_ignore_ascii_case(input, "remove")
        || input.contains("Удалить")
        || input.contains("удалить")
        || input.contains("УДАЛИТЬ")
        || input.contains("Удаление")
        || input.contains("удаление")
        || input.contains("rm ")
}

#[inline]
fn strip_prefix_ignore_case<'a>(haystack: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_ascii() {
        if starts_with_ignore_ascii_case(haystack, prefix) {
            return haystack.get(prefix.len()..);
        }
        return None;
    }

    let mut h_chars = haystack.chars();
    for p_ch in prefix.chars() {
        let h_ch = h_chars.next()?;
        if !h_ch.eq_ignore_ascii_case(&p_ch) && !h_ch.to_lowercase().eq(p_ch.to_lowercase()) {
            return None;
        }
    }
    Some(h_chars.as_str())
}

fn try_extract_canonical_delete(
    trimmed: &str,
    header_suffix: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Option<String> {
    for &prefix in DELETE_HEADER_PREFIXES {
        if let Some(after) = strip_prefix_ignore_case(trimmed, prefix) {
            let after_trimmed = after.trim();
            let raw_path = after_trimmed.strip_suffix(header_suffix).unwrap_or(after_trimmed);
            let clean = normalize_extracted_path(raw_path, project_root, config);
            if looks_like_file_path(&clean) {
                return Some(clean);
            }
        }
    }
    None
}

fn try_extract_markdown_delete(
    trimmed: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Option<String> {
    let stripped_line = trimmed
        .trim_start_matches(['#', '-', '*', '>', ' '])
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ')' || c == ' ');

    for &del_prefix in DELETE_ACTION_PREFIXES {
        if let Some(rest) = strip_prefix_ignore_case(stripped_line, del_prefix) {
            let candidate = extract_backticked_or_labeled(rest.trim(), config);
            let clean = normalize_extracted_path(candidate, project_root, config);
            if looks_like_file_path(&clean) {
                return Some(clean);
            }
        }
    }
    None
}

pub(crate) fn find_line_start_marker(haystack: &str, marker: &str, from: usize) -> Option<usize> {
    if marker.is_empty() {
        return None;
    }
    let mut cursor = from;
    let bytes = haystack.as_bytes();
    while let Some(rel) = haystack.get(cursor..).and_then(|s| s.find(marker)) {
        let abs = cursor + rel;
        let at_line_start = abs == 0 || bytes.get(abs - 1).copied() == Some(b'\n');
        if at_line_start {
            return Some(abs);
        }
        cursor = abs + marker.len();
    }
    None
}

fn find_next_canonical_header<'a>(
    haystack: &'a str,
    primary_prefix: &'a str,
    has_create_headers: bool,
    from: usize,
) -> Option<(usize, &'a str)> {
    let mut best: Option<(usize, &'a str)> = None;
    if let Some(pos) = find_line_start_marker(haystack, primary_prefix, from)
        && !is_offset_inside_rust_string(haystack, pos)
    {
        best = Some((pos, primary_prefix));
    }

    if has_create_headers {
        for &prefix in CREATE_HEADER_PREFIXES {
            if let Some(pos) = find_line_start_marker(haystack, prefix, from)
                && !is_offset_inside_rust_string(haystack, pos)
                && best.is_none_or(|(best_pos, _)| pos < best_pos)
            {
                best = Some((pos, prefix));
            }
        }
    }

    best
}

fn find_standalone_footer_marker(haystack: &str, marker: &str, from: usize) -> Option<usize> {
    if marker.is_empty() {
        return None;
    }
    let mut cursor = from;
    let bytes = haystack.as_bytes();
    while let Some(rel) = haystack.get(cursor..).and_then(|s| s.find(marker)) {
        let abs = cursor + rel;
        let at_line_start = abs == 0 || bytes.get(abs - 1).copied() == Some(b'\n');
        if at_line_start
            && is_clean_line_tail(haystack, abs + marker.len())
            && !is_offset_inside_rust_string(haystack, abs)
        {
            return Some(abs);
        }
        cursor = abs + marker.len();
    }
    None
}

fn parse_canonical_blocks(
    input: &str,
    project_root: Option<&Path>,
    template: &OutputTemplate,
    config: &ParserConfig,
) -> Vec<ParsedFile> {
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

    let has_create_headers = (input.contains("=== CREATE")
        || input.contains("=== NEW")
        || input.contains("=== UPDATE")
        || input.contains("=== MODIFY"))
        && CREATE_HEADER_PREFIXES.iter().any(|&p| find_line_start_marker(input, p, 0).is_some());

    if !has_create_headers && find_line_start_marker(input, header_prefix, 0).is_none() {
        return Vec::new();
    }

    let estimated_files = (input.len() / 128).clamp(4, 512);
    let mut files = Vec::with_capacity(estimated_files);
    let mut search_from = 0;

    while let Some((header_start, matched_prefix)) =
        find_next_canonical_header(input, header_prefix, has_create_headers, search_from)
    {
        let path_start = header_start + matched_prefix.len();
        let Some(rest) = input.get(path_start..) else {
            break;
        };
        let Some(line_end_offset) = rest.find('\n') else {
            break;
        };
        let raw_header_line = rest.get(..line_end_offset).unwrap_or("");
        let header_used_crlf = raw_header_line.ends_with('\r');
        let header_line = raw_header_line.trim_end_matches('\r');
        let Some(raw_path) = header_line.strip_suffix(header_suffix) else {
            search_from = path_start + line_end_offset + 1;
            continue;
        };
        let rel_path = normalize_extracted_path(raw_path, project_root, config);
        let body_start = path_start + line_end_offset + 1;
        let Some(body_rest) = input.get(body_start..) else {
            break;
        };

        let next_header_offset =
            find_next_canonical_header(body_rest, header_prefix, has_create_headers, 0)
                .map(|(pos, _)| pos);
        let footer_match = match (
            find_standalone_footer_marker(body_rest, footer_marker, 0),
            next_header_offset,
        ) {
            (Some(f_pos), Some(h_pos)) if h_pos < f_pos => Some((h_pos, 0)),
            (Some(f_pos), _) => Some((f_pos, footer_marker.len())),
            (None, Some(h_pos)) => Some((h_pos, 0)),
            (None, None) => None,
        };

        if let Some((end_offset, consumed_marker_len)) = footer_match {
            let mut raw_body = body_rest.get(..end_offset).unwrap_or("");
            if header_used_crlf && let Some(stripped) = raw_body.strip_suffix("\r\n") {
                raw_body = stripped;
            } else if let Some(stripped) = raw_body.strip_suffix('\n') {
                raw_body = stripped;
            }
            let cleaned_body = strip_redundant_inner_code_fence(raw_body);
            let is_sr_body = (cleaned_body.contains(&config.search_replace_markers.search_header)
                || cleaned_body.contains("<<<<<"))
                && cleaned_body.contains(&config.search_replace_markers.divider);
            if !rel_path.is_empty() && !is_sr_body {
                files.push(ParsedFile {
                    rel_path,
                    content: cleaned_body.to_owned(),
                    is_truncated: false,
                });
            }
            search_from = body_start + end_offset + consumed_marker_len;
        } else {
            let cleaned_body = strip_redundant_inner_code_fence(body_rest);
            let is_sr_body = (cleaned_body.contains(&config.search_replace_markers.search_header)
                || cleaned_body.contains("<<<<<"))
                && cleaned_body.contains(&config.search_replace_markers.divider);
            if !rel_path.is_empty() && !is_sr_body {
                files.push(ParsedFile {
                    rel_path,
                    content: cleaned_body.to_owned(),
                    is_truncated: true,
                });
            }
            break;
        }
    }

    files
}

fn strip_redundant_inner_code_fence(raw_body: &str) -> &str {
    let trimmed = raw_body.trim();
    if !trimmed.starts_with(CODE_FENCE) || !trimmed.ends_with(CODE_FENCE) {
        return raw_body;
    }
    let Some(first_nl) = trimmed.find('\n') else {
        return raw_body;
    };
    let after_open = trimmed.get(first_nl + 1..).unwrap_or("");
    let Some(without_close) = after_open.strip_suffix(CODE_FENCE) else {
        return raw_body;
    };
    if without_close.lines().any(|l| l.trim().starts_with(CODE_FENCE)) {
        return raw_body;
    }
    without_close
        .strip_suffix("\r\n")
        .or_else(|| without_close.strip_suffix('\n'))
        .unwrap_or(without_close)
}

fn parse_markdown_blocks(
    input: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Vec<ParsedFile> {
    let estimated_files = (input.len() / 128).clamp(4, 512);
    let mut files = Vec::with_capacity(estimated_files);
    let mut pending_path: Option<String> = None;
    let mut in_fence = false;
    let mut current_body = String::with_capacity(512);

    for line in input.split_inclusive('\n') {
        let trimmed = line.trim();
        if !in_fence {
            if let Some(inline_path) = parse_fence_opening(trimmed, project_root, config) {
                pending_path = Some(inline_path);
                in_fence = true;
                current_body.clear();
            } else if trimmed.starts_with(CODE_FENCE) {
                in_fence = true;
                current_body.clear();
            } else if try_extract_markdown_delete(trimmed, project_root, config).is_some() {
                pending_path = None;
            } else if let Some(h_path) =
                extract_markdown_heading_or_label_path(trimmed, project_root, config)
            {
                pending_path = Some(h_path);
            }
        } else if trimmed == CODE_FENCE {
            finalize_fenced_block(
                &mut files,
                &mut pending_path,
                &current_body,
                false,
                project_root,
                config,
            );
            current_body.clear();
            in_fence = false;
        } else {
            current_body.push_str(line);
        }
    }

    if in_fence {
        finalize_fenced_block(
            &mut files,
            &mut pending_path,
            &current_body,
            true,
            project_root,
            config,
        );
    }

    files
}

fn finalize_fenced_block(
    files: &mut Vec<ParsedFile>,
    pending_path: &mut Option<String>,
    current_body: &str,
    is_truncated: bool,
    project_root: Option<&Path>,
    config: &ParserConfig,
) {
    if current_body.contains(&config.search_replace_markers.search_header)
        && current_body.contains(&config.search_replace_markers.divider)
    {
        return;
    }

    let (resolved_path, body_slice) = if let Some(first_line_path) =
        extract_first_line_comment_path(current_body, project_root, config)
    {
        let stripped_body = current_body
            .find('\n')
            .and_then(|newline_pos| current_body.get(newline_pos + 1..))
            .unwrap_or("");
        let _ = pending_path.take();
        (Some(first_line_path), stripped_body)
    } else {
        (pending_path.take(), current_body)
    };

    if let Some(rel_path) = resolved_path
        && !rel_path.is_empty()
    {
        files.push(ParsedFile { rel_path, content: body_slice.to_owned(), is_truncated });
    }
}

fn parse_fence_opening(
    trimmed: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Option<String> {
    let after_ticks = trimmed.strip_prefix(CODE_FENCE)?.trim();
    if after_ticks.is_empty() {
        return None;
    }

    for attr_key in ["title=", "file=", "path=", "filename="] {
        if let Some(pos) = after_ticks.find(attr_key) {
            let rest = after_ticks.get(pos + attr_key.len()..)?.trim();
            let candidate = extract_quoted_or_token(rest);
            let normalized = normalize_extracted_path(candidate, project_root, config);
            if looks_like_file_path(&normalized) {
                return Some(normalized);
            }
        }
    }

    if let Some((_, path_part)) = after_ticks.split_once(':') {
        let candidate = normalize_extracted_path(path_part, project_root, config);
        if looks_like_file_path(&candidate) {
            return Some(candidate);
        }
    }

    if let Some((_, second_token)) = after_ticks.split_once(' ') {
        let candidate = normalize_extracted_path(second_token, project_root, config);
        if looks_like_file_path(&candidate) {
            return Some(candidate);
        }
    }

    let single = normalize_extracted_path(after_ticks, project_root, config);
    if (single.contains('/') || single.contains('\\')) && looks_like_file_path(&single) {
        return Some(single);
    }

    None
}

pub(crate) fn extract_markdown_heading_or_label_path(
    trimmed: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Option<String> {
    if trimmed.starts_with('#') {
        let without_hashes = trimmed.trim_start_matches('#').trim();
        let candidate = extract_backticked_or_labeled(without_hashes, config);
        let clean = normalize_extracted_path(candidate, project_root, config);
        if looks_like_file_path(&clean) {
            return Some(clean);
        }
    }

    if (trimmed.starts_with("**") || trimmed.starts_with('`') || trimmed.starts_with('-'))
        && let Some(start) = trimmed.find('`')
        && let Some(end) = trimmed.get(start + 1..)?.find('`')
    {
        let inside = trimmed.get(start + 1..start + 1 + end)?;
        let clean = normalize_extracted_path(inside, project_root, config);
        if looks_like_file_path(&clean) {
            return Some(clean);
        }
    }

    for prefix in &config.comment_path_prefixes {
        if let Some(after) = strip_prefix_ignore_case(trimmed, prefix) {
            let clean = normalize_extracted_path(after.trim(), project_root, config);
            if looks_like_file_path(&clean) {
                return Some(clean);
            }
        }
    }

    for &action in ACTION_WORDS_TO_STRIP {
        if let Some(after) = strip_prefix_ignore_case(trimmed, action) {
            let candidate = extract_backticked_or_labeled(after.trim(), config);
            let clean = normalize_extracted_path(candidate, project_root, config);
            if looks_like_file_path(&clean) {
                return Some(clean);
            }
        }
    }

    None
}

fn extract_first_line_comment_path(
    body: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> Option<String> {
    let first_line = body.lines().next()?.trim();
    if first_line.is_empty() || first_line.starts_with("#!") {
        return None;
    }

    let comment_inner = if let Some(s) = first_line.strip_prefix("//") {
        s.trim()
    } else if let Some(s) = first_line.strip_prefix('#') {
        s.trim()
    } else if let Some(s) = first_line.strip_prefix("/*").and_then(|s| s.strip_suffix("*/")) {
        s.trim()
    } else if let Some(s) = first_line.strip_prefix("<!--").and_then(|s| s.strip_suffix("-->")) {
        s.trim()
    } else {
        let s = first_line.strip_prefix("--")?;
        s.trim()
    };

    for prefix in &config.comment_path_prefixes {
        if let Some(after) = strip_prefix_ignore_case(comment_inner, prefix) {
            let clean = normalize_extracted_path(after.trim(), project_root, config);
            if looks_like_file_path(&clean) {
                return Some(clean);
            }
        }
    }

    let clean = normalize_extracted_path(comment_inner, project_root, config);
    if looks_like_file_path(&clean) && (clean.contains('/') || clean.contains('.')) {
        return Some(clean);
    }

    None
}

pub(crate) fn extract_backticked_or_labeled<'a>(s: &'a str, config: &ParserConfig) -> &'a str {
    if let Some(start) = s.find('`')
        && let Some(rest) = s.get(start + 1..)
        && let Some(end) = rest.find('`')
        && let Some(extracted) = s.get(start + 1..start + 1 + end)
    {
        return extracted;
    }
    let mut trimmed =
        s.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ')' || c == ' ');
    for &action in ACTION_WORDS_TO_STRIP {
        if let Some(after) = strip_prefix_ignore_case(trimmed, action) {
            trimmed = after.trim();
        }
    }
    for prefix in &config.comment_path_prefixes {
        if let Some(after) = strip_prefix_ignore_case(trimmed, prefix) {
            return after.trim();
        }
    }
    if let Some((before_paren, _)) = trimmed.split_once(" (") {
        return before_paren.trim();
    }
    trimmed
}

fn extract_quoted_or_token(s: &str) -> &str {
    let trimmed = s.trim();
    if let Some(stripped) = trimmed.strip_prefix('"') {
        return stripped.split('"').next().unwrap_or(stripped);
    }
    if let Some(stripped) = trimmed.strip_prefix('\'') {
        return stripped.split('\'').next().unwrap_or(stripped);
    }
    trimmed.split_whitespace().next().unwrap_or(trimmed)
}

pub(crate) fn normalize_extracted_path(
    raw: &str,
    project_root: Option<&Path>,
    config: &ParserConfig,
) -> String {
    let mut s = raw
        .trim()
        .trim_matches(|c: char| c == '`' || c == '*' || c == '"' || c == '\'')
        .trim_end_matches(':')
        .trim();

    if let Some((before_paren, _)) = s.split_once(" (") {
        s = before_paren.trim().trim_matches('`').trim();
    }

    for &action in ACTION_WORDS_TO_STRIP {
        if let Some(after) = strip_prefix_ignore_case(s, action) {
            s = after.trim().trim_matches('`').trim();
        }
    }

    for prefix in &config.comment_path_prefixes {
        if let Some(after) = strip_prefix_ignore_case(s, prefix) {
            s = after.trim().trim_matches('`').trim();
        }
    }

    while let Some(stripped) = s.strip_prefix("./").or_else(|| s.strip_prefix(".\\")) {
        s = stripped;
    }

    if config.strip_project_root_prefix
        && let Some(root) = project_root
        && let Some(root_name) = root.file_name().and_then(|n| n.to_str())
        && !root_name.is_empty()
        && starts_with_ignore_ascii_case(s, root_name)
        && let Some(after_root) = s.get(root_name.len()..)
        && (after_root.starts_with('/') || after_root.starts_with('\\'))
        && let Some(clean_rel) = after_root.get(1..)
    {
        s = clean_rel;
    }

    s.to_owned()
}

pub(crate) fn looks_like_file_path(s: &str) -> bool {
    if s.is_empty()
        || s.contains(' ')
        || s.starts_with("http://")
        || s.starts_with("https://")
        || s.chars().all(|c| c == '.')
    {
        return false;
    }
    if s.contains('/') || s.contains('\\') || s.contains('.') {
        return true;
    }
    is_known_extensionless_filename(s)
}

#[inline]
fn is_known_extensionless_filename(name: &str) -> bool {
    EXTENSIONLESS_ROOT_FILENAMES.iter().any(|candidate| name.eq_ignore_ascii_case(candidate))
        || starts_with_ignore_ascii_case(name, "license-")
        || starts_with_ignore_ascii_case(name, "licence-")
        || starts_with_ignore_ascii_case(name, "dockerfile-")
        || starts_with_ignore_ascii_case(name, "makefile-")
}
