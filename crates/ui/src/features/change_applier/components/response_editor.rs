use std::fmt;
use std::sync::Arc;

use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::locale::{RU_LABELS, UiLabels};
use crate::shared::containers::virtual_text::{
    LineHighlightMode, VirtualTextBuffer, VirtualTextView,
};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme};

pub type ResponseActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct ResponseEditorComponent {
    buffer: VirtualTextBuffer,
    parsed_blocks_count: usize,
    preamble_stripped: bool,
    closed_truncated_block: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_paste_clipboard: Option<ResponseActionCallback>,
    on_autofix: Option<ResponseActionCallback>,
}

impl fmt::Debug for ResponseEditorComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResponseEditorComponent")
            .field("parsed_blocks_count", &self.parsed_blocks_count)
            .field("preamble_stripped", &self.preamble_stripped)
            .field("closed_truncated_block", &self.closed_truncated_block)
            .finish_non_exhaustive()
    }
}

impl ResponseEditorComponent {
    #[inline]
    pub const fn new(
        buffer: VirtualTextBuffer,
        preamble_stripped: bool,
        closed_truncated_block: bool,
    ) -> Self {
        Self {
            buffer,
            parsed_blocks_count: 0,
            preamble_stripped,
            closed_truncated_block,
            labels: &RU_LABELS,
            theme: active_theme(),
            on_paste_clipboard: None,
            on_autofix: None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_parsed_count(mut self, count: usize) -> Self {
        self.parsed_blocks_count = count;
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
    pub fn with_actions(
        mut self,
        on_paste_clipboard: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_autofix: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_paste_clipboard = Some(Arc::new(on_paste_clipboard));
        self.on_autofix = Some(Arc::new(on_autofix));
        self
    }
}

impl RenderOnce for ResponseEditorComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let labels = self.labels;
        let chars_len = self.buffer.total_bytes();

        let mut left_header = div()
            .flex_1()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.sm))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(theme.colors.text_primary)
                    .child(labels.applier_input_header),
            );

        if chars_len > 0 {
            left_header = left_header.child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(theme.colors.text_muted)
                    .child(SharedString::from(format!(
                        "{} {}",
                        self.parsed_blocks_count, labels.unit_blocks
                    ))),
            );
        }

        let _ = (self.closed_truncated_block, self.preamble_stripped, &self.on_autofix);

        let mut actions =
            div().flex_shrink_0().flex().flex_row().items_center().gap(px(theme.spacing.xs));

        if let Some(cb) = self.on_paste_clipboard {
            actions = actions.child(
                UiButton::new("btn-read-clipboard-response", labels.btn_paste_clipboard)
                    .icon(UiIcon::ClipboardPaste)
                    .variant(ButtonVariant::Secondary)
                    .with_theme(theme)
                    .on_click(move |ev, w, c| cb(ev, w, c)),
            );
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
                    .gap(px(theme.spacing.sm))
                    .child(left_header)
                    .child(actions),
            )
            .child(
                div().flex_1().overflow_hidden().child(
                    VirtualTextView::new("applier-raw-response-view", self.buffer)
                        .with_highlight_mode(LineHighlightMode::RawAiStream)
                        .with_context(labels, theme),
                ),
            )
    }
}
