use std::fmt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _,
    Window, div, px,
};

use crate::locale::AppLocale;
use crate::shared::primitives::icon::{UiIcon, render_icon, render_vector_checkbox};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme};

pub type InputActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[inline]
pub fn format_cursor_text(text: &str, placeholder: &'static str, focused: bool) -> SharedString {
    let has_text = !text.is_empty();
    if focused && has_text {
        SharedString::from(format!("{text}▏"))
    } else if has_text {
        SharedString::from(text.to_owned())
    } else if focused {
        SharedString::from(format!("▏{placeholder}"))
    } else {
        SharedString::from(placeholder)
    }
}

#[derive(IntoElement)]
pub struct TextInputField {
    id: ElementId,
    text: String,
    placeholder: &'static str,
    focused: bool,
    is_invalid: bool,
    monospace: bool,
    fixed_width_px: Option<f32>,
    leading_icon: Option<UiIcon>,
    leading_prefix: Option<&'static str>,
    trailing_element: Option<AnyElement>,
    theme: &'static ThemeTokens,
    on_focus: Option<InputActionCallback>,
}

impl fmt::Debug for TextInputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInputField")
            .field("text", &self.text)
            .field("focused", &self.focused)
            .finish_non_exhaustive()
    }
}

impl TextInputField {
    #[inline]
    pub fn new(
        id: impl Into<ElementId>,
        text: String,
        placeholder: &'static str,
        focused: bool,
    ) -> Self {
        Self {
            id: id.into(),
            text,
            placeholder,
            focused,
            is_invalid: false,
            monospace: true,
            fixed_width_px: None,
            leading_icon: None,
            leading_prefix: None,
            trailing_element: None,
            theme: active_theme(),
            on_focus: None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn invalid(mut self, is_invalid: bool) -> Self {
        self.is_invalid = is_invalid;
        self
    }

    #[must_use]
    #[inline]
    pub const fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }

    #[must_use]
    #[inline]
    pub const fn fixed_width(mut self, width_px: f32) -> Self {
        self.fixed_width_px = Some(width_px);
        self
    }

    #[must_use]
    #[inline]
    pub const fn leading_icon(mut self, icon: UiIcon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    #[must_use]
    #[inline]
    pub const fn leading_prefix(mut self, prefix: &'static str) -> Self {
        self.leading_prefix = Some(prefix);
        self
    }

    #[must_use]
    #[inline]
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing_element = Some(el.into_any_element());
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
    pub fn on_focus_arc(mut self, handler: Option<InputActionCallback>) -> Self {
        self.on_focus = handler;
        self
    }
}

impl RenderOnce for TextInputField {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = &self.theme.colors;
        let has_text = !self.text.is_empty();
        let display_str = format_cursor_text(&self.text, self.placeholder, self.focused);

        let border_col = if self.is_invalid {
            colors.danger
        } else if self.focused {
            colors.cyan
        } else {
            colors.border
        };

        let mut box_el = div()
            .id(self.id)
            .on_mouse_down(MouseButton::Left, |_ev, _win, cx| {
                cx.stop_propagation();
            })
            .h(px(self.theme.layout.control_height))
            .px(px(self.theme.spacing.md))
            .rounded(px(self.theme.radius.md))
            .border_1()
            .border_color(border_col)
            .bg(colors.bg_app)
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(self.theme.spacing.sm));

        if let Some(w) = self.fixed_width_px {
            box_el = box_el.w(px(w)).flex_shrink_0();
        } else {
            box_el = box_el.flex_1().min_w(px(0.0));
        }

        if let Some(icon) = self.leading_icon {
            box_el = box_el.child(render_icon(
                icon,
                13.0,
                if self.focused { colors.cyan } else { colors.text_muted },
            ));
        }
        if let Some(prefix) = self.leading_prefix {
            box_el = box_el.child(
                div().font_family(FONT_FAMILY_MONO).text_xs().text_color(colors.cyan).child(prefix),
            );
        }

        box_el = box_el.child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(if self.monospace { FONT_FAMILY_MONO } else { FONT_FAMILY_UI })
                .text_xs()
                .text_color(if has_text || self.focused {
                    colors.text_primary
                } else {
                    colors.text_muted
                })
                .child(display_str),
        );

        if let Some(trailing) = self.trailing_element {
            box_el = box_el.child(trailing);
        }

        if let Some(cb) = self.on_focus {
            box_el = box_el.on_click(move |ev, w, c| {
                c.stop_propagation();
                cb(ev, w, c);
            });
        }

        box_el
    }
}

pub fn render_segmented_tab(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    active: bool,
    monospace: bool,
    flex_grow: bool,
    theme: &'static ThemeTokens,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let mut tab = div()
        .id(id.into())
        .px(px(theme.spacing.sm))
        .h_full()
        .rounded(px(theme.radius.sm))
        .bg(if active { colors.bg_elevated } else { colors.bg_app })
        .border_1()
        .border_color(if active { colors.border_active } else { colors.bg_app })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .items_center()
        .justify_center()
        .font_family(if monospace { FONT_FAMILY_MONO } else { FONT_FAMILY_UI })
        .text_xs()
        .text_color(if active { colors.text_primary } else { colors.text_secondary })
        .on_click(on_click)
        .child(label.into());

    if flex_grow {
        tab = tab.flex_1();
    }
    tab
}

pub fn render_checkbox_switch_row(
    id: impl Into<ElementId>,
    label: &'static str,
    is_on: bool,
    theme: &'static ThemeTokens,
) -> gpui::Stateful<gpui::Div> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let (check_background, check_border, check_foreground) = if is_on {
        (colors.badge_accent.bg, colors.cyan, colors.cyan)
    } else {
        (colors.bg_elevated, colors.border_active, colors.text_muted)
    };

    div()
        .id(id.into())
        .w_full()
        .h(px(theme.layout.control_height))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_panel)
        .border_1()
        .border_color(if is_on { colors.border_active } else { colors.border })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(if is_on { colors.text_primary } else { colors.text_secondary })
                .child(label),
        )
        .child(render_vector_checkbox(
            is_on,
            false,
            14.0,
            check_background,
            check_border,
            check_foreground,
            theme.radius.sm,
        ))
}

pub fn render_pane_header(
    left_content: impl IntoElement,
    right_content: impl IntoElement,
    padding_x: f32,
    theme: &'static ThemeTokens,
) -> gpui::Div {
    div()
        .w_full()
        .h(px(theme.layout.pane_header_height))
        .px(px(padding_x))
        .bg(theme.colors.bg_panel)
        .border_b_1()
        .border_color(theme.colors.border)
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.sm))
        .child(left_content)
        .child(right_content)
}

pub fn render_labeled_setting_row(
    label: &'static str,
    control: impl IntoElement,
    theme: &'static ThemeTokens,
) -> gpui::Div {
    div()
        .w_full()
        .min_h(px(28.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.lg))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(theme.colors.text_primary)
                .child(label),
        )
        .child(control)
}

pub fn render_compact_toggle_pill(
    id: &'static str,
    enabled: bool,
    locale: AppLocale,
    theme: &'static ThemeTokens,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let (check_background, check_border, check_foreground) = if enabled {
        (colors.badge_accent.bg, colors.cyan, colors.cyan)
    } else {
        (colors.bg_elevated, colors.border_active, colors.text_muted)
    };

    div()
        .id(id)
        .w(px(88.0))
        .h(px(theme.layout.control_height))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(if enabled { colors.border_active } else { colors.border })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex_shrink_0()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .on_click(on_click)
        .child(render_vector_checkbox(
            enabled,
            false,
            14.0,
            check_background,
            check_border,
            check_foreground,
            theme.radius.sm,
        ))
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if enabled { colors.text_primary } else { colors.text_secondary })
                .child(locale.toggle_state_label(enabled)),
        )
}
