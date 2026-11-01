use std::fmt;
use std::sync::Arc;

use core::system::ports::updater::{SemanticVersion, UpdateChannel};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Rgba,
    SharedString, Styled as _, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens};

pub type UpdateModalActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone, Copy)]
pub struct UpdatePromptProps {
    pub current_version: SemanticVersion,
    pub new_version: SemanticVersion,
    pub channel: UpdateChannel,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

#[derive(IntoElement)]
pub struct UpdatePromptModalComponent {
    props: UpdatePromptProps,
    on_confirm_update: Option<UpdateModalActionCallback>,
    on_dismiss: Option<UpdateModalActionCallback>,
}

impl fmt::Debug for UpdatePromptModalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdatePromptModalComponent")
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}

impl UpdatePromptModalComponent {
    #[inline]
    pub const fn new(props: UpdatePromptProps) -> Self {
        Self { props, on_confirm_update: None, on_dismiss: None }
    }

    #[must_use]
    pub fn with_actions(
        mut self,
        on_confirm_update: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_dismiss: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_confirm_update = Some(Arc::new(on_confirm_update));
        self.on_dismiss = Some(Arc::new(on_dismiss));
        self
    }

    fn render_details_box(&self, theme: &'static ThemeTokens) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = self.props.labels;
        let channel_name = match self.props.channel {
            UpdateChannel::Stable => labels.settings_channel_stable,
            UpdateChannel::Beta => labels.settings_channel_beta,
            UpdateChannel::Alpha => labels.settings_channel_alpha,
        };
        let channel_variant = match self.props.channel {
            UpdateChannel::Stable => BadgeVariant::Success,
            UpdateChannel::Beta => BadgeVariant::Accent,
            UpdateChannel::Alpha => BadgeVariant::Warning,
        };

        div()
            .w_full()
            .p(px(theme.spacing.lg))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.sm))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(labels.update_modal_channel_label),
                    )
                    .child(Badge::new(channel_name, channel_variant).with_theme(theme)),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(labels.update_modal_current_label),
                    )
                    .child(
                        div()
                            .font_family(FONT_FAMILY_MONO)
                            .text_xs()
                            .text_color(colors.text_muted)
                            .child(SharedString::from(format!("v{}", self.props.current_version))),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_primary)
                            .child(labels.update_modal_new_label),
                    )
                    .child(
                        Badge::new(format!("v{}", self.props.new_version), BadgeVariant::Accent)
                            .with_theme(theme),
                    ),
            )
    }
}

impl RenderOnce for UpdatePromptModalComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.props.theme;
        let colors = &theme.colors;
        let labels = self.props.labels;
        let details_box = self.render_details_box(theme);

        let mut close_btn = UiButton::icon_only("update-modal-close-icon", UiIcon::X)
            .variant(ButtonVariant::Ghost)
            .with_theme(theme);
        let mut later_btn = UiButton::new("update-modal-later-btn", labels.btn_update_later)
            .variant(ButtonVariant::Secondary)
            .with_theme(theme);
        if let Some(ref dismiss_fn) = self.on_dismiss {
            let dismiss_icon = Arc::clone(dismiss_fn);
            let dismiss_btn = Arc::clone(dismiss_fn);
            close_btn = close_btn.on_click(move |ev, w, c| dismiss_icon(ev, w, c));
            later_btn = later_btn.on_click(move |ev, w, c| dismiss_btn(ev, w, c));
        }

        let mut update_btn = UiButton::new("update-modal-confirm-btn", labels.btn_update_now)
            .icon(UiIcon::Check)
            .variant(ButtonVariant::Primary)
            .with_theme(theme);
        if let Some(confirm_fn) = self.on_confirm_update {
            update_btn = update_btn.on_click(move |ev, w, c| confirm_fn(ev, w, c));
        }

        let backdrop = Rgba { r: 0.02, g: 0.03, b: 0.05, a: 0.72 };

        div()
            .id("update-prompt-modal-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(backdrop)
            .flex()
            .items_center()
            .justify_center()
            .p(px(theme.spacing.xxl))
            .child(
                div()
                    .w(px(440.0))
                    .p(px(theme.spacing.xxl))
                    .rounded(px(theme.radius.lg))
                    .bg(colors.bg_panel)
                    .border_1()
                    .border_color(colors.border_active)
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.lg))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .font_family(FONT_FAMILY_UI)
                                    .text_sm()
                                    .text_color(colors.text_primary)
                                    .child(labels.update_modal_title),
                            )
                            .child(close_btn),
                    )
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(labels.update_modal_subtitle),
                    )
                    .child(details_box)
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .gap(px(theme.spacing.sm))
                            .child(later_btn)
                            .child(update_btn),
                    ),
            )
    }
}
