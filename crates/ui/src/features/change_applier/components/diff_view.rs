use std::fmt;
use std::sync::Arc;

use core::shared_kernel::fs::line_ending::LineEnding;
use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::locale::{AppLocale, UiLabels};
use crate::shared::containers::virtual_text::{
    LineHighlightMode, VirtualTextBuffer, VirtualTextView,
};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme};

pub type DiffActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

pub type SelectDiffLineCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct DiffViewMeta {
    pub file_path: String,
    pub editing_line_index: Option<usize>,
    pub editing_line_draft: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub line_ending: LineEnding,
    pub is_truncated: bool,
    pub is_sandbox_safe: bool,
    pub has_conflict: bool,
    pub is_applied: bool,
    pub is_raw_code_mode: bool,
}

#[derive(IntoElement)]
pub struct DiffViewComponent {
    meta: DiffViewMeta,
    buffer: VirtualTextBuffer,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_apply_single: Option<DiffActionCallback>,
    on_toggle_raw_mode: Option<DiffActionCallback>,
    on_force_apply: Option<DiffActionCallback>,
    on_select_line: Option<SelectDiffLineCallback>,
    on_commit_line_edit: Option<DiffActionCallback>,
    on_cancel_line_edit: Option<DiffActionCallback>,
}

impl fmt::Debug for DiffViewComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiffViewComponent").field("meta", &self.meta).finish_non_exhaustive()
    }
}

impl DiffViewComponent {
    #[inline]
    pub fn new(meta: DiffViewMeta, buffer: VirtualTextBuffer) -> Self {
        Self {
            meta,
            buffer,
            labels: AppLocale::Ru.labels(),
            theme: active_theme(),
            on_apply_single: None,
            on_toggle_raw_mode: None,
            on_force_apply: None,
            on_select_line: None,
            on_commit_line_edit: None,
            on_cancel_line_edit: None,
        }
    }

    #[must_use]
    pub fn on_line_editing(
        mut self,
        on_select_line: impl Fn(usize, &mut Window, &mut App) + 'static,
        on_commit: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_cancel: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_line = Some(Arc::new(on_select_line));
        self.on_commit_line_edit = Some(Arc::new(on_commit));
        self.on_cancel_line_edit = Some(Arc::new(on_cancel));
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
    pub fn with_file_actions(
        mut self,
        on_apply_single: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_toggle_raw_mode: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_force_apply: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_apply_single = Some(Arc::new(on_apply_single));
        self.on_toggle_raw_mode = Some(Arc::new(on_toggle_raw_mode));
        self.on_force_apply = Some(Arc::new(on_force_apply));
        self
    }
}

impl RenderOnce for DiffViewComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let labels = self.labels;

        if self.buffer.is_empty() {
            return render_empty_diff_card(labels, theme).into_any_element();
        }

        let alert_strip =
            render_exception_alert_strip(&self.meta, labels, theme, self.on_force_apply.as_ref());
        let right_meta = render_banner_right_controls(
            &self.meta,
            labels,
            theme,
            self.on_toggle_raw_mode,
            self.on_apply_single,
        );

        let file_banner = div()
            .w_full()
            .h(px(30.0))
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

        let _ = (&self.on_commit_line_edit, &self.on_cancel_line_edit);
        let draft_opt = self.meta.editing_line_index.map(|_| self.meta.editing_line_draft);

        let highlight_mode = if self.meta.is_raw_code_mode {
            LineHighlightMode::PackedContext
        } else {
            LineHighlightMode::UnifiedDiff
        };

        let mut text_view = VirtualTextView::new("diff-preview-virtual-text", self.buffer)
            .with_selected_line(self.meta.editing_line_index)
            .with_editing_line_draft(draft_opt)
            .with_highlight_mode(highlight_mode)
            .with_context(labels, theme);
        if let Some(line_cb) = self.on_select_line {
            text_view = text_view.on_line_click(move |idx, w, c| line_cb(idx, w, c));
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_app)
            .overflow_hidden()
            .child(alert_strip)
            .child(file_banner)
            .child(div().flex_1().overflow_hidden().child(text_view))
            .into_any_element()
    }
}

fn render_empty_diff_card(labels: &'static UiLabels, theme: &'static ThemeTokens) -> gpui::Div {
    div()
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
}

fn render_banner_right_controls(
    meta: &DiffViewMeta,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_toggle_raw_mode: Option<DiffActionCallback>,
    on_apply_single: Option<DiffActionCallback>,
) -> gpui::Div {
    let mut right_meta = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));
    if let Some(toggle_mode_fn) = on_toggle_raw_mode {
        let mode_label = if meta.is_raw_code_mode { "Diff" } else { "Code" };
        right_meta = right_meta.child(
            UiButton::new("diff-toggle-raw-code-btn", mode_label)
                .icon(UiIcon::FileCode)
                .variant(ButtonVariant::Ghost)
                .active(meta.is_raw_code_mode)
                .with_theme(theme)
                .on_click(move |ev, w, c| toggle_mode_fn(ev, w, c)),
        );
    }

    if meta.is_applied {
        return right_meta.child(
            Badge::new(labels.applier_valid_block, BadgeVariant::Success).with_theme(theme),
        );
    }

    if meta.line_ending == LineEnding::Crlf {
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
                .child(format!("+{}", meta.added_lines)),
        )
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(theme.colors.danger)
                .child(format!("-{}", meta.removed_lines)),
        );
    if meta.is_sandbox_safe
        && !meta.has_conflict
        && let Some(apply_single_fn) = on_apply_single
    {
        right_meta = right_meta.child(
            UiButton::new("diff-apply-single-file-btn", labels.btn_apply_changes)
                .icon(UiIcon::Check)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme)
                .on_click(move |ev, w, c| apply_single_fn(ev, w, c)),
        );
    }
    right_meta
}

fn render_exception_alert_strip(
    meta: &DiffViewMeta,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_force_apply: Option<&DiffActionCallback>,
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

    let mut row = div()
        .w_full()
        .px(px(theme.spacing.xl))
        .py(px(theme.spacing.xs))
        .bg(strip_tone.bg)
        .border_b_1()
        .border_color(strip_tone.border)
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.sm))
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
        );

    if meta.has_conflict
        && meta.is_sandbox_safe
        && let Some(force_cb) = on_force_apply
    {
        let cb = Arc::clone(force_cb);
        row = row.child(
            UiButton::new("diff-force-resolve-conflict-btn", labels.btn_autofix)
                .variant(ButtonVariant::Danger)
                .with_theme(theme)
                .on_click(move |ev, w, c| cb(ev, w, c)),
        );
    }

    row.into_any_element()
}
