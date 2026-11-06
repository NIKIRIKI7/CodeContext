use std::fmt;
use std::sync::Arc;

use di::{BackupSnapshotInfo, ParsedResponseFile};
use gpui::{
    App, ClickEvent, Div, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::features::change_applier::components::backup_drawer::{
    BackupDrawerComponent, RestoreSnapshotCallback,
};
use crate::locale::{AppLocale, UiLabels};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon, render_vector_checkbox};
use crate::theme::{FONT_FAMILY_MONO, ThemeTokens, active_theme};

pub type SelectParsedFileCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type ToggleFileCheckCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type ApplyActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct FileChangesListComponent {
    files: Vec<ParsedResponseFile>,
    snapshots: Vec<BackupSnapshotInfo>,
    selected_index: usize,
    backups_enabled: bool,
    options_open: bool,
    status_message: Option<String>,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_select: Option<SelectParsedFileCallback>,
    on_toggle_file_select: Option<ToggleFileCheckCallback>,
    on_toggle_backup: Option<ApplyActionCallback>,
    on_restore_snapshot: Option<RestoreSnapshotCallback>,
    on_toggle_options: Option<ApplyActionCallback>,
    on_apply: Option<ApplyActionCallback>,
    on_undo: Option<ApplyActionCallback>,
}

impl fmt::Debug for FileChangesListComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileChangesListComponent")
            .field("files_count", &self.files.len())
            .field("selected_index", &self.selected_index)
            .field("backups_enabled", &self.backups_enabled)
            .finish_non_exhaustive()
    }
}

impl FileChangesListComponent {
    #[inline]
    pub fn new(files: Vec<ParsedResponseFile>, selected_index: usize) -> Self {
        Self {
            files,
            snapshots: Vec::new(),
            selected_index,
            backups_enabled: true,
            options_open: false,
            status_message: None,
            labels: AppLocale::Ru.labels(),
            theme: active_theme(),
            on_select: None,
            on_toggle_file_select: None,
            on_toggle_backup: None,
            on_restore_snapshot: None,
            on_toggle_options: None,
            on_apply: None,
            on_undo: None,
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
    pub const fn with_backups_enabled(mut self, enabled: bool) -> Self {
        self.backups_enabled = enabled;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_options_open(mut self, open: bool) -> Self {
        self.options_open = open;
        self
    }

    #[must_use]
    #[inline]
    pub fn with_status(mut self, status: Option<String>) -> Self {
        self.status_message = status;
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
    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_file_select(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_file_select = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_backup(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_backup = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_options(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_options = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn with_apply_actions(
        mut self,
        on_apply: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_undo: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_apply = Some(Arc::new(on_apply));
        self.on_undo = Some(Arc::new(on_undo));
        self
    }

    fn render_tabs(&self, theme: &'static ThemeTokens) -> Div {
        let mut tabs_row =
            div().flex().flex_row().flex_wrap().items_center().gap(px(theme.spacing.xs));

        for (idx, file) in self.files.iter().enumerate() {
            let tab_label = format_disambiguated_tab_name(&file.rel_path, &self.files);
            let is_blocked = !file.is_sandbox_safe || file.has_conflict;
            let leading_indicator = self.render_tab_leading_indicator(idx, file, is_blocked, theme);

            let is_active = idx == self.selected_index;
            let text_color = if is_blocked {
                theme.colors.danger
            } else if !file.is_selected && !file.is_applied {
                theme.colors.text_muted
            } else if is_active {
                theme.colors.text_primary
            } else {
                theme.colors.text_secondary
            };

            let hover_bg = theme.colors.bg_hover;
            let mut tab = div()
                .id(("parsed-file-tab", idx))
                .h(px(theme.layout.control_height_sm))
                .px(px(theme.spacing.md))
                .rounded(px(theme.radius.md))
                .border_1()
                .border_color(if is_active {
                    theme.colors.border_active
                } else {
                    theme.colors.border
                })
                .bg(if is_active { theme.colors.bg_elevated } else { theme.colors.bg_panel })
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.xs))
                .child(leading_indicator)
                .child(
                    div()
                        .font_family(FONT_FAMILY_MONO)
                        .text_xs()
                        .text_color(text_color)
                        .child(SharedString::from(tab_label)),
                );

            if !file.is_applied && file.added_lines > 0 {
                tab = tab.child(
                    div()
                        .font_family(FONT_FAMILY_MONO)
                        .text_xs()
                        .text_color(theme.colors.success)
                        .child(format!("+{}", file.added_lines)),
                );
            }

            if !file.is_applied && file.removed_lines > 0 {
                tab = tab.child(
                    div()
                        .font_family(FONT_FAMILY_MONO)
                        .text_xs()
                        .text_color(theme.colors.danger)
                        .child(format!("-{}", file.removed_lines)),
                );
            }

            if let Some(ref cb) = self.on_select {
                let cb_clone = Arc::clone(cb);
                tab = tab.on_click(move |_ev, win, cx| cb_clone(idx, win, cx));
            }
            tabs_row = tabs_row.child(tab);
        }

        tabs_row
    }

    fn render_tab_leading_indicator(
        &self,
        idx: usize,
        file: &ParsedResponseFile,
        is_blocked: bool,
        theme: &'static ThemeTokens,
    ) -> gpui::AnyElement {
        if is_blocked {
            return div()
                .w(px(6.0))
                .h(px(6.0))
                .rounded(px(99.0))
                .bg(theme.colors.danger)
                .into_any_element();
        }
        if file.is_applied {
            return render_icon(UiIcon::Check, 12.0, theme.colors.text_muted).into_any_element();
        }

        let (check_background, check_border, check_foreground) = if file.is_selected {
            (theme.colors.bg_app, theme.colors.border_active, theme.colors.text_primary)
        } else {
            (theme.colors.bg_app, theme.colors.border, theme.colors.text_muted)
        };

        let mut check_el = render_vector_checkbox(
            file.is_selected,
            false,
            12.0,
            check_background,
            check_border,
            check_foreground,
            theme.radius.sm,
        )
        .id(("parsed-file-check", idx));

        if let Some(ref toggle_fn) = self.on_toggle_file_select {
            let cb_toggle = Arc::clone(toggle_fn);
            check_el = check_el.cursor_pointer().on_click(move |_ev, win, cx| {
                cx.stop_propagation();
                cb_toggle(idx, win, cx);
            });
        }

        check_el.into_any_element()
    }
}

fn format_disambiguated_tab_name(rel_path: &str, all_files: &[ParsedResponseFile]) -> String {
    let segments: Vec<&str> = rel_path.split(['/', '\\']).filter(|s| !s.is_empty()).collect();
    let Some(&file_name) = segments.last() else {
        return rel_path.to_owned();
    };

    let is_generic_name = matches!(
        file_name,
        "mod.rs" | "lib.rs" | "view.rs" | "state.rs" | "main.rs" | "index.ts" | "index.js"
    );
    let has_duplicate = all_files
        .iter()
        .filter(|f| {
            let name = f
                .rel_path
                .rfind(['/', '\\'])
                .and_then(|p| f.rel_path.get(p + 1..))
                .unwrap_or(&f.rel_path);
            name == file_name
        })
        .count()
        > 1;

    if (is_generic_name || has_duplicate)
        && segments.len() >= 2
        && let Some(&parent) = segments.get(segments.len() - 2)
    {
        return format!("{parent}/{file_name}");
    }

    file_name.to_owned()
}

impl RenderOnce for FileChangesListComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let labels = self.labels;
        let files_count = self.files.len();
        let applicable_selected_count = self
            .files
            .iter()
            .filter(|f| f.is_selected && !f.is_applied && f.is_sandbox_safe && !f.has_conflict)
            .count();
        let tabs_row = self.render_tabs(theme);

        let _ = &self.status_message;
        let mut right_controls =
            div().flex_shrink_0().flex().flex_row().items_center().gap(px(theme.spacing.xs));

        let mut options_btn =
            UiButton::icon_only("btn-applier-options-toggle", UiIcon::SlidersHorizontal)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme)
                .active(self.options_open);
        if let Some(toggle_opts) = self.on_toggle_options {
            options_btn = options_btn.on_click(move |ev, w, c| toggle_opts(ev, w, c));
        }
        right_controls = right_controls.child(options_btn);

        if let Some(undo_fn) = self.on_undo {
            right_controls = right_controls.child(
                UiButton::icon_only("btn-undo-last-apply", UiIcon::RotateCcw)
                    .variant(ButtonVariant::Ghost)
                    .with_theme(theme)
                    .on_click(move |ev, w, c| undo_fn(ev, w, c)),
            );
        }

        let mut apply_btn = UiButton::new(
            "btn-apply-changes-cta",
            format!("{} ({applicable_selected_count})", labels.btn_apply_changes),
        )
        .icon(UiIcon::Check)
        .variant(ButtonVariant::SuccessCta)
        .disabled(applicable_selected_count == 0)
        .with_theme(theme);

        if let Some(apply_fn) = self.on_apply {
            apply_btn = apply_btn.on_click(move |ev, w, c| apply_fn(ev, w, c));
        }
        right_controls = right_controls.child(apply_btn);

        let inline_tabs = files_count > 0 && files_count <= 2;
        let (left_header_content, overflow_tabs_opt) = if inline_tabs {
            (
                div().flex_1().min_w(px(0.0)).overflow_hidden().child(tabs_row).into_any_element(),
                None,
            )
        } else {
            (
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_xs()
                    .text_color(theme.colors.text_secondary)
                    .child(if files_count == 0 {
                        format!("{} 0", labels.applier_parsed_files_prefix)
                    } else {
                        format!(
                            "{} {applicable_selected_count}/{files_count}",
                            labels.applier_parsed_files_prefix
                        )
                    })
                    .into_any_element(),
                (files_count > 2).then_some(tabs_row),
            )
        };

        let mut container = div().w_full().bg(theme.colors.bg_panel).flex().flex_col().child(
            div()
                .w_full()
                .h(px(theme.layout.pane_header_height))
                .px(px(theme.spacing.lg))
                .border_b_1()
                .border_color(theme.colors.border)
                .overflow_hidden()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(theme.spacing.xs))
                .child(left_header_content)
                .child(right_controls),
        );

        if self.options_open {
            let mut drawer = BackupDrawerComponent::new(self.backups_enabled)
                .with_snapshots(self.snapshots)
                .with_context(labels, theme);
            if let Some(toggle_bak) = self.on_toggle_backup {
                drawer = drawer.on_toggle(move |ev, w, c| toggle_bak(ev, w, c));
            }
            if let Some(restore_snap) = self.on_restore_snapshot {
                drawer = drawer.on_restore_snapshot(move |id, w, c| restore_snap(id, w, c));
            }
            container = container.child(drawer);
        }

        if let Some(overflow_tabs) = overflow_tabs_opt {
            container = container.child(
                div()
                    .id("parsed-files-overflow-scroll")
                    .w_full()
                    .max_h(px(112.0))
                    .overflow_y_scroll()
                    .px(px(theme.spacing.lg))
                    .py(px(theme.spacing.xs))
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(overflow_tabs),
            );
        }

        container
    }
}
