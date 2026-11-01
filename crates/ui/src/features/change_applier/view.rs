use std::time::Duration;

use core::shared_kernel::fs::line_ending::LineEnding;
use gpui::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::features::change_applier::components::diff_view::{DiffViewComponent, DiffViewMeta};
use crate::features::change_applier::components::file_changes_list::FileChangesListComponent;
use crate::features::change_applier::components::response_editor::ResponseEditorComponent;
use crate::features::change_applier::state::{ApplierStatusNotice, ChangeApplierState};
use crate::locale::AppLocale;
use crate::shared::containers::split_pane::SplitPane;
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{FONT_FAMILY_UI, ThemeTokens, ThemeVariant};

#[derive(Debug, Clone)]
pub struct ChangeApplierView {
    state: Entity<ChangeApplierState>,
    locale: AppLocale,
    theme_variant: ThemeVariant,
}

impl ChangeApplierView {
    pub fn new(
        state: Entity<ChangeApplierState>,
        locale: AppLocale,
        theme_variant: ThemeVariant,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_this, _model, cx| {
            cx.notify();
        })
        .detach();
        Self { state, locale, theme_variant }
    }

    pub fn sync_preferences(
        &mut self,
        locale: AppLocale,
        theme_variant: ThemeVariant,
        cx: &mut Context<Self>,
    ) {
        if self.locale != locale || self.theme_variant != theme_variant {
            self.locale = locale;
            self.theme_variant = theme_variant;
            cx.notify();
        }
    }

    fn spawn_toast_auto_dismiss(state: Entity<ChangeApplierState>, cx: &mut gpui::App) {
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_secs(4)).await;
            let _ = state.update(cx, |s, cx| {
                s.clear_status_notice();
                cx.notify();
            });
        })
        .detach();
    }

    fn render_toast_notification(
        &self,
        notice: &ApplierStatusNotice,
        text: String,
        theme: &'static ThemeTokens,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let (tone, icon) = match notice {
            ApplierStatusNotice::Applied { .. } => (colors.badge_success, UiIcon::Check),
            ApplierStatusNotice::Undone { .. } => (colors.badge_accent, UiIcon::RotateCcw),
            ApplierStatusNotice::Error { .. } => (colors.banner_critical, UiIcon::Info),
        };
        let state_dismiss = self.state.clone();
        let hover_bg = colors.bg_hover;

        div()
            .id("applier-toast-notification")
            .absolute()
            .bottom(px(theme.spacing.xxl))
            .right(px(theme.spacing.xxl))
            .max_w(px(420.0))
            .px(px(theme.spacing.xl))
            .py(px(theme.spacing.md))
            .rounded(px(theme.radius.lg))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(tone.border)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.md))
            .child(render_icon(icon, 15.0, tone.fg))
            .child(
                div()
                    .flex_1()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_primary)
                    .child(SharedString::from(text)),
            )
            .child(
                div()
                    .id("applier-toast-close-btn")
                    .p(px(theme.spacing.xxs))
                    .rounded(px(theme.radius.sm))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_click(move |_ev, _win, cx| {
                        state_dismiss.update(cx, |s, cx| {
                            s.clear_status_notice();
                            cx.notify();
                        });
                    })
                    .child(render_icon(UiIcon::X, 13.0, colors.text_secondary)),
            )
    }
}

impl Render for ChangeApplierView {
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_variant.tokens();
        let labels = self.locale.labels();
        let snap = self.state.read(cx);
        let flags = snap.autofix_flags();
        let response_buf = snap.response_buffer().clone();
        let parsed_files = snap.parsed_files().to_vec();
        let parsed_count = parsed_files.len();
        let selected_idx = snap.selected_file_index();
        let selected_buf = snap.selected_file_buffer().clone();
        let status_notice_opt = snap.status_notice().cloned();
        let status_msg = snap.formatted_status_message(labels);
        let backups_enabled = snap.backups_enabled();
        let options_open = snap.is_options_open();

        let diff_meta = parsed_files.get(selected_idx).map_or_else(
            || DiffViewMeta {
                file_path: labels.applier_no_parsed_files.to_owned(),
                added_lines: 0,
                removed_lines: 0,
                line_ending: LineEnding::Lf,
                is_truncated: false,
                is_sandbox_safe: true,
                has_conflict: false,
            },
            |f| DiffViewMeta {
                file_path: f.rel_path.clone(),
                added_lines: f.added_lines,
                removed_lines: f.removed_lines,
                line_ending: f.line_ending,
                is_truncated: f.is_truncated,
                is_sandbox_safe: f.is_sandbox_safe,
                has_conflict: f.has_conflict,
            },
        );

        let state_paste = self.state.clone();
        let state_fix = self.state.clone();
        let left_editor = ResponseEditorComponent::new(
            response_buf,
            flags.preamble_stripped,
            flags.closed_truncated_block,
        )
        .with_parsed_count(parsed_count)
        .with_context(labels, theme)
        .with_actions(
            move |_ev, _win, cx| {
                let clip_str = cx.read_from_clipboard().and_then(|item| item.text());
                state_paste.update(cx, |s, cx| {
                    if let Some(text) = clip_str {
                        s.set_response_text(text);
                    } else {
                        s.load_response_from_clipboard();
                    }
                    cx.notify();
                });
            },
            move |_ev, _win, cx| {
                state_fix.update(cx, |s, cx| {
                    s.run_autofix();
                    cx.notify();
                });
            },
        );

        let state_sel = self.state.clone();
        let state_bak = self.state.clone();
        let state_opts = self.state.clone();
        let state_apply = self.state.clone();
        let state_undo = self.state.clone();
        let right_header_tabs = FileChangesListComponent::new(parsed_files, selected_idx)
            .with_backups_enabled(backups_enabled)
            .with_options_open(options_open)
            .with_context(labels, theme)
            .on_select(move |idx, _win, cx| {
                state_sel.update(cx, |s, cx| {
                    s.select_parsed_file(idx);
                    cx.notify();
                });
            })
            .on_toggle_backup(move |_ev, _win, cx| {
                state_bak.update(cx, |s, cx| {
                    s.toggle_backups_enabled();
                    cx.notify();
                });
            })
            .on_toggle_options(move |_ev, _win, cx| {
                state_opts.update(cx, |s, cx| {
                    s.toggle_options_open();
                    cx.notify();
                });
            })
            .with_apply_actions(
                move |_ev, _win, cx| {
                    let is_ok = state_apply.update(cx, |s, cx| {
                        let res = s.apply_changes_to_disk().is_ok();
                        cx.notify();
                        res
                    });
                    if is_ok {
                        Self::spawn_toast_auto_dismiss(state_apply.clone(), cx);
                    }
                },
                move |_ev, _win, cx| {
                    let is_ok = state_undo.update(cx, |s, cx| {
                        let res = s.undo_last_apply().is_ok();
                        cx.notify();
                        res
                    });
                    if is_ok {
                        Self::spawn_toast_auto_dismiss(state_undo.clone(), cx);
                    }
                },
            );

        let diff_view = DiffViewComponent::new(diff_meta, selected_buf).with_context(labels, theme);

        let right_col = div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(right_header_tabs)
            .child(div().flex_1().overflow_hidden().child(diff_view));

        let mut root = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_app)
            .overflow_hidden()
            .child(
                SplitPane::new(theme.layout.applier_split_ratio, left_editor, right_col)
                    .with_theme(theme),
            );

        if let (Some(notice), Some(msg)) = (status_notice_opt, status_msg) {
            root = root.child(self.render_toast_notification(&notice, msg, theme));
        }

        root
    }
}
