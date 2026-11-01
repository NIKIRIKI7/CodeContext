use std::fmt;
use std::sync::Arc;

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{FONT_FAMILY_UI, ThemeTokens, active_theme};

pub type ButtonClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    Primary,
    CyanCta,
    SuccessCta,
    FilterPill,
    #[default]
    Secondary,
    Ghost,
    Danger,
}

#[derive(IntoElement)]
pub struct UiButton {
    id: ElementId,
    label: SharedString,
    icon: Option<UiIcon>,
    shortcut: Option<SharedString>,
    variant: ButtonVariant,
    active: bool,
    disabled: bool,
    min_width: Option<f32>,
    fixed_width: Option<f32>,
    theme: &'static ThemeTokens,
    on_click: Option<ButtonClickHandler>,
}

impl fmt::Debug for UiButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiButton")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("variant", &self.variant)
            .field("active", &self.active)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl UiButton {
    #[inline]
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            variant: ButtonVariant::Secondary,
            active: false,
            disabled: false,
            min_width: None,
            fixed_width: None,
            theme: active_theme(),
            on_click: None,
        }
    }

    #[inline]
    pub fn icon_only(id: impl Into<ElementId>, icon: UiIcon) -> Self {
        Self::new(id, "").icon(icon)
    }

    #[must_use]
    #[inline]
    pub const fn icon(mut self, icon: UiIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    #[inline]
    pub const fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    #[must_use]
    #[inline]
    pub const fn fixed_width(mut self, width: f32) -> Self {
        self.fixed_width = Some(width);
        self
    }

    #[must_use]
    #[inline]
    pub const fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    #[inline]
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    #[must_use]
    #[inline]
    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    #[must_use]
    #[inline]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_theme(mut self, theme: &'static ThemeTokens) -> Self {
        self.theme = theme;
        self
    }

    #[must_use]
    #[inline]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for UiButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = &self.theme.colors;
        let (bg, hover_bg, border, text_col) = if self.disabled {
            (colors.bg_elevated, colors.bg_elevated, colors.border, colors.text_muted)
        } else {
            match (self.variant, self.active) {
                (ButtonVariant::CyanCta, _) => {
                    (colors.cyan, colors.cyan_hover, colors.cyan, colors.text_on_cyan)
                }
                (ButtonVariant::SuccessCta, _) => (
                    colors.button_success.bg,
                    colors.success,
                    colors.button_success.border,
                    colors.button_success.fg,
                ),
                (ButtonVariant::Primary, _) | (ButtonVariant::Secondary, true) => {
                    (colors.accent, colors.accent_hover, colors.accent_hover, colors.text_on_accent)
                }
                (ButtonVariant::FilterPill | ButtonVariant::Ghost, true) => {
                    (colors.bg_elevated, colors.bg_hover, colors.border_active, colors.text_primary)
                }
                (ButtonVariant::FilterPill | ButtonVariant::Secondary, false) => {
                    (colors.bg_elevated, colors.bg_hover, colors.border, colors.text_primary)
                }
                (ButtonVariant::Ghost, false) => {
                    (colors.bg_panel, colors.bg_hover, colors.bg_panel, colors.text_secondary)
                }
                (ButtonVariant::Danger, _) => (
                    colors.button_danger.bg,
                    colors.danger,
                    colors.button_danger.border,
                    colors.button_danger.fg,
                ),
            }
        };

        let btn_height = if self.variant == ButtonVariant::Ghost {
            self.theme.layout.control_height_sm
        } else {
            self.theme.layout.control_height
        };
        let is_icon_only = self.icon.is_some() && self.label.is_empty();
        let pad_x = if is_icon_only {
            0.0
        } else if self.variant == ButtonVariant::Ghost {
            self.theme.spacing.sm
        } else {
            self.theme.spacing.md
        };

        let _ = self.shortcut;

        let mut el = div()
            .id(self.id)
            .h(px(btn_height))
            .px(px(pad_x))
            .rounded(px(self.theme.radius.md))
            .border_1()
            .border_color(border)
            .bg(bg)
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(text_col)
            .whitespace_nowrap()
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(self.theme.spacing.xs));

        if let Some(icon) = self.icon {
            el = el.child(render_icon(icon, 14.0, text_col));
        }
        if !self.label.is_empty() {
            el = el.child(self.label);
        }

        if let Some(min_w) = self.min_width {
            el = el.min_w(px(min_w));
        }
        if let Some(fixed_w) = self.fixed_width {
            el = el.w(px(fixed_w));
        } else if is_icon_only {
            el = el.w(px(btn_height));
        }

        if !self.disabled {
            el = el.cursor_pointer().hover(move |style| style.bg(hover_bg));
            if let Some(handler) = self.on_click {
                el = el.on_click(move |ev, win, cx| handler(ev, win, cx));
            }
        }

        el
    }
}
