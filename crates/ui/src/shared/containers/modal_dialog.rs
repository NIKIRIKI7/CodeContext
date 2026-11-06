use std::fmt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, Rgba, SharedString, StatefulInteractiveElement as _,
    Styled as _, Window, div, px,
};

use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{FONT_FAMILY_UI, ThemeTokens, active_theme};

pub type ModalCloseCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

const MODAL_BACKDROP_COLOR: Rgba = Rgba { r: 0.02, g: 0.03, b: 0.05, a: 0.72 };

#[derive(IntoElement)]
pub struct ModalDialogShell {
    backdrop_id: ElementId,
    dialog_id: ElementId,
    close_btn_id: ElementId,
    title: SharedString,
    leading_icon: Option<UiIcon>,
    header_badge: Option<AnyElement>,
    width_px: f32,
    fixed_height_px: Option<f32>,
    max_height_px: Option<f32>,
    header_border: bool,
    theme: &'static ThemeTokens,
    on_close: Option<ModalCloseCallback>,
    children: Vec<AnyElement>,
}

impl fmt::Debug for ModalDialogShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModalDialogShell")
            .field("title", &self.title)
            .field("width_px", &self.width_px)
            .finish_non_exhaustive()
    }
}

impl ModalDialogShell {
    #[inline]
    pub fn new(
        backdrop_id: impl Into<ElementId>,
        dialog_id: impl Into<ElementId>,
        close_btn_id: impl Into<ElementId>,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            backdrop_id: backdrop_id.into(),
            dialog_id: dialog_id.into(),
            close_btn_id: close_btn_id.into(),
            title: title.into(),
            leading_icon: None,
            header_badge: None,
            width_px: 620.0,
            fixed_height_px: None,
            max_height_px: None,
            header_border: false,
            theme: active_theme(),
            on_close: None,
            children: Vec::new(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn width(mut self, width_px: f32) -> Self {
        self.width_px = width_px;
        self
    }

    #[must_use]
    #[inline]
    pub const fn fixed_height(mut self, height_px: f32) -> Self {
        self.fixed_height_px = Some(height_px);
        self
    }

    #[must_use]
    #[inline]
    pub const fn max_height(mut self, max_height_px: f32) -> Self {
        self.max_height_px = Some(max_height_px);
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
    pub fn header_badge(mut self, badge: impl IntoElement) -> Self {
        self.header_badge = Some(badge.into_any_element());
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_header_border(mut self, border: bool) -> Self {
        self.header_border = border;
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
    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_close_arc(mut self, handler: Option<ModalCloseCallback>) -> Self {
        self.on_close = handler;
        self
    }

    #[must_use]
    #[inline]
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for ModalDialogShell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let colors = &theme.colors;

        let mut close_btn = UiButton::icon_only(self.close_btn_id, UiIcon::X)
            .variant(ButtonVariant::Ghost)
            .with_theme(theme);
        if let Some(ref close_fn) = self.on_close {
            let cb = Arc::clone(close_fn);
            close_btn = close_btn.on_click(move |ev, w, c| cb(ev, w, c));
        }

        let mut title_row = div().flex().flex_row().items_center().gap(px(theme.spacing.sm));
        if let Some(icon) = self.leading_icon {
            title_row = title_row.child(render_icon(icon, 15.0, colors.cyan));
        }
        title_row = title_row.child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_sm()
                .text_color(colors.text_primary)
                .child(self.title),
        );
        if let Some(badge) = self.header_badge {
            title_row = title_row.child(badge);
        }

        let mut header = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(title_row)
            .child(close_btn);
        if self.header_border {
            header = header.pb(px(theme.spacing.sm)).border_b_1().border_color(colors.border);
        }

        let mut dialog = div()
            .id(self.dialog_id)
            .on_mouse_down(MouseButton::Left, |_ev, _win, cx| {
                cx.stop_propagation();
            })
            .on_click(|_ev, _win, cx| {
                cx.stop_propagation();
            })
            .w(px(self.width_px))
            .p(px(theme.spacing.xxl))
            .rounded(px(theme.radius.lg))
            .bg(colors.bg_panel)
            .border_1()
            .border_color(colors.border_active)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.md))
            .child(header)
            .children(self.children);

        if let Some(h) = self.fixed_height_px {
            dialog = dialog.h(px(h));
        }
        if let Some(max_h) = self.max_height_px {
            dialog = dialog.max_h(px(max_h)).overflow_y_scroll();
        }

        let mut backdrop = div()
            .id(self.backdrop_id)
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(MODAL_BACKDROP_COLOR)
            .flex()
            .items_center()
            .justify_center()
            .p(px(theme.spacing.xxl));

        if let Some(close_fn) = self.on_close {
            backdrop = backdrop.on_click(move |ev, w, c| close_fn(ev, w, c));
        }

        backdrop.child(dialog)
    }
}
