use std::fmt;
use std::sync::Arc;

use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, StatefulInteractiveElement as _,
    Styled as _, Window, div, px,
};

use crate::locale::{AppLocale, UiLabels};
use crate::shared::containers::form_controls::render_checkbox_switch_row;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
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

        let mut backup_row = render_checkbox_switch_row(
            "toggle-backup-drawer-pill",
            labels.settings_create_backups,
            self.backups_enabled,
            theme,
        );
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
