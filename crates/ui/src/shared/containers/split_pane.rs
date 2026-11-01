//! Разделитель панелей (`SplitPane`), использующий `gpui::relative` и токены `ThemeTokens`.

use std::fmt;

use gpui::{
    AnyElement, App, IntoElement, ParentElement as _, RenderOnce, Styled as _, Window, div, px,
    relative,
};

use crate::theme::{DEFAULT_LAYOUT, ThemeTokens, active_theme};

/// Горизонтальный двухпанельный контейнер с фиксируемой пропорцией ширины.
#[derive(IntoElement)]
pub struct SplitPane {
    left_ratio: f32,
    theme: &'static ThemeTokens,
    left_child: AnyElement,
    right_child: AnyElement,
}

impl fmt::Debug for SplitPane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitPane").field("left_ratio", &self.left_ratio).finish_non_exhaustive()
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
            theme: active_theme(),
            left_child: left_child.into_any_element(),
            right_child: right_child.into_any_element(),
        }
    }

    /// Задаёт тему оформления для разделителя.
    #[must_use]
    #[inline]
    pub const fn with_theme(mut self, theme: &'static ThemeTokens) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for SplitPane {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let left_pct = self.left_ratio;
        let right_pct = 1.0 - left_pct;

        div()
            .size_full()
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(
                div()
                    .w(relative(left_pct))
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.left_child),
            )
            .child(
                div()
                    .w(px(self.theme.layout.border_thickness))
                    .h_full()
                    .bg(self.theme.colors.border),
            )
            .child(
                div()
                    .w(relative(right_pct))
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.right_child),
            )
    }
}
