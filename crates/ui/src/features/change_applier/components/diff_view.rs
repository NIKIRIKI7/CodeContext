use core::shared_kernel::fs::line_ending::LineEnding;
use gpui::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div, px,
};

use crate::locale::{AppLocale, UiLabels};
use crate::shared::containers::virtual_text::{
    LineHighlightMode, VirtualTextBuffer, VirtualTextView,
};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme};

#[derive(Debug, Clone)]
pub struct DiffViewMeta {
    pub file_path: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub line_ending: LineEnding,
    pub is_truncated: bool,
    pub is_sandbox_safe: bool,
    pub has_conflict: bool,
}

#[derive(Debug, Clone, IntoElement)]
pub struct DiffViewComponent {
    meta: DiffViewMeta,
    buffer: VirtualTextBuffer,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
}

impl DiffViewComponent {
    #[inline]
    pub fn new(meta: DiffViewMeta, buffer: VirtualTextBuffer) -> Self {
        Self { meta, buffer, labels: AppLocale::Ru.labels(), theme: active_theme() }
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
}

impl RenderOnce for DiffViewComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let labels = self.labels;

        if self.buffer.is_empty() {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(theme.spacing.xxl))
                .bg(theme.colors.bg_app)
                .child(
                    div()
                        .max_w(px(380.0))
                        .p(px(theme.spacing.xxl))
                        .rounded(px(theme.radius.lg))
                        .border_1()
                        .border_color(theme.colors.border)
                        .bg(theme.colors.bg_panel)
                        .flex()
                        .flex_col()
                        .gap(px(theme.spacing.sm))
                        .child(
                            div()
                                .font_family(FONT_FAMILY_UI)
                                .text_xs()
                                .text_color(theme.colors.text_primary)
                                .child(labels.applier_no_parsed_files),
                        )
                        .child(
                            div()
                                .font_family(FONT_FAMILY_UI)
                                .text_xs()
                                .text_color(theme.colors.text_muted)
                                .child(labels.applier_empty_diff_hint),
                        ),
                )
                .into_any_element();
        }

        let alert_strip = render_exception_alert_strip(&self.meta, labels, theme);

        let mut right_meta = div().flex().flex_row().items_center().gap(px(theme.spacing.sm));
        if self.meta.line_ending == LineEnding::Crlf {
            right_meta = right_meta.child(
                Badge::new(labels.applier_encoding_crlf, BadgeVariant::Neutral).with_theme(theme),
            );
        }
        right_meta = right_meta
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(theme.colors.success)
                    .child(format!("+{}", self.meta.added_lines)),
            )
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(theme.colors.danger)
                    .child(format!("-{}", self.meta.removed_lines)),
            );

        let file_banner = div()
            .w_full()
            .h(px(28.0))
            .px(px(theme.spacing.xl))
            .bg(theme.colors.bg_panel)
            .border_b_1()
            .border_color(theme.colors.border)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(theme.colors.text_secondary)
                    .child(SharedString::from(self.meta.file_path)),
            )
            .child(right_meta);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_app)
            .overflow_hidden()
            .child(alert_strip)
            .child(file_banner)
            .child(
                div().flex_1().overflow_hidden().child(
                    VirtualTextView::new("diff-preview-virtual-text", self.buffer)
                        .with_highlight_mode(LineHighlightMode::UnifiedDiff)
                        .with_context(labels, theme),
                ),
            )
            .into_any_element()
    }
}

fn render_exception_alert_strip(
    meta: &DiffViewMeta,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
) -> gpui::AnyElement {
    let (strip_tone, strip_text) = if !meta.is_sandbox_safe {
        (theme.colors.badge_danger, labels.applier_sandbox_blocked)
    } else if meta.has_conflict {
        (theme.colors.badge_danger, labels.applier_conflict_warning)
    } else if meta.is_truncated {
        (theme.colors.badge_warning, labels.applier_truncated_warning)
    } else {
        return div().into_any_element();
    };

    div()
        .w_full()
        .px(px(theme.spacing.xl))
        .py(px(theme.spacing.xs))
        .bg(strip_tone.bg)
        .border_b_1()
        .border_color(strip_tone.border)
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(strip_tone.fg)
                .child(strip_text),
        )
        .into_any_element()
}
