use gpui::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div, px,
};

use crate::theme::{FONT_FAMILY_MONO, ThemeTokens, active_theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, IntoElement)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
    show_dot: bool,
    theme: &'static ThemeTokens,
}

impl Badge {
    #[inline]
    pub fn new(label: impl Into<SharedString>, variant: BadgeVariant) -> Self {
        Self { label: label.into(), variant, show_dot: false, theme: active_theme() }
    }

    #[must_use]
    #[inline]
    pub const fn with_dot(mut self, show_dot: bool) -> Self {
        self.show_dot = show_dot;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_theme(mut self, theme: &'static ThemeTokens) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = &self.theme.colors;
        let tone = match self.variant {
            BadgeVariant::Neutral => colors.badge_neutral,
            BadgeVariant::Accent => colors.badge_accent,
            BadgeVariant::Success => colors.badge_success,
            BadgeVariant::Warning => colors.badge_warning,
            BadgeVariant::Danger => colors.badge_danger,
        };

        let _ = self.show_dot;
        div()
            .h(px(22.0))
            .px(px(self.theme.spacing.md))
            .rounded(px(self.theme.radius.md))
            .border_1()
            .border_color(tone.border)
            .bg(tone.bg)
            .font_family(FONT_FAMILY_MONO)
            .text_xs()
            .text_color(tone.fg)
            .whitespace_nowrap()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .child(self.label)
    }
}
