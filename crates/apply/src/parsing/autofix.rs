use core::preset::models::template::{
    DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX, OutputTemplate, ParserConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutofixResult {
    pub fixed_text: String,
    pub preamble_stripped: bool,
    pub closed_truncated_block: bool,
}

const CODE_FENCE: &str = "\x60\x60\x60";

pub(crate) fn autofix_response(raw: &str) -> AutofixResult {
    autofix_response_with_config(raw, &OutputTemplate::canonical(), &ParserConfig::canonical())
}

pub(crate) fn autofix_response_with_config(
    raw: &str,
    template: &OutputTemplate,
    config: &ParserConfig,
) -> AutofixResult {
    let header_prefix = if config.sync_with_output_template {
        template.file_header_prefix.as_str()
    } else {
        DEFAULT_FILE_HEADER_PREFIX
    };
    let footer_marker = if config.sync_with_output_template {
        template.footer_trimmed()
    } else {
        DEFAULT_FILE_FOOTER.trim_matches(['\r', '\n'])
    };
    let search_marker = config.search_replace_markers.search_header.as_str();
    let divider_marker = config.search_replace_markers.divider.as_str();
    let replace_marker = config.search_replace_markers.replace_footer.as_str();

    let normalized = normalize_glued_code_fences(raw);
    let mut current = normalized.as_str();
    let mut preamble_stripped = false;

    for _ in 0..2 {
        if let Some(start_idx) = find_first_block_offset(
            current,
            header_prefix,
            search_marker,
            config.enable_unified_diff,
        ) && start_idx > 0
        {
            preamble_stripped = true;
            current = current.get(start_idx..).unwrap_or(current);
        }

        let (unwrapped, stripped) =
            strip_outer_markdown_wrapper(current, header_prefix, search_marker);
        if stripped {
            preamble_stripped = true;
            current = unwrapped;
        } else {
            break;
        }
    }

    let mut fixed = current.to_owned();
    let mut closed_truncated_block = false;

    if let Some(last_header_pos) = rfind_last_canonical_header(&fixed, header_prefix) {
        let is_unclosed = rfind_unquoted_standalone_line_marker(&fixed, footer_marker)
            .is_none_or(|last_footer_pos| last_header_pos > last_footer_pos);
        if is_unclosed {
            if let Some(stripped_fence_tail) = fixed.trim_end().strip_suffix(CODE_FENCE)
                && stripped_fence_tail.ends_with('\n')
            {
                fixed.truncate(stripped_fence_tail.len());
            }
            if !fixed.ends_with('\n') {
                fixed.push('\n');
            }
            fixed.push_str(footer_marker);
            fixed.push('\n');
            closed_truncated_block = true;
        }
    } else {
        if config.enable_search_replace
            && let Some(last_search_pos) = rfind_line_start_marker(&fixed, search_marker)
        {
            let is_replace_unclosed = rfind_standalone_line_marker(&fixed, replace_marker)
                .is_none_or(|last_rep| last_search_pos > last_rep);
            if is_replace_unclosed {
                if !fixed.ends_with('\n') {
                    fixed.push('\n');
                }
                let has_divider = rfind_standalone_line_marker(&fixed, divider_marker)
                    .is_some_and(|div_pos| div_pos > last_search_pos);
                if !has_divider {
                    fixed.push_str(divider_marker);
                    fixed.push('\n');
                }
                fixed.push_str(replace_marker);
                fixed.push('\n');
                closed_truncated_block = true;
            }
        }

        let fence_count = fixed.lines().filter(|line| line.trim().starts_with(CODE_FENCE)).count();
        if fence_count % 2 == 1 {
            if !fixed.ends_with('\n') {
                fixed.push('\n');
            }
            fixed.push_str(CODE_FENCE);
            fixed.push('\n');
            closed_truncated_block = true;
        }
    }

    AutofixResult { fixed_text: fixed, preamble_stripped, closed_truncated_block }
}

fn contains_canonical_or_nested_blocks(
    inner: &str,
    header_prefix: &str,
    search_marker: &str,
) -> bool {
    if !header_prefix.is_empty() && rfind_line_start_marker(inner, header_prefix).is_some() {
        return true;
    }
    for &prefix in super::fenced_blocks::CREATE_HEADER_PREFIXES {
        if rfind_line_start_marker(inner, prefix).is_some() {
            return true;
        }
    }
    if !search_marker.is_empty() && rfind_line_start_marker(inner, search_marker).is_some() {
        return true;
    }
    inner.lines().any(|l| {
        let t = l.trim();
        t.starts_with("=== DELETE FILE:")
            || t.starts_with("=== DELETE:")
            || t.starts_with("=== REMOVE FILE:")
            || t.starts_with("--- a/")
            || t.starts_with("--- /dev/null")
    })
}

fn strip_outer_markdown_wrapper<'a>(
    raw: &'a str,
    header_prefix: &str,
    search_marker: &str,
) -> (&'a str, bool) {
    let trimmed = raw.trim();
    if !trimmed.starts_with(CODE_FENCE) {
        return (raw, false);
    }
    let Some(first_nl) = trimmed.find('\n') else {
        return (raw, false);
    };
    let first_line = trimmed.get(..first_nl).unwrap_or("").trim();
    let lang = first_line.strip_prefix(CODE_FENCE).unwrap_or("").trim();
    let is_wrapper_lang = lang.is_empty()
        || lang.eq_ignore_ascii_case("text")
        || lang.eq_ignore_ascii_case("plaintext")
        || lang.eq_ignore_ascii_case("markdown")
        || lang.eq_ignore_ascii_case("md");

    let inner = trimmed.get(first_nl + 1..).unwrap_or("");
    let contains_nested = contains_canonical_or_nested_blocks(inner, header_prefix, search_marker);

    if !is_wrapper_lang || !contains_nested {
        return (raw, false);
    }

    let cleaned = inner.trim_end().strip_suffix(CODE_FENCE).map_or(inner, str::trim_end);
    (cleaned, true)
}

fn rfind_last_canonical_header(haystack: &str, primary_prefix: &str) -> Option<usize> {
    let mut best = rfind_unquoted_line_start_marker(haystack, primary_prefix);
    for &prefix in super::fenced_blocks::CREATE_HEADER_PREFIXES {
        if let Some(pos) = rfind_unquoted_line_start_marker(haystack, prefix)
            && best.is_none_or(|best_pos| pos > best_pos)
        {
            best = Some(pos);
        }
    }
    best
}

fn normalize_glued_code_fences(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 32);
    for line in input.split_inclusive('\n') {
        let trimmed_end = line.trim_end_matches(['\r', '\n']);
        let trimmed = trimmed_end.trim_start();
        if !trimmed.starts_with(CODE_FENCE)
            && trimmed_end.ends_with(CODE_FENCE)
            && !trimmed_end.contains('\"')
            && !trimmed_end.contains('\'')
        {
            let without_fence =
                trimmed_end.strip_suffix(CODE_FENCE).unwrap_or(trimmed_end).trim_end();
            out.push_str(without_fence);
            out.push('\n');
            out.push_str(CODE_FENCE);
            out.push('\n');
        } else {
            out.push_str(line);
        }
    }
    out
}

pub(crate) fn rfind_line_start_marker(haystack: &str, marker: &str) -> Option<usize> {
    if marker.is_empty() {
        return None;
    }
    let bytes = haystack.as_bytes();
    let mut search_end = haystack.len();
    while let Some(pos) = haystack.get(..search_end).and_then(|s| s.rfind(marker)) {
        if pos == 0 || bytes.get(pos - 1).copied() == Some(b'\n') {
            return Some(pos);
        }
        search_end = pos;
    }
    None
}

fn rfind_unquoted_line_start_marker(haystack: &str, marker: &str) -> Option<usize> {
    if marker.is_empty() {
        return None;
    }
    let bytes = haystack.as_bytes();
    let mut search_end = haystack.len();
    while let Some(pos) = haystack.get(..search_end).and_then(|s| s.rfind(marker)) {
        let at_line_start = pos == 0 || bytes.get(pos - 1).copied() == Some(b'\n');
        if at_line_start && !is_offset_inside_rust_string(haystack, pos) {
            return Some(pos);
        }
        search_end = pos;
    }
    None
}

pub(crate) fn rfind_standalone_line_marker(haystack: &str, marker: &str) -> Option<usize> {
    if marker.is_empty() {
        return None;
    }
    let bytes = haystack.as_bytes();
    let mut search_end = haystack.len();
    while let Some(pos) = haystack.get(..search_end).and_then(|s| s.rfind(marker)) {
        let at_line_start = pos == 0 || bytes.get(pos - 1).copied() == Some(b'\n');
        if at_line_start && is_clean_line_tail(haystack, pos + marker.len()) {
            return Some(pos);
        }
        search_end = pos;
    }
    None
}

fn rfind_unquoted_standalone_line_marker(haystack: &str, marker: &str) -> Option<usize> {
    if marker.is_empty() {
        return None;
    }
    let bytes = haystack.as_bytes();
    let mut search_end = haystack.len();
    while let Some(pos) = haystack.get(..search_end).and_then(|s| s.rfind(marker)) {
        let at_line_start = pos == 0 || bytes.get(pos - 1).copied() == Some(b'\n');
        if at_line_start
            && is_clean_line_tail(haystack, pos + marker.len())
            && !is_offset_inside_rust_string(haystack, pos)
        {
            return Some(pos);
        }
        search_end = pos;
    }
    None
}

#[inline]
pub(crate) fn is_clean_line_tail(haystack: &str, after_marker_pos: usize) -> bool {
    let tail = haystack.get(after_marker_pos..).unwrap_or("");
    let line_rem = tail.split('\n').next().unwrap_or(tail);
    line_rem.trim_matches(['\r', ' ', '\t']).is_empty()
}

pub(crate) fn is_offset_inside_rust_string(haystack: &str, target_offset: usize) -> bool {
    let after = haystack.get(target_offset..).unwrap_or("");
    let next_lines_have_closing_quote_semi = after
        .lines()
        .take(6)
        .any(|l| l.trim_end().ends_with("\";") || l.trim_end().ends_with("\")"));
    if !next_lines_have_closing_quote_semi {
        return false;
    }

    let window_start = target_offset.saturating_sub(2048);
    let slice = haystack.get(window_start..target_offset).unwrap_or("");
    let bytes = slice.as_bytes();
    let mut in_quotes = false;
    let mut escaped = false;
    for &b in bytes {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_quotes = false;
            }
        } else if b == b'"' {
            in_quotes = true;
        }
    }
    in_quotes
}

fn find_first_block_offset(
    raw: &str,
    header_prefix: &str,
    search_marker: &str,
    enable_unified_diff: bool,
) -> Option<usize> {
    let mut offset = 0;
    for line in raw.split_inclusive('\n') {
        let trimmed = line.trim();
        if (!header_prefix.is_empty() && trimmed.starts_with(header_prefix))
            || trimmed.starts_with("=== CREATE FILE:")
            || trimmed.starts_with("=== NEW FILE:")
            || trimmed.starts_with("=== DELETE FILE:")
            || trimmed.starts_with("=== DELETE:")
            || trimmed.starts_with("=== REMOVE FILE:")
            || (trimmed.starts_with('#')
                && (trimmed.contains('`') || trimmed.contains('/') || trimmed.contains('.')))
            || trimmed.starts_with(CODE_FENCE)
            || (!search_marker.is_empty() && trimmed.starts_with(search_marker))
            || (enable_unified_diff
                && (trimmed.starts_with("--- a/") || trimmed.starts_with("--- /dev/null")))
            || trimmed.starts_with("**`")
        {
            let leading_ws = line.len() - line.trim_start().len();
            return Some(offset + leading_ws);
        }
        offset += line.len();
    }
    None
}
