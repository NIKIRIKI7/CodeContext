use std::fmt;
use std::sync::Arc;

use di::BackupSnapshotInfo;
use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::{AppLocale, UiLabels};
use crate::shared::containers::form_controls::render_checkbox_switch_row;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{
    FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme, format_bytes_human,
};

pub type ToggleBackupCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type RestoreSnapshotCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct BackupDrawerComponent {
    backups_enabled: bool,
    snapshots: Vec<BackupSnapshotInfo>,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_toggle: Option<ToggleBackupCallback>,
    on_restore_snapshot: Option<RestoreSnapshotCallback>,
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
            snapshots: Vec::new(),
            labels: AppLocale::Ru.labels(),
            theme: active_theme(),
            on_toggle: None,
            on_restore_snapshot: None,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_snapshots(mut self, snapshots: Vec<BackupSnapshotInfo>) -> Self {
        self.snapshots = snapshots;
        self
    }

    #[must_use]
    #[inline]
    pub fn on_restore_snapshot(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_restore_snapshot = Some(Arc::new(handler));
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

        let mut container = div()
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
            );

        for (idx, snap) in self.snapshots.iter().take(4).enumerate() {
            let secs = (snap.timestamp_ms / 1_000) % 86_400;
            let hh = (secs / 3_600) % 24;
            let mm = (secs % 3_600) / 60;
            let ss = secs % 60;
            let info_str = SharedString::from(format!(
                "{hh:02}:{mm:02}:{ss:02} · {} {} · {}",
                snap.files_count,
                labels.unit_files,
                format_bytes_human(snap.size_bytes)
            ));
            let mut undo_btn = UiButton::new(("restore-snap-btn", idx), labels.btn_undo)
                .icon(UiIcon::RotateCcw)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme);
            if let Some(ref restore_fn) = self.on_restore_snapshot {
                let cb = Arc::clone(restore_fn);
                let id_owned = snap.snapshot_id.clone();
                undo_btn = undo_btn.on_click(move |_ev, win, cx| cb(&id_owned, win, cx));
            }
            container = container.child(
                div()
                    .w_full()
                    .h(px(24.0))
                    .px(px(theme.spacing.sm))
                    .rounded(px(theme.radius.sm))
                    .bg(colors.bg_panel)
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .font_family(FONT_FAMILY_MONO)
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(info_str),
                    )
                    .child(undo_btn),
            );
        }

        container
    }
}
