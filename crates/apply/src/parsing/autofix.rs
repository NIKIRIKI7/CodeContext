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

    let (unwrapped_initial, outer_fence_stripped_1) =
        strip_outer_markdown_wrapper(raw, header_prefix, footer_marker);
    let normalized_fences = normalize_glued_code_fences(unwrapped_initial);

    let start_idx_1 = find_first_block_offset(
        &normalized_fences,
        header_prefix,
        search_marker,
        config.enable_unified_diff,
    )
    .unwrap_or(0);
    let slice_after_preamble = normalized_fences.get(start_idx_1..).unwrap_or(&normalized_fences);

    let (unwrapped_second, outer_fence_stripped_2) =
        strip_outer_markdown_wrapper(slice_after_preamble, header_prefix, footer_marker);
    let start_idx_2 = if outer_fence_stripped_2 {
        find_first_block_offset(
            unwrapped_second,
            header_prefix,
            search_marker,
            config.enable_unified_diff,
        )
        .unwrap_or(0)
    } else {
        0
    };

    let preamble_stripped =
        outer_fence_stripped_1 || outer_fence_stripped_2 || start_idx_1 > 0 || start_idx_2 > 0;
    let working_slice = unwrapped_second.get(start_idx_2..).unwrap_or(unwrapped_second);

    let mut fixed = working_slice.to_owned();
    let mut closed_truncated_block = false;

    if config.enable_search_replace
        && let Some(last_search_pos) = rfind_line_start_marker(&fixed, search_marker)
    {
        let is_replace_unclosed = rfind_standalone_line_marker(&fixed, replace_marker)
            .is_none_or(|last_rep| last_search_pos > last_rep);
        if is_replace_unclosed {
            if let Some(stripped_fence_tail) = fixed.trim_end().strip_suffix(CODE_FENCE)
                && stripped_fence_tail.ends_with('\n')
            {
                fixed.truncate(stripped_fence_tail.len());
            }
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

    if let Some(last_header_pos) = rfind_unquoted_line_start_marker(&fixed, header_prefix) {
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

fn strip_outer_markdown_wrapper<'a>(
    raw: &'a str,
    header_prefix: &str,
    footer_marker: &str,
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
    let has_header = rfind_line_start_marker(inner, header_prefix).is_some()
        || inner.lines().any(|l| {
            let t = l.trim();
            t.starts_with("=== CREATE") || t.starts_with("=== NEW") || t.starts_with("=== DELETE")
        });
    let has_both_header_and_footer =
        has_header && rfind_line_start_marker(inner, footer_marker).is_some();

    if !is_wrapper_lang || (!has_header && !has_both_header_and_footer) {
        return (raw, false);
    }

    let cleaned = inner.trim_end().strip_suffix(CODE_FENCE).map_or(inner, str::trim_end);
    (cleaned, true)
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
            let without_fence = trimmed_end.trim_end_matches('`').trim_end();
            if !without_fence.is_empty() {
                out.push_str(without_fence);
                out.push('\n');
            }
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
    let slice = haystack.get(..target_offset).unwrap_or(haystack);
    let after = haystack.get(target_offset..).unwrap_or("");
    let next_lines_have_closing_quote_semi = after
        .lines()
        .take(6)
        .any(|l| l.trim_end().ends_with("\";") || l.trim_end().ends_with("\")"));
    if !next_lines_have_closing_quote_semi {
        return false;
    }

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
        if !header_prefix.is_empty() && line.starts_with(header_prefix) {
            return Some(offset);
        }
        let trimmed = line.trim();
        if trimmed.starts_with("=== CREATE FILE:")
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
