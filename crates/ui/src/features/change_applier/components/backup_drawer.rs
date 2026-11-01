use std::fmt;
use std::sync::Arc;

use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::{AppLocale, UiLabels};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::icon::render_vector_checkbox;
use crate::theme::{FONT_FAMILY_UI, ThemeTokens, active_theme};

pub type ToggleBackupCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct BackupDrawerComponent {
    backups_enabled: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_toggle: Option<ToggleBackupCallback>,
}

impl fmt::Debug for BackupDrawerComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackupDrawerComponent")
            .field("backups_enabled", &self.backups_enabled)
            .finish_non_exhaustive()
    }
}

impl BackupDrawerComponent {
    #[inline]
    pub fn new(backups_enabled: bool) -> Self {
        Self {
            backups_enabled,
            labels: AppLocale::Ru.labels(),
            theme: active_theme(),
            on_toggle: None,
        }
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
    #[inline]
    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for BackupDrawerComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let colors = &theme.colors;
        let labels = self.labels;
        let hover_bg = colors.bg_hover;

        let (check_background, check_border, check_foreground) = if self.backups_enabled {
            (colors.badge_accent.bg, colors.cyan, colors.cyan)
        } else {
            (colors.bg_elevated, colors.border_active, colors.text_muted)
        };

        let mut backup_row = div()
            .id("toggle-backup-drawer-pill")
            .w_full()
            .h(px(theme.layout.control_height))
            .px(px(theme.spacing.md))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_panel)
            .border_1()
            .border_color(if self.backups_enabled { colors.border_active } else { colors.border })
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
                    .text_color(colors.text_primary)
                    .child(labels.settings_create_backups),
            )
            .child(render_vector_checkbox(
                self.backups_enabled,
                false,
                14.0,
                check_background,
                check_border,
                check_foreground,
                theme.radius.sm,
            ));

        if let Some(cb) = self.on_toggle {
            backup_row = backup_row.on_click(move |ev, w, c| cb(ev, w, c));
        }

        div()
            .w_full()
            .px(px(theme.spacing.lg))
            .py(px(theme.spacing.sm))
            .bg(colors.bg_elevated)
            .border_b_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xs))
            .child(backup_row)
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
                            .text_color(colors.text_muted)
                            .child(labels.status_sandbox_label),
                    )
                    .child(
                        Badge::new(labels.status_sandbox_on, BadgeVariant::Neutral)
                            .with_theme(theme),
                    ),
            )
    }
}
