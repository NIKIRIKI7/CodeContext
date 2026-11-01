use std::ops::Range;
use std::sync::Arc;

use core::preset::models::template::{
    DEFAULT_DIVIDER_MARKER, DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX,
    DEFAULT_REPLACE_MARKER, DEFAULT_SEARCH_MARKER, OutputTemplate, ParserConfig,
};
use gpui::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    UniformListScrollHandle, Window, div, px, uniform_list,
};

use crate::locale::{RU_LABELS, UiLabels};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme};

pub const DEFAULT_MAX_DISPLAY_LINE_BYTES: usize = 256;
const MIN_DISPLAY_LINE_BYTES: usize = 16;
const ESTIMATED_AVG_BYTES_PER_LINE: usize = 45;
const TRUNCATED_LINE_ELLIPSIS: &str = " …";
const CODE_FENCE: &str = "\x60\x60\x60";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineHighlightMode {
    #[default]
    PackedContext,
    UnifiedDiff,
    RawAiStream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightMarkers {
    pub file_header_prefix: SharedString,
    pub file_footer_marker: SharedString,
    pub search_header: SharedString,
    pub divider: SharedString,
    pub replace_footer: SharedString,
}

impl Default for HighlightMarkers {
    fn default() -> Self {
        Self {
            file_header_prefix: SharedString::from(DEFAULT_FILE_HEADER_PREFIX),
            file_footer_marker: SharedString::from(
                DEFAULT_FILE_FOOTER.trim_matches(['\r', '\n']).to_owned(),
            ),
            search_header: SharedString::from(DEFAULT_SEARCH_MARKER),
            divider: SharedString::from(DEFAULT_DIVIDER_MARKER),
            replace_footer: SharedString::from(DEFAULT_REPLACE_MARKER),
        }
    }
}

impl HighlightMarkers {
    #[inline]
    pub fn from_template_and_parser(template: &OutputTemplate, parser: &ParserConfig) -> Self {
        Self {
            file_header_prefix: SharedString::from(template.file_header_prefix.clone()),
            file_footer_marker: SharedString::from(template.footer_trimmed().to_owned()),
            search_header: SharedString::from(parser.search_replace_markers.search_header.clone()),
            divider: SharedString::from(parser.search_replace_markers.divider.clone()),
            replace_footer: SharedString::from(
                parser.search_replace_markers.replace_footer.clone(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualTextBuffer {
    content: Arc<str>,
    line_offsets: Arc<[u32]>,
    max_display_line_bytes: usize,
}

impl Default for VirtualTextBuffer {
    fn default() -> Self {
        Self::new("")
    }
}

impl VirtualTextBuffer {
    pub fn new(raw: impl Into<Arc<str>>) -> Self {
        let content: Arc<str> = raw.into();
        let bytes = content.as_bytes();
        let estimated_lines = (bytes.len() / ESTIMATED_AVG_BYTES_PER_LINE).max(16) + 2;
        let mut offsets = Vec::with_capacity(estimated_lines);
        offsets.push(0_u32);
        for (idx, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                let next_pos = u32::try_from(idx + 1).unwrap_or(u32::MAX);
                offsets.push(next_pos);
            }
        }
        Self {
            content,
            line_offsets: Arc::from(offsets),
            max_display_line_bytes: DEFAULT_MAX_DISPLAY_LINE_BYTES,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_max_display_line_bytes(mut self, max_bytes: usize) -> Self {
        self.max_display_line_bytes =
            if max_bytes < MIN_DISPLAY_LINE_BYTES { MIN_DISPLAY_LINE_BYTES } else { max_bytes };
        self
    }

    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.content.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    #[inline]
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() { 0 } else { self.line_offsets.len() }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.content
    }

    #[inline]
    pub fn line_offsets(&self) -> &[u32] {
        &self.line_offsets
    }

    #[inline]
    pub fn raw_line(&self, line_idx: usize) -> &str {
        let Some(&start_u32) = self.line_offsets.get(line_idx) else {
            return "";
        };
        let start = start_u32 as usize;
        let end = self
            .line_offsets
            .get(line_idx + 1)
            .map_or(self.content.len(), |&next_u32| next_u32 as usize);
        let raw_slice = self.content.get(start..end).unwrap_or("");
        let without_lf = raw_slice.strip_suffix('\n').unwrap_or(raw_slice);
        without_lf.strip_suffix('\r').unwrap_or(without_lf)
    }

    #[inline]
    pub fn display_line(&self, line_idx: usize) -> (&str, bool) {
        let line = self.raw_line(line_idx);
        if line.len() <= self.max_display_line_bytes {
            return (line, false);
        }
        let safe_end = floor_utf8_char_boundary(line, self.max_display_line_bytes);
        let prefix = line.get(..safe_end).unwrap_or("");
        (prefix, true)
    }
}

#[inline]
fn floor_utf8_char_boundary(s: &str, max_bytes: usize) -> usize {
    if max_bytes >= s.len() {
        return s.len();
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[derive(Debug, Clone, IntoElement)]
pub struct VirtualTextView {
    id: ElementId,
    buffer: VirtualTextBuffer,
    highlight_mode: LineHighlightMode,
    markers: HighlightMarkers,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    scroll_handle: Option<UniformListScrollHandle>,
}

impl VirtualTextView {
    #[inline]
    pub fn new(id: impl Into<ElementId>, buffer: VirtualTextBuffer) -> Self {
        Self {
            id: id.into(),
            buffer,
            highlight_mode: LineHighlightMode::PackedContext,
            markers: HighlightMarkers::default(),
            labels: &RU_LABELS,
            theme: active_theme(),
            scroll_handle: None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_highlight_mode(mut self, mode: LineHighlightMode) -> Self {
        self.highlight_mode = mode;
        self
    }

    #[must_use]
    #[inline]
    pub fn with_markers(mut self, markers: HighlightMarkers) -> Self {
        self.markers = markers;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_context(
        mut self,
        labels: &'static UiLabels,
        theme: &'static ThemeTokens,
    ) -> Self {
        self.labels = labels;
        self.theme = theme;
        self
    }

    #[must_use]
    #[inline]
    pub fn track_scroll(mut self, handle: UniformListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }
}

impl RenderOnce for VirtualTextView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let colors = &theme.colors;
        let total_lines = self.buffer.line_count();

        if total_lines == 0 {
            let empty_hint = match self.highlight_mode {
                LineHighlightMode::RawAiStream => self.labels.applier_empty_diff_hint,
                LineHighlightMode::PackedContext | LineHighlightMode::UnifiedDiff => {
                    self.labels.empty_preview_hint
                }
            };
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(theme.spacing.xxl))
                .bg(colors.bg_app)
                .child(
                    div()
                        .max_w(px(360.0))
                        .p(px(theme.spacing.xxl))
                        .rounded(px(theme.radius.lg))
                        .border_1()
                        .border_color(colors.border)
                        .bg(colors.bg_panel)
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_muted)
                        .child(empty_hint),
                )
                .into_any_element();
        }

        let buffer = self.buffer;
        let mode = self.highlight_mode;
        let markers = self.markers;

        let list = uniform_list(
            self.id,
            total_lines,
            move |range: Range<usize>, _window: &mut Window, _cx: &mut App| {
                let mut rows = Vec::with_capacity(range.len());
                for line_idx in range {
                    let (line_text, is_clamped) = buffer.display_line(line_idx);
                    let line_number = SharedString::from(format!("{:>4}", line_idx + 1));
                    let (line_fg, row_bg) =
                        classify_line_by_mode(line_text, mode, &markers, colors);
                    let rendered_text = if is_clamped {
                        SharedString::from(format!("{line_text}{TRUNCATED_LINE_ELLIPSIS}"))
                    } else {
                        SharedString::from(line_text.to_owned())
                    };
                    rows.push(
                        div()
                            .w_full()
                            .h(px(theme.layout.code_line_height))
                            .bg(row_bg)
                            .flex()
                            .flex_row()
                            .items_center()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .font_family(FONT_FAMILY_MONO)
                            .child(
                                div()
                                    .w(px(theme.layout.line_number_width))
                                    .h_full()
                                    .pr(px(theme.spacing.md))
                                    .border_r_1()
                                    .border_color(colors.border)
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .flex_shrink_0()
                                    .text_xs()
                                    .text_color(colors.text_muted)
                                    .child(line_number),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .pl(px(theme.spacing.xl))
                                    .pr(px(theme.spacing.md))
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_xs()
                                    .text_color(line_fg)
                                    .child(rendered_text),
                            ),
                    );
                }
                rows
            },
        )
        .size_full();

        let tracked_list =
            if let Some(handle) = self.scroll_handle { list.track_scroll(handle) } else { list };

        div().size_full().bg(colors.bg_app).overflow_hidden().child(tracked_list).into_any_element()
    }
}

#[inline]
fn classify_line_by_mode(
    raw_line: &str,
    mode: LineHighlightMode,
    markers: &HighlightMarkers,
    colors: &crate::theme::ColorTokens,
) -> (gpui::Rgba, gpui::Rgba) {
    match mode {
        LineHighlightMode::UnifiedDiff => {
            if raw_line.starts_with(concat!("@", "@"))
                || raw_line.starts_with("--- ")
                || raw_line.starts_with("+++ ")
            {
                (colors.cyan, colors.badge_accent.bg)
            } else if raw_line.starts_with('+') {
                (colors.success, colors.badge_success.bg)
            } else if raw_line.starts_with('-') {
                (colors.danger, colors.badge_danger.bg)
            } else {
                (colors.text_primary, colors.bg_app)
            }
        }
        LineHighlightMode::RawAiStream => {
            let trimmed = raw_line.trim_start();
            if trimmed.starts_with(markers.file_header_prefix.as_ref())
                || trimmed.starts_with(concat!("=== ", "FILE:"))
                || trimmed.starts_with(concat!("=== ", "CREATE FILE:"))
                || trimmed.starts_with(concat!("=== ", "NEW FILE:"))
            {
                (colors.cyan, colors.bg_app)
            } else if trimmed.starts_with(markers.file_footer_marker.as_ref())
                || trimmed.starts_with(concat!("=== ", "END FILE"))
                || trimmed.starts_with(markers.search_header.as_ref())
                || trimmed.starts_with("<<<<<")
                || trimmed.starts_with(markers.divider.as_ref())
                || trimmed.starts_with("=====")
                || trimmed.starts_with(markers.replace_footer.as_ref())
                || trimmed.starts_with(">>>>>")
                || trimmed.starts_with(CODE_FENCE)
            {
                (colors.text_muted, colors.bg_app)
            } else {
                classify_code_line_colors(trimmed, markers, colors)
            }
        }
        LineHighlightMode::PackedContext => {
            classify_code_line_colors(raw_line.trim_start(), markers, colors)
        }
    }
}

#[inline]
fn classify_code_line_colors(
    trimmed: &str,
    markers: &HighlightMarkers,
    colors: &crate::theme::ColorTokens,
) -> (gpui::Rgba, gpui::Rgba) {
    if trimmed.starts_with(markers.file_header_prefix.as_ref())
        || trimmed.starts_with(markers.file_footer_marker.as_ref())
        || trimmed.starts_with(concat!("=== ", "FILE:"))
        || trimmed.starts_with(concat!("=== ", "END FILE"))
        || trimmed.starts_with(concat!("=== ", "PROJECT TREE"))
        || trimmed.starts_with(concat!("=== ", "END PROJECT TREE"))
    {
        (colors.cyan, colors.badge_accent.bg)
    } else if trimmed.starts_with("//") || trimmed.starts_with('#') {
        (colors.text_secondary, colors.bg_app)
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        (colors.cyan, colors.bg_app)
    } else if trimmed.starts_with("pub fn ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("impl ")
    {
        (colors.warning, colors.bg_app)
    } else if trimmed.starts_with("use ") || trimmed.starts_with("pub mod ") {
        (colors.kind_config, colors.bg_app)
    } else {
        (colors.text_primary, colors.bg_app)
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualTextBuffer;

    #[test]
    fn indexes_lf_and_crlf_accurately_with_zero_copy_slices() {
        let raw = concat!(
            "=== ",
            "FILE: src/main.rs ===\r\nfn main() {\n    println!(\"Привет, мир!\");\n}\n=== ",
            "END FILE ==="
        );
        let buf = VirtualTextBuffer::new(raw);
        assert_eq!(buf.line_count(), 5);
        assert_eq!(buf.raw_line(0), concat!("=== ", "FILE: src/main.rs ==="));
        assert_eq!(buf.raw_line(1), "fn main() {");
        assert_eq!(buf.raw_line(2), "    println!(\"Привет, мир!\");");
        assert_eq!(buf.raw_line(3), "}");
        assert_eq!(buf.raw_line(4), concat!("=== ", "END FILE ==="));
        assert_eq!(buf.raw_line(999), "");
    }

    #[test]
    fn clamps_long_multibyte_utf8_lines_on_valid_char_boundary() {
        let cyrillic_line = "Ы".repeat(400);
        let buf = VirtualTextBuffer::new(cyrillic_line).with_max_display_line_bytes(51);
        let (sliced, is_clamped) = buf.display_line(0);
        assert!(is_clamped);
        assert_eq!(sliced.len(), 50);
        assert_eq!(sliced.chars().count(), 25);
    }
}
