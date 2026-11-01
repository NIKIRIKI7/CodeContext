use std::fmt;
use std::sync::Arc;

use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    UniformListScrollHandle, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::containers::virtual_text::{VirtualTextBuffer, VirtualTextView};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_UI, ThemeTokens, format_bytes_human, format_tokens_estimate};

pub type SelectChunkCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type PreviewActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct LivePreviewProps {
    pub buffer: VirtualTextBuffer,
    pub total_bytes: u64,
    pub packed_files_count: usize,
    pub blocked_secrets_count: usize,
    pub skipped_binary_count: usize,
    pub active_chunk_index: usize,
    pub total_chunks: usize,
    pub copied_recently: bool,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

#[derive(IntoElement)]
pub struct LivePreviewComponent {
    props: LivePreviewProps,
    scroll_handle: Option<UniformListScrollHandle>,
    on_select_chunk: Option<SelectChunkCallback>,
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
            on_open_meta: None,
            on_copy: None,
        }
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

        let mut text_view = VirtualTextView::new("live-preview-virtual-text", self.props.buffer)
            .with_context(labels, theme);
        if let Some(handle) = self.scroll_handle {
            text_view = text_view.track_scroll(handle);
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_app)
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .h(px(theme.layout.pane_header_height))
                    .px(px(theme.spacing.xl))
                    .bg(theme.colors.bg_panel)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.md))
                    .child(left_info)
                    .child(copy_btn),
            )
            .child(chunk_bar)
            .child(div().flex_1().overflow_hidden().child(text_view))
    }
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
