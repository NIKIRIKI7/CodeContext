//! Разделитель панелей (`SplitPane`), использующий `gpui::relative` и токены `ThemeTokens`.

use std::fmt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent,
    ParentElement as _, RenderOnce, Styled as _, Window, div, px, relative,
};

use crate::theme::{DEFAULT_LAYOUT, ThemeTokens, active_theme};

pub type SplitDividerDragStartCallback =
    Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

/// Горизонтальный двухпанельный контейнер с интерактивным перетаскиванием разделителя.
#[derive(IntoElement)]
pub struct SplitPane {
    left_ratio: f32,
    is_resizing: bool,
    theme: &'static ThemeTokens,
    left_child: AnyElement,
    right_child: AnyElement,
    on_start_resize: Option<SplitDividerDragStartCallback>,
}

impl fmt::Debug for SplitPane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitPane")
            .field("left_ratio", &self.left_ratio)
            .field("is_resizing", &self.is_resizing)
            .finish_non_exhaustive()
    }
}

impl SplitPane {
    /// Создаёт новый двухпанельный контейнер (`left_ratio` зажимается в безопасные границы макета).
    #[inline]
    pub fn new(
        left_ratio: f32,
        left_child: impl IntoElement,
        right_child: impl IntoElement,
    ) -> Self {
        Self {
            left_ratio: left_ratio
                .clamp(DEFAULT_LAYOUT.split_ratio_min, DEFAULT_LAYOUT.split_ratio_max),
            is_resizing: false,
            theme: active_theme(),
            left_child: left_child.into_any_element(),
            right_child: right_child.into_any_element(),
            on_start_resize: None,
        }
    }

    /// Задаёт тему оформления для разделителя.
    #[must_use]
    #[inline]
    pub const fn with_theme(mut self, theme: &'static ThemeTokens) -> Self {
        self.theme = theme;
        self
    }

    /// Указывает состояние активного перетаскивания разделителя.
    #[must_use]
    #[inline]
    pub const fn with_resizing(mut self, is_resizing: bool) -> Self {
        self.is_resizing = is_resizing;
        self
    }

    /// Регистрирует обработчик начала перетаскивания разделителя мышью.
    #[must_use]
    #[inline]
    pub fn on_start_resize(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_start_resize = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for SplitPane {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let left_pct = self.left_ratio;
        let right_pct = 1.0 - left_pct;
        let line_color =
            if self.is_resizing { self.theme.colors.cyan } else { self.theme.colors.border };
        let hover_color = self.theme.colors.cyan;

        let mut divider = div()
            .id("split-pane-resize-handle")
            .w(px(5.0))
            .h_full()
            .flex_shrink_0()
            .cursor_ew_resize()
            .bg(if self.is_resizing { self.theme.colors.cyan } else { self.theme.colors.bg_panel })
            .hover(move |s| s.bg(hover_color))
            .flex()
            .items_center()
            .justify_center()
            .child(div().w(px(self.theme.layout.border_thickness)).h_full().bg(line_color));

        if let Some(cb) = self.on_start_resize {
            divider = divider.on_mouse_down(MouseButton::Left, move |ev, win, cx| {
                cx.stop_propagation();
                cb(ev, win, cx);
            });
        }

        div()
            .size_full()
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(
                div()
                    .w(relative(left_pct))
                    .min_w(px(260.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.left_child),
            )
            .child(divider)
            .child(
                div()
                    .w(relative(right_pct))
                    .min_w(px(320.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.right_child),
            )
    }
}
