use std::fmt;
use std::sync::Arc;

use core::preset::models::template::SystemPromptMode;
use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    UniformListScrollHandle, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::containers::form_controls::render_pane_header;
use crate::shared::containers::virtual_text::{VirtualTextBuffer, VirtualTextView};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_UI, ThemeTokens, format_bytes_human, format_tokens_estimate};

pub type SelectChunkCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type PreviewActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

pub type SelectPreviewLineCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct LivePreviewProps {
    pub buffer: VirtualTextBuffer,
    pub editing_line_index: Option<usize>,
    pub editing_line_draft: String,
    pub total_bytes: u64,
    pub packed_files_count: usize,
    pub blocked_secrets_count: usize,
    pub skipped_binary_count: usize,
    pub active_chunk_index: usize,
    pub total_chunks: usize,
    pub system_prompt_mode: SystemPromptMode,
    pub copied_recently: bool,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

#[derive(IntoElement)]
pub struct LivePreviewComponent {
    props: LivePreviewProps,
    scroll_handle: Option<UniformListScrollHandle>,
    on_select_chunk: Option<SelectChunkCallback>,
    on_select_line: Option<SelectPreviewLineCallback>,
    on_commit_line_edit: Option<PreviewActionCallback>,
    on_cancel_line_edit: Option<PreviewActionCallback>,
    on_open_prompt_modal: Option<PreviewActionCallback>,
    on_open_meta: Option<PreviewActionCallback>,
    on_copy: Option<PreviewActionCallback>,
}

impl fmt::Debug for LivePreviewComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LivePreviewComponent").field("props", &self.props).finish_non_exhaustive()
    }
}

impl LivePreviewComponent {
    #[inline]
    pub const fn new(props: LivePreviewProps) -> Self {
        Self {
            props,
            scroll_handle: None,
            on_select_chunk: None,
            on_select_line: None,
            on_commit_line_edit: None,
            on_cancel_line_edit: None,
            on_open_prompt_modal: None,
            on_open_meta: None,
            on_copy: None,
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
    pub fn track_scroll(mut self, handle: UniformListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    #[must_use]
    #[inline]
    pub fn on_select_chunk(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_chunk = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_open_prompt_modal(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_prompt_modal = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_open_meta(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_meta = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_copy(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_copy = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for LivePreviewComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.props.theme;
        let labels = self.props.labels;

        let compact_summary = SharedString::from(format!(
            "{} · {}",
            format_tokens_estimate(self.props.total_bytes, labels.unit_tokens),
            format_bytes_human(self.props.total_bytes)
        ));

        let mut meta_btn = UiButton::icon_only("btn-preview-open-meta", UiIcon::Info)
            .variant(ButtonVariant::Ghost)
            .with_theme(theme);
        if let Some(meta_fn) = self.on_open_meta {
            meta_btn = meta_btn.on_click(move |ev, w, c| meta_fn(ev, w, c));
        }

        let mut left_info = div()
            .flex_1()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xs))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(theme.colors.text_primary)
                    .child(labels.preview_header),
            )
            .child(Badge::new(compact_summary, BadgeVariant::Neutral).with_theme(theme))
            .child(meta_btn);

        if self.props.blocked_secrets_count > 0 {
            left_info = left_info.child(
                Badge::new(
                    format!("{} {}", self.props.blocked_secrets_count, labels.unit_secrets_hidden),
                    BadgeVariant::Danger,
                )
                .with_theme(theme),
            );
        }

        let copy_label: SharedString = if self.props.copied_recently {
            SharedString::from(labels.btn_copied)
        } else if self.props.total_chunks > 1 {
            SharedString::from(format!(
                "{} ({}/{})",
                labels.btn_copy_context,
                self.props.active_chunk_index + 1,
                self.props.total_chunks
            ))
        } else {
            SharedString::from(labels.btn_copy_context)
        };
        let copy_icon = if self.props.copied_recently { UiIcon::Check } else { UiIcon::Copy };
        let _ = (self.props.system_prompt_mode, &self.on_open_prompt_modal);

        let mut copy_btn = UiButton::new("btn-copy-context-main", copy_label)
            .icon(copy_icon)
            .variant(if self.props.copied_recently {
                ButtonVariant::SuccessCta
            } else {
                ButtonVariant::CyanCta
            })
            .with_theme(theme);

        if let Some(cb) = self.on_copy {
            copy_btn = copy_btn.on_click(move |ev, w, c| cb(ev, w, c));
        }

        let chunk_bar = render_chunk_selector_bar(
            self.props.active_chunk_index,
            self.props.total_chunks,
            labels,
            theme,
            self.on_select_chunk.as_ref(),
        );

        let edit_bar_opt = render_inline_line_editor_bar(
            self.props.editing_line_index,
            &self.props.editing_line_draft,
            labels,
            theme,
            self.on_commit_line_edit.as_ref(),
            self.on_cancel_line_edit.as_ref(),
        );

        let mut text_view = VirtualTextView::new("live-preview-virtual-text", self.props.buffer)
            .with_selected_line(self.props.editing_line_index)
            .with_context(labels, theme);
        if let Some(line_cb) = self.on_select_line {
            text_view = text_view.on_line_click(move |line_idx, w, c| line_cb(line_idx, w, c));
        }
        if let Some(handle) = self.scroll_handle {
            text_view = text_view.track_scroll(handle);
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_app)
            .overflow_hidden()
            .child(render_pane_header(left_info, copy_btn, theme.spacing.xl, theme))
            .child(chunk_bar)
            .children(edit_bar_opt)
            .child(div().flex_1().overflow_hidden().child(text_view))
    }
}

pub(crate) fn render_inline_line_editor_bar(
    editing_line_idx: Option<usize>,
    draft: &str,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_commit: Option<&PreviewActionCallback>,
    on_cancel: Option<&PreviewActionCallback>,
) -> Option<gpui::AnyElement> {
    let line_idx = editing_line_idx?;
    let colors = &theme.colors;
    let line_prefix: &'static str = Box::leak(format!("L{}:", line_idx + 1).into_boxed_str());

    let input = crate::shared::containers::form_controls::TextInputField::new(
        "inline-virtual-line-editor-input",
        draft.to_owned(),
        "",
        true,
    )
    .leading_prefix(line_prefix)
    .with_theme(theme);

    let mut save_btn = UiButton::new("inline-line-edit-save-btn", labels.btn_save_global)
        .icon(UiIcon::Check)
        .variant(ButtonVariant::Primary)
        .with_theme(theme);
    if let Some(cb) = on_commit {
        let cb_clone = Arc::clone(cb);
        save_btn = save_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let mut cancel_btn = UiButton::icon_only("inline-line-edit-cancel-btn", UiIcon::X)
        .variant(ButtonVariant::Ghost)
        .with_theme(theme);
    if let Some(cb) = on_cancel {
        let cb_clone = Arc::clone(cb);
        cancel_btn = cancel_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    Some(
        div()
            .w_full()
            .h(px(34.0))
            .px(px(theme.spacing.xl))
            .bg(colors.bg_elevated)
            .border_b_1()
            .border_color(colors.border_active)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xs))
            .child(input)
            .child(save_btn)
            .child(cancel_btn)
            .into_any_element(),
    )
}

fn render_chunk_selector_bar(
    active_idx: usize,
    total_chunks: usize,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_select: Option<&SelectChunkCallback>,
) -> gpui::AnyElement {
    if total_chunks <= 1 {
        return div().into_any_element();
    }

    let mut row = div()
        .w_full()
        .px(px(theme.spacing.xl))
        .py(px(theme.spacing.xs))
        .bg(theme.colors.bg_panel)
        .border_b_1()
        .border_color(theme.colors.border)
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(
            div()
                .text_xs()
                .text_color(theme.colors.text_secondary)
                .child(format!("{} ({total_chunks}):", labels.chunk_selector_prefix)),
        );

    for idx in 0..total_chunks {
        let mut btn = UiButton::new(
            ("preview-chunk-btn", idx),
            format!("{} {}", labels.chunk_volume_prefix, idx + 1),
        )
        .variant(ButtonVariant::FilterPill)
        .with_theme(theme)
        .active(idx == active_idx);

        if let Some(cb) = on_select {
            let cb_clone = Arc::clone(cb);
            btn = btn.on_click(move |_ev, win, cx| cb_clone(idx, win, cx));
        }
        row = row.child(btn);
    }

    row.into_any_element()
}
