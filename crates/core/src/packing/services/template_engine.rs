use crate::preset::models::template::OutputTemplate;
use crate::shared_kernel::fs::file_kind::{CommentStyle, FileKind, MinifyStyle};

const TREE_HEADER_OPEN: &str = concat!("=== ", "PROJECT TREE ===\n");
const TREE_HEADER_CLOSE: &str = concat!("=== ", "END PROJECT TREE ===\n\n");
const TREE_LINE_PREFIX: &str = "- ";

#[derive(Debug, Clone, Copy)]
struct IndentedCommentStrategy {
    prefix: &'static str,
    marker_byte: u8,
    is_double_byte: bool,
    preserve_shebang: bool,
}

impl IndentedCommentStrategy {
    #[inline]
    const fn for_style(style: CommentStyle) -> Option<Self> {
        match style {
            CommentStyle::Hash => Some(Self {
                prefix: "#",
                marker_byte: b'#',
                is_double_byte: false,
                preserve_shebang: true,
            }),
            CommentStyle::SqlDash => Some(Self {
                prefix: "--",
                marker_byte: b'-',
                is_double_byte: true,
                preserve_shebang: false,
            }),
            CommentStyle::Semicolon => Some(Self {
                prefix: ";",
                marker_byte: b';',
                is_double_byte: false,
                preserve_shebang: false,
            }),
            CommentStyle::CStyle => Some(Self {
                prefix: "//",
                marker_byte: b'/',
                is_double_byte: true,
                preserve_shebang: false,
            }),
            CommentStyle::Xml | CommentStyle::None => None,
        }
    }
}

pub trait TextSink {
    fn push_str(&mut self, s: &str);
    fn push_char(&mut self, ch: char);
}

#[derive(Debug)]
pub struct PlainTextSink<'a> {
    out: &'a mut String,
}

impl<'a> PlainTextSink<'a> {
    #[inline]
    pub const fn new(out: &'a mut String) -> Self {
        Self { out }
    }
}

impl TextSink for PlainTextSink<'_> {
    #[inline]
    fn push_str(&mut self, s: &str) {
        self.out.push_str(s);
    }

    #[inline]
    fn push_char(&mut self, ch: char) {
        self.out.push(ch);
    }
}

#[derive(Debug)]
pub struct LfNormalizingSink<'a, S> {
    inner: &'a mut S,
}

impl<'a, S: TextSink> LfNormalizingSink<'a, S> {
    #[inline]
    pub const fn new(inner: &'a mut S) -> Self {
        Self { inner }
    }
}

impl<S: TextSink> TextSink for LfNormalizingSink<'_, S> {
    #[inline]
    fn push_str(&mut self, s: &str) {
        for chunk in s.split('\r') {
            self.inner.push_str(chunk);
        }
    }

    #[inline]
    fn push_char(&mut self, ch: char) {
        if ch != '\r' {
            self.inner.push_char(ch);
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TemplateEngine;

impl TemplateEngine {
    #[inline]
    pub fn calculate_tree_header_len<P, C>(files: &[(P, C)]) -> usize
    where
        P: AsRef<str>,
    {
        if files.is_empty() {
            return 0;
        }
        let lines_bytes: usize =
            files.iter().map(|(path, _)| TREE_LINE_PREFIX.len() + path.as_ref().len() + 1).sum();
        TREE_HEADER_OPEN.len() + lines_bytes + TREE_HEADER_CLOSE.len()
    }

    #[inline]
    pub fn append_tree_header<P, C>(files: &[(P, C)], out: &mut String)
    where
        P: AsRef<str>,
    {
        if files.is_empty() {
            return;
        }
        out.push_str(TREE_HEADER_OPEN);
        for (path, _) in files {
            out.push_str(TREE_LINE_PREFIX);
            out.push_str(path.as_ref());
            out.push('\n');
        }
        out.push_str(TREE_HEADER_CLOSE);
    }

    #[inline]
    pub fn calculate_exact_capacity<P, C>(
        files: &[(P, C)],
        template: &OutputTemplate,
        include_tree_in_this_chunk: bool,
    ) -> usize
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        let tree_len = if include_tree_in_this_chunk && template.include_file_tree {
            Self::calculate_tree_header_len(files)
        } else {
            0
        };
        let fixed_marker_len = template.fixed_marker_bytes_per_file();
        let files_len: usize = files
            .iter()
            .map(|(path, content)| path.as_ref().len() + content.as_ref().len() + fixed_marker_len)
            .sum();
        template.preamble.len() + tree_len + files_len + template.postamble.len()
    }

    #[inline]
    pub fn format_files_preallocated<P, C>(
        files: &[(P, C)],
        template: &OutputTemplate,
        include_tree_in_this_chunk: bool,
        minify: bool,
        normalize_lf: bool,
    ) -> String
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        let capacity = Self::calculate_exact_capacity(files, template, include_tree_in_this_chunk);
        let mut output = String::with_capacity(capacity);

        if !template.preamble.is_empty() {
            output.push_str(&template.preamble);
        }
        if include_tree_in_this_chunk && template.include_file_tree {
            Self::append_tree_header(files, &mut output);
        }
        for (path, content) in files {
            let path_str = path.as_ref();
            output.push_str(&template.file_header_prefix);
            output.push_str(path_str);
            output.push_str(&template.file_header_suffix);
            Self::append_transformed_text(
                path_str,
                content.as_ref(),
                &mut output,
                minify,
                normalize_lf,
            );
            output.push_str(&template.file_footer);
        }
        if !template.postamble.is_empty() {
            output.push_str(&template.postamble);
        }
        output
    }

    #[inline]
    pub fn is_indentation_sensitive_path(rel_path: &str) -> bool {
        FileKind::syntax_profile_for_path(rel_path).minify_style
            == MinifyStyle::IndentationPreserving
    }

    #[inline]
    pub fn append_minified_to_sink<S: TextSink>(rel_path: &str, raw: &str, sink: &mut S) {
        let profile = FileKind::syntax_profile_for_path(rel_path);
        match profile.minify_style {
            MinifyStyle::IndentationPreserving => {
                append_minified_indented(raw, profile.comment_style, sink);
            }
            MinifyStyle::SingleLineCStyle => {
                append_minified_single_line(raw, profile.comment_style, sink);
            }
        }
    }

    #[inline]
    pub fn append_minified_to_buffer(rel_path: &str, raw: &str, out: &mut String) {
        let mut sink = PlainTextSink::new(out);
        Self::append_minified_to_sink(rel_path, raw, &mut sink);
    }

    #[inline]
    pub fn append_transformed_text(
        rel_path: &str,
        raw: &str,
        out: &mut String,
        minify: bool,
        normalize_lf: bool,
    ) {
        let mut base_sink = PlainTextSink::new(out);
        if normalize_lf {
            let mut lf_sink = LfNormalizingSink::new(&mut base_sink);
            if minify {
                Self::append_minified_to_sink(rel_path, raw, &mut lf_sink);
            } else {
                lf_sink.push_str(raw);
            }
        } else if minify {
            Self::append_minified_to_sink(rel_path, raw, &mut base_sink);
        } else {
            base_sink.push_str(raw);
        }
    }
}

fn append_minified_single_line<S: TextSink>(raw: &str, comment_style: CommentStyle, out: &mut S) {
    let mut in_block_comment = false;
    let mut has_written_token = false;
    let (block_open, block_close) = match comment_style {
        CommentStyle::Xml => ("<!--", "-->"),
        CommentStyle::CStyle
        | CommentStyle::Hash
        | CommentStyle::SqlDash
        | CommentStyle::Semicolon
        | CommentStyle::None => ("/*", "*/"),
    };

    for raw_line in raw.lines() {
        let Some(clean_line) = extract_clean_block_or_c_line(
            raw_line,
            &mut in_block_comment,
            comment_style,
            block_open,
            block_close,
        ) else {
            continue;
        };
        if has_written_token {
            out.push_char(' ');
        }
        out.push_str(clean_line);
        has_written_token = true;
    }
}

fn append_minified_indented<S: TextSink>(raw: &str, comment_style: CommentStyle, out: &mut S) {
    let strategy_opt = IndentedCommentStrategy::for_style(comment_style);
    let mut has_written_line = false;

    for (line_idx, raw_line) in raw.lines().enumerate() {
        let trimmed_end = raw_line.trim_end();
        let trimmed_both = trimmed_end.trim_start();
        if trimmed_both.is_empty() {
            continue;
        }
        let clean_line = if let Some(strat) = strategy_opt {
            let is_shebang =
                strat.preserve_shebang && line_idx == 0 && trimmed_both.starts_with("#!");
            if !is_shebang && trimmed_both.starts_with(strat.prefix) {
                continue;
            }
            if is_shebang || !trimmed_end.contains(strat.prefix) {
                trimmed_end
            } else {
                let comment_pos = if strat.is_double_byte {
                    find_unquoted_double_char(trimmed_end, strat.marker_byte)
                } else {
                    find_unquoted_char_comment(trimmed_end, strat.marker_byte)
                };
                if let Some(pos) = comment_pos {
                    trimmed_end.get(..pos).unwrap_or(trimmed_end).trim_end()
                } else {
                    trimmed_end
                }
            }
        } else {
            trimmed_end
        };

        if clean_line.trim_start().is_empty() {
            continue;
        }
        if has_written_line {
            out.push_char('\n');
        }
        out.push_str(clean_line);
        has_written_line = true;
    }
}

#[inline]
fn extract_clean_block_or_c_line<'a>(
    raw_line: &'a str,
    in_block_comment: &mut bool,
    comment_style: CommentStyle,
    block_open: &str,
    block_close: &str,
) -> Option<&'a str> {
    let mut line = raw_line.trim();
    if line.is_empty() {
        return None;
    }
    if *in_block_comment {
        let end_pos = line.find(block_close)?;
        *in_block_comment = false;
        line = line.get(end_pos + block_close.len()..).unwrap_or("").trim();
        if line.is_empty() {
            return None;
        }
    }
    if line.starts_with(block_open) {
        if let Some(end_pos) = line.find(block_close) {
            line = line.get(end_pos + block_close.len()..).unwrap_or("").trim();
            if line.is_empty() {
                return None;
            }
        } else {
            *in_block_comment = true;
            return None;
        }
    }
    if comment_style == CommentStyle::CStyle {
        if line.starts_with("//") {
            return None;
        }
        if line.contains("//")
            && let Some(comment_pos) = find_unquoted_double_char(line, b'/')
        {
            line = line.get(..comment_pos).unwrap_or(line).trim_end();
        }
    }
    (!line.is_empty()).then_some(line)
}

#[inline]
fn find_unquoted_double_char(line: &str, marker_byte: u8) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let mut in_quotes = false;
    let mut escaped = false;
    let mut idx = 0;
    while idx < bytes.len() {
        let b = bytes.get(idx).copied().unwrap_or(0);
        if in_quotes {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'\x22' {
                in_quotes = false;
            }
            idx += 1;
            continue;
        }
        if b == b'\x22' {
            in_quotes = true;
            idx += 1;
            continue;
        }
        if b == marker_byte && bytes.get(idx + 1).copied() == Some(marker_byte) {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

#[inline]
fn find_unquoted_char_comment(line: &str, marker_byte: u8) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut quote_char: Option<u8> = None;
    let mut escaped = false;
    for (idx, &b) in bytes.iter().enumerate() {
        if let Some(q) = quote_char {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == q {
                quote_char = None;
            }
            continue;
        }
        if b == b'\x22' || b == b'\'' {
            quote_char = Some(b);
            continue;
        }
        if b == marker_byte {
            return Some(idx);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::TemplateEngine;
    use crate::preset::models::template::OutputTemplate;

    #[test]
    fn exact_capacity_matches_actual_length_with_and_without_tree() {
        let files =
            vec![("src/main.rs", "fn main() {}\n"), ("Cargo.toml", "[package]\nname = \"app\"")];
        let tpl_no_tree = OutputTemplate::canonical();
        let expected_no_tree =
            TemplateEngine::calculate_exact_capacity(&files, &tpl_no_tree, false);
        let out_no_tree =
            TemplateEngine::format_files_preallocated(&files, &tpl_no_tree, false, false, false);
        assert_eq!(out_no_tree.len(), expected_no_tree);
        assert_eq!(out_no_tree.capacity(), expected_no_tree);

        let tpl_with_tree = OutputTemplate::canonical().with_file_tree(true);
        let expected_with_tree =
            TemplateEngine::calculate_exact_capacity(&files, &tpl_with_tree, true);
        let out_with_tree =
            TemplateEngine::format_files_preallocated(&files, &tpl_with_tree, true, false, false);
        assert_eq!(out_with_tree.len(), expected_with_tree);
        assert_eq!(out_with_tree.capacity(), expected_with_tree);
        assert!(out_with_tree.starts_with(concat!("=== ", "PROJECT TREE ===\n- src/main.rs\n")));
    }

    #[test]
    fn rust_minification_collapses_into_single_line_and_strips_comments() {
        let code = concat!(
            "pub fn connect() -> &'static str {\n",
            "    let url = \"https://github.com/rust-lang\"; // Хвостовой комментарий\n",
            "    url\n",
            "}\n"
        );
        let files = vec![("src/net.rs", code)];
        let tpl = OutputTemplate::canonical();
        let minified = TemplateEngine::format_files_preallocated(&files, &tpl, false, true, false);
        let expected = concat!(
            "=== ",
            "FILE: src/net.rs ===\n",
            "pub fn connect() -> &'static str { let url = \"https://github.com/rust-lang\"; url }\n",
            "=== ",
            "END FILE ===\n\n"
        );
        assert_eq!(minified, expected);
    }

    #[test]
    fn python_minification_preserves_indentation_and_newlines_while_stripping_hash_comments() {
        let py_code = concat!(
            "#!/usr/bin/env python3\n",
            "# Инициализация\n",
            "def calc(items: list[int]) -> int:\n",
            "    # Проверка на пустоту\n",
            "    if not items:\n",
            "        return 0 # пусто\n\n",
            "    total = 0\n",
            "    for x in items:\n",
            "        total += x\n",
            "    return total\n"
        );
        let files = vec![("scripts/app.py", py_code)];
        let tpl = OutputTemplate::canonical();
        let minified = TemplateEngine::format_files_preallocated(&files, &tpl, false, true, false);
        let expected = concat!(
            "=== ",
            "FILE: scripts/app.py ===\n",
            "#!/usr/bin/env python3\n",
            "def calc(items: list[int]) -> int:\n",
            "    if not items:\n",
            "        return 0\n",
            "    total = 0\n",
            "    for x in items:\n",
            "        total += x\n",
            "    return total\n",
            "=== ",
            "END FILE ===\n\n"
        );
        assert_eq!(minified, expected);
    }

    #[test]
    fn decorator_sink_composes_minification_and_crlf_normalization() {
        let raw_crlf = "pub fn demo() {\r\n    let x = 1; // comment\r\n}\r\n";
        let mut out = String::with_capacity(64);
        TemplateEngine::append_transformed_text("src/lib.rs", raw_crlf, &mut out, true, true);
        assert_eq!(out, "pub fn demo() { let x = 1; }");
        assert!(!out.contains('\r'));
    }
}
