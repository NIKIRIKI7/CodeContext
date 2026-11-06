use core::packing::aggregates::project::Project;
use core::packing::ports::clipboard::ClipboardPort as _;
use core::preset::models::resolution::{
    TerminalCommandPreset, TerminalTaskCategory, UiChromeConfig,
};
use core::system::ports::updater::{SemanticVersion, UpdateChannel, UpdateStage, UpdaterPort as _};
use gpui::{
    App, ClipboardItem, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, px,
};

use crate::features::main_window::components::activity_rail::ActivityRailComponent;
use crate::features::main_window::components::diagnostics_dock::{
    DiagnosticsDockComponent, DiagnosticsDockProps, DockToolTab,
};
use crate::features::main_window::components::project_meta_modal::{
    ProjectMetaModalComponent, ProjectMetaProps,
};
use crate::features::main_window::components::recent_projects_popover::RecentProjectsPopover;
use crate::features::main_window::state::WorkspaceTab;
use crate::features::main_window::view::MainWindowView;
use crate::locale::AppLocale;
use crate::shared::containers::modal_dialog::ModalDialogShell;
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, format_project_path_smart};

impl MainWindowView {
    pub(crate) fn render_top_bar(
        &self,
        active_tab: WorkspaceTab,
        show_fallback_settings: bool,
        locale: AppLocale,
        theme: &'static ThemeTokens,
        project_display: &str,
    ) -> impl IntoElement + use<> {
        let labels = locale.labels();
        let compact_project = format_project_path_smart(project_display, 48);
        let left_controls = self.render_project_controls(&compact_project, labels, theme);
        let mode_title = match active_tab {
            WorkspaceTab::ContextPacker => labels.tab_context_packer,
            WorkspaceTab::ChangeApplier => labels.tab_change_applier,
        };

        let fallback_settings_opt = show_fallback_settings.then(|| {
            let state_settings_top = self.state.clone();
            UiButton::icon_only("top-bar-fallback-settings-btn", UiIcon::Settings)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme)
                .on_click(move |_ev, _win, cx| {
                    state_settings_top.update(cx, |s, cx| {
                        s.toggle_settings();
                        cx.notify();
                    });
                })
        });

        div()
            .w_full()
            .h(px(theme.layout.top_bar_height))
            .flex_shrink_0()
            .px(px(theme.spacing.xl))
            .bg(theme.colors.bg_panel)
            .border_b_1()
            .border_color(theme.colors.border)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(theme.spacing.md))
            .child(left_controls)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(theme.spacing.sm))
                    .child(
                        div()
                            .font_family(FONT_FAMILY_MONO)
                            .text_xs()
                            .text_color(theme.colors.text_muted)
                            .child(mode_title),
                    )
                    .children(fallback_settings_opt),
            )
    }

    fn render_project_controls(
        &self,
        project_display: &str,
        labels: &'static crate::locale::UiLabels,
        theme: &'static ThemeTokens,
    ) -> impl IntoElement + use<> {
        let state_recent = self.state.clone();
        let colors = &theme.colors;
        let hover_bg = colors.bg_hover;

        let projects_dropdown_btn = div()
            .id("nav-btn-open-project-folder")
            .h(px(theme.layout.control_height))
            .px(px(theme.spacing.md))
            .rounded(px(theme.radius.md))
            .border_1()
            .border_color(colors.border)
            .bg(colors.bg_elevated)
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xs))
            .on_click(move |_ev, _win, cx| {
                cx.stop_propagation();
                state_recent.update(cx, |s, cx| {
                    s.toggle_recent_projects();
                    cx.notify();
                });
            })
            .child(render_icon(UiIcon::Folder, 14.0, colors.text_secondary))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_primary)
                    .child(labels.btn_projects_menu),
            )
            .child(render_icon(UiIcon::ChevronDown, 12.0, colors.text_muted));

        div()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.md))
            .child(projects_dropdown_btn)
            .child(
                div()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(SharedString::from(project_display.to_owned())),
            )
    }

    pub(crate) fn build_recent_projects_popover(
        &self,
        locale: AppLocale,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> RecentProjectsPopover {
        let labels = locale.labels();
        let submenu_open = self.state.read(cx).is_recent_submenu_open();
        let packer_snap = self.packer_state.read(cx);
        let recent_paths = packer_snap.resolved_config().global_config.recent_projects.clone();
        let active_path = packer_snap.project_root().map(|p| p.display().to_string());

        let packer_sel = self.packer_state.clone();
        let applier_sel = self.applier_state.clone();
        let state_sel = self.state.clone();

        let packer_browse = self.packer_state.clone();
        let applier_browse = self.applier_state.clone();
        let state_browse = self.state.clone();
        let prompt_open_label = labels.btn_open_folder_prompt;

        let state_submenu = self.state.clone();
        let state_close = self.state.clone();

        RecentProjectsPopover::new(recent_paths, active_path, labels, theme)
            .with_submenu_open(submenu_open)
            .on_set_submenu_open(move |open, _win, cx: &mut App| {
                state_submenu.update(cx, |s, cx| {
                    if s.set_recent_submenu_open(open) {
                        cx.notify();
                    }
                });
            })
            .with_actions(
                move |chosen_dir, _win, cx| {
                    state_sel.update(cx, |s, cx| {
                        s.set_recent_projects_open(false);
                        cx.notify();
                    });
                    let applier_dir = chosen_dir.clone();
                    packer_sel.update(cx, |p, cx| {
                        let _ = p.open_project(chosen_dir);
                        cx.notify();
                    });
                    applier_sel.update(cx, |a, cx| {
                        a.set_project_root(Some(applier_dir));
                        cx.notify();
                    });
                },
                move |_ev, _win, cx| {
                    state_browse.update(cx, |s, cx| {
                        s.set_recent_projects_open(false);
                        cx.notify();
                    });
                    Self::prompt_and_open_project_folder(
                        packer_browse.clone(),
                        applier_browse.clone(),
                        prompt_open_label,
                        cx,
                    );
                },
                move |_ev, _win, cx| {
                    state_close.update(cx, |s, cx| {
                        s.set_recent_projects_open(false);
                        cx.notify();
                    });
                },
            )
    }

    pub(crate) fn build_meta_modal_overlay(
        &self,
        locale: AppLocale,
        theme: &'static ThemeTokens,
        project_display: String,
        cx: &Context<Self>,
    ) -> ProjectMetaModalComponent {
        let labels = locale.labels();
        let snap = self.packer_state.read(cx);
        let preset = snap.effective_preset().clone();
        let origin = snap.resolved_config().origin;
        let selected_files = snap.tree_projection().selected_files();
        let total_indexed_files = snap.tree_projection().total_indexed_files();
        let total_lines = snap.preview_buffer().line_count();

        let (total_bytes, blocked_secrets_count, skipped_binary_count) =
            snap.pack_result().map_or_else(
                || (snap.project().map_or(0, Project::estimated_pack_capacity_bytes), 0, 0),
                |res| (res.total_bytes as u64, res.blocked_secrets_count, res.skipped_binary_count),
            );

        let props = ProjectMetaProps {
            project_path: project_display,
            origin,
            preset,
            total_bytes,
            selected_files,
            total_indexed_files,
            total_lines,
            blocked_secrets_count,
            skipped_binary_count,
            labels,
            theme,
        };

        let packer_save = self.packer_state.clone();
        let state_close = self.state.clone();
        ProjectMetaModalComponent::new(props).with_actions(
            move |_ev, _win, cx| {
                packer_save.update(cx, |p, cx| {
                    let _ = p.save_current_preset_to_project();
                    cx.notify();
                });
            },
            move |_ev, _win, cx| {
                state_close.update(cx, |s, cx| {
                    s.set_project_meta_open(false);
                    cx.notify();
                });
            },
        )
    }

    pub(crate) fn render_update_available_modal(
        &self,
        new_version: SemanticVersion,
        channel: UpdateChannel,
        locale: AppLocale,
        theme: &'static ThemeTokens,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = locale.labels();
        let current_ver = self.container.updater().current_version();
        let state_dismiss = self.state.clone();
        let state_dismiss_btn = self.state.clone();
        let update_now_btn = self.build_update_now_button(labels, theme);

        ModalDialogShell::new(
            "update-available-modal-backdrop",
            "update-available-modal-dialog",
            "update-available-close-btn",
            labels.update_modal_title,
        )
        .width(460.0)
        .leading_icon(UiIcon::Package)
        .with_theme(theme)
        .on_close(move |_ev, _win, cx| {
            state_dismiss.update(cx, |s, cx| {
                s.dismiss_update_modal();
                cx.notify();
            });
        })
        .child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_secondary)
                .child(labels.update_modal_subtitle),
        )
        .child(
            div()
                .p(px(theme.spacing.lg))
                .rounded(px(theme.radius.md))
                .bg(colors.bg_elevated)
                .border_1()
                .border_color(colors.border)
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_primary)
                .child(SharedString::from(format!(
                    "{}: v{current_ver}  →  {}: v{new_version} ({})",
                    labels.update_modal_current_label,
                    labels.update_modal_new_label,
                    channel.display_name()
                ))),
        )
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .justify_end()
                .gap(px(theme.spacing.sm))
                .child(
                    UiButton::new("btn-update-later", labels.btn_update_later)
                        .variant(ButtonVariant::Secondary)
                        .with_theme(theme)
                        .on_click(move |_ev, _win, cx| {
                            state_dismiss_btn.update(cx, |s, cx| {
                                s.dismiss_update_modal();
                                cx.notify();
                            });
                        }),
                )
                .child(update_now_btn),
        )
    }

    pub(crate) fn build_activity_rail(
        &self,
        active_tab: WorkspaceTab,
        settings_open: bool,
        labels: &'static crate::locale::UiLabels,
        theme: &'static ThemeTokens,
    ) -> ActivityRailComponent {
        let state_rail_tab = self.state.clone();
        let packer_for_tab = self.packer_state.clone();
        let applier_for_auto = self.applier_state.clone();
        let settings_for_auto = self.settings_state.clone();
        let state_rail_settings = self.state.clone();

        ActivityRailComponent::new(active_tab, settings_open, labels, theme).with_actions(
            move |tab, _win, cx: &mut App| {
                let auto_clip =
                    settings_for_auto.read(cx).global_config().auto_read_clipboard_on_apply;
                let mut should_load_clip = false;
                state_rail_tab.update(cx, |s, cx| {
                    should_load_clip = s.switch_to_tab(tab, auto_clip);
                    cx.notify();
                });
                if tab == WorkspaceTab::ContextPacker {
                    packer_for_tab.update(cx, |p, cx| {
                        if p.sync_with_disk_if_changed() {
                            cx.notify();
                        }
                    });
                } else if should_load_clip {
                    applier_for_auto.update(cx, |a, cx| {
                        a.load_response_from_clipboard_if_changed();
                        cx.notify();
                    });
                }
            },
            move |_ev, _win, cx: &mut App| {
                state_rail_settings.update(cx, |s, cx| {
                    s.toggle_settings();
                    cx.notify();
                });
            },
        )
    }

    pub(crate) fn render_bottom_status_bar(
        &self,
        telemetry_summary: core::shared_kernel::execution::telemetry::TelemetrySummary,
        chrome: &UiChromeConfig,
        labels: &'static crate::locale::UiLabels,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let snap = self.state.read(cx);
        let dock_open = snap.is_logs_dock_open();
        let active_dock_tab = snap.dock_active_tab();
        let has_slow =
            telemetry_summary.slow_or_warn_count > 0 || telemetry_summary.error_count > 0;
        let total_logs_count = telemetry_summary.total_count;

        let state_term = self.state.clone();
        let state_logs = self.state.clone();
        let hover_bg = colors.bg_hover;

        let is_term_active = dock_open && active_dock_tab == DockToolTab::Terminal;
        let is_logs_active = dock_open && active_dock_tab == DockToolTab::Logs;

        let mut row = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));

        if chrome.show_terminal_button {
            let term_btn = div()
                .id("zed-bottom-bar-terminal-btn")
                .h(px(22.0))
                .px(px(theme.spacing.sm))
                .rounded(px(theme.radius.sm))
                .bg(if is_term_active { colors.bg_elevated } else { colors.bg_panel })
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.xxs))
                .on_click(move |_ev, _win, cx| {
                    cx.stop_propagation();
                    state_term.update(cx, |s, cx| {
                        s.open_or_toggle_dock_tab(DockToolTab::Terminal);
                        if s.is_logs_dock_open() {
                            s.bottom_dock_mut().terminal_focused = true;
                        }
                        cx.notify();
                    });
                })
                .child(render_icon(
                    UiIcon::Terminal,
                    12.0,
                    if is_term_active { colors.cyan } else { colors.text_secondary },
                ))
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(if is_term_active {
                            colors.text_primary
                        } else {
                            colors.text_secondary
                        })
                        .child(labels.dock_tab_terminal),
                );
            row = row.child(term_btn);
        }

        if chrome.show_logs_button {
            let logs_btn = div()
                .id("zed-bottom-bar-logs-btn")
                .h(px(22.0))
                .px(px(theme.spacing.sm))
                .rounded(px(theme.radius.sm))
                .bg(if is_logs_active { colors.bg_elevated } else { colors.bg_panel })
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.xxs))
                .on_click(move |_ev, _win, cx| {
                    cx.stop_propagation();
                    state_logs.update(cx, |s, cx| {
                        s.open_or_toggle_dock_tab(DockToolTab::Logs);
                        cx.notify();
                    });
                })
                .child(div().w(px(6.0)).h(px(6.0)).rounded(px(99.0)).bg(if has_slow {
                    colors.warning
                } else {
                    colors.success
                }))
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(if is_logs_active {
                            colors.text_primary
                        } else {
                            colors.text_secondary
                        })
                        .child(SharedString::from(format!(
                            "{} ({total_logs_count})",
                            labels.dock_tab_logs
                        ))),
                );
            row = row.child(logs_btn);
        }

        div()
            .w_full()
            .h(px(theme.layout.status_bar_height))
            .min_h(px(theme.layout.status_bar_height))
            .px(px(theme.spacing.lg))
            .bg(colors.bg_panel)
            .border_t_1()
            .border_color(colors.border)
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .child(row)
    }

    pub(crate) fn build_diagnostics_dock(
        &self,
        summary: core::shared_kernel::execution::telemetry::TelemetrySummary,
        chrome: &UiChromeConfig,
        labels: &'static crate::locale::UiLabels,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> DiagnosticsDockComponent {
        let dock_snap = self.state.read(cx).bottom_dock().clone();
        let terminal_presets =
            self.settings_state.read(cx).global_config().terminal_presets.clone();
        let filter_mode = dock_snap.log_filter_mode;
        let active_tab = dock_snap.active_tab;
        let entries = self.container.telemetry().entries_filtered(filter_mode);
        let show_inline_categories =
            !chrome.show_bottom_bar || !chrome.show_terminal_button || !chrome.show_logs_button;

        let props = DiagnosticsDockProps {
            active_tab,
            entries,
            summary,
            filter_mode,
            task_category: dock_snap.task_category,
            terminal_presets,
            terminal_input: dock_snap.terminal_input,
            terminal_focused: dock_snap.terminal_focused,
            terminal_running: dock_snap.terminal_running,
            dock_height_px: dock_snap.dock_height_px,
            is_resizing_dock: dock_snap.resizing_dock.is_some(),
            show_terminal_presets: chrome.show_terminal_presets,
            show_inline_categories,
            terminal_history: dock_snap.terminal_history,
            labels,
            theme,
        };

        let dock = self.attach_dock_terminal_handlers(DiagnosticsDockComponent::new(props), cx);
        let state_filter = self.state.clone();
        let container_copy = self.container.clone();
        let state_copy = self.state.clone();
        let container_clear = self.container.clone();
        let state_clear = self.state.clone();
        let state_close = self.state.clone();

        let container_ai_err = self.container.clone();
        let dock = dock.on_copy_error_for_ai(move |cmd, output, _win, cx| {
            let prompt = format!(
                "Команда `{cmd}` завершилась с ошибкой:\n```text\n{output}\n```\nИсправь причину ошибки и выдай патч в формате SEARCH / REPLACE."
            );
            let _ = container_ai_err.clipboard().copy_text(&prompt);
            cx.write_to_clipboard(ClipboardItem::new_string(prompt));
        });

        dock.with_actions(
            move |mode, _win, cx| {
                state_filter.update(cx, |s, cx| {
                    s.set_log_filter_mode(mode);
                    cx.notify();
                });
            },
            move |_ev, _win, cx| {
                let text = if active_tab == DockToolTab::Logs {
                    container_copy.telemetry().export_plain_text(filter_mode)
                } else {
                    state_copy
                        .read(cx)
                        .bottom_dock()
                        .terminal_history
                        .iter()
                        .map(|h| format!("$ {}\n{}", h.command, h.output))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                };
                if !text.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            },
            move |_ev, _win, cx| {
                if active_tab == DockToolTab::Logs {
                    container_clear.telemetry().clear();
                }
                state_clear.update(cx, |s, cx| {
                    if active_tab == DockToolTab::Terminal {
                        s.bottom_dock_mut().terminal_history.clear();
                    }
                    cx.notify();
                });
            },
            move |_ev, _win, cx| {
                state_close.update(cx, |s, cx| {
                    s.set_logs_dock_open(false);
                    cx.notify();
                });
            },
        )
    }

    fn attach_dock_terminal_handlers(
        &self,
        dock: DiagnosticsDockComponent,
        cx: &Context<Self>,
    ) -> DiagnosticsDockComponent {
        let state_tab = self.state.clone();
        let state_cat = self.state.clone();
        let state_save_preset = self.state.clone();
        let settings_save_preset = self.settings_state.clone();
        let settings_del_preset = self.settings_state.clone();
        let state_resize_start = self.state.clone();
        let state_focus_term = self.state.clone();
        let view_submit_term = cx.entity().clone();
        let view_preset_term = cx.entity().clone();

        dock.on_select_tab(move |tab, _win, cx| {
            state_tab.update(cx, |s, cx| {
                s.set_dock_tab(tab);
                if tab == DockToolTab::Terminal {
                    s.bottom_dock_mut().terminal_focused = true;
                }
                cx.notify();
            });
        })
        .on_select_task_category(move |cat_opt, _win, cx| {
            state_cat.update(cx, |s, cx| {
                s.bottom_dock_mut().task_category = cat_opt;
                cx.notify();
            });
        })
        .on_preset_management(
            move |_ev, _win, cx| {
                let (raw_cmd, cat) = {
                    let d = state_save_preset.read(cx).bottom_dock();
                    (
                        d.terminal_input.trim().to_owned(),
                        d.task_category.unwrap_or(TerminalTaskCategory::Custom),
                    )
                };
                if !raw_cmd.is_empty() {
                    let preset_count =
                        settings_save_preset.read(cx).global_config().terminal_presets.len();
                    let preset = TerminalCommandPreset {
                        id: format!("custom_cmd_{}", preset_count + 1),
                        name: raw_cmd.clone(),
                        command: raw_cmd,
                        category: cat,
                        is_custom: true,
                    };
                    settings_save_preset.update(cx, |ss, cx| {
                        let _ = ss.add_terminal_preset(preset);
                        cx.notify();
                    });
                }
            },
            move |idx, _win, cx| {
                settings_del_preset.update(cx, |ss, cx| {
                    let _ = ss.remove_terminal_preset(idx);
                    cx.notify();
                });
            },
            move |ev, _win, cx| {
                let start_y = ev.position.y / px(1.0);
                state_resize_start.update(cx, |s, cx| {
                    s.start_dock_resize(start_y);
                    cx.notify();
                });
            },
        )
        .on_terminal_actions(
            move |_ev, _win, cx| {
                state_focus_term.update(cx, |s, cx| {
                    s.bottom_dock_mut().terminal_focused = true;
                    cx.notify();
                });
            },
            move |_ev, _win, cx| {
                view_submit_term.update(cx, |this, cx| {
                    let cmd = this.state.update(cx, |s, cx| {
                        let taken = std::mem::take(&mut s.bottom_dock_mut().terminal_input);
                        cx.notify();
                        taken
                    });
                    this.spawn_terminal_command(&cmd, cx);
                });
            },
            move |preset_cmd, _win, cx| {
                view_preset_term.update(cx, |this, cx| {
                    this.spawn_terminal_command(preset_cmd, cx);
                });
            },
        )
    }

    fn build_update_now_button(
        &self,
        labels: &'static crate::locale::UiLabels,
        theme: &'static ThemeTokens,
    ) -> UiButton {
        let state_now = self.state.clone();
        let settings_now = self.settings_state.clone();
        let container_now = self.container.clone();

        UiButton::new("btn-update-now", labels.btn_update_now)
            .variant(ButtonVariant::Primary)
            .with_theme(theme)
            .on_click(move |_ev, _win, cx| {
                state_now.update(cx, |s, cx| {
                    s.close_update_modal();
                    cx.notify();
                });
                let c_bg = container_now.clone();
                let c_restart = container_now.clone();
                let settings_after = settings_now.clone();
                cx.spawn(async move |cx| {
                    let res = cx
                        .background_executor()
                        .spawn(async move { c_bg.updater().apply_staged_update() })
                        .await;
                    let _ = settings_after.update(cx, |s, cx| {
                        s.sync_from_container();
                        cx.notify();
                    });
                    if matches!(res, Ok(UpdateStage::ReadyToRestart { .. }))
                        && c_restart.updater().restart_into_updated_binary().is_ok()
                    {
                        let _ = cx.update(|app| app.quit());
                    }
                })
                .detach();
            })
    }

    pub(crate) fn execute_selected_command_palette_item(&mut self, cx: &mut Context<Self>) {
        let (query, sel_idx) = {
            let cp = self.state.read(cx).command_palette();
            (cp.query.clone(), cp.selected_index)
        };
        self.state.update(cx, |s, cx| {
            s.close_command_palette();
            cx.notify();
        });

        let trimmed = query.trim();
        if let Some(cmd_q) = trimmed.strip_prefix('>') {
            let q = cmd_q.trim().to_ascii_lowercase();
            let cmds = [
                "copy", "apply", "undo", "rescan", "terminal", "theme", "lang", "prompt",
                "filters", "setup", "settings",
            ];
            let matched: Vec<&str> = cmds.into_iter().filter(|c| c.contains(&q)).collect();
            let chosen = matched.get(sel_idx).copied().unwrap_or("copy");
            match chosen {
                "copy" => self.copy_packed_context_to_os_clipboard(cx),
                "apply" => {
                    self.applier_state.update(cx, |a, cx| {
                        let _ = a.apply_changes_to_disk();
                        cx.notify();
                    });
                }
                "undo" => {
                    self.applier_state.update(cx, |a, cx| {
                        let _ = a.undo_last_apply();
                        cx.notify();
                    });
                }
                "rescan" => {
                    self.packer_state.update(cx, |p, cx| {
                        let _ = p.rescan_current_project();
                        cx.notify();
                    });
                }
                "terminal" => {
                    self.state.update(cx, |s, cx| {
                        s.open_or_toggle_dock_tab(DockToolTab::Terminal);
                        cx.notify();
                    });
                }
                "theme" => {
                    self.state.update(cx, |s, cx| {
                        s.toggle_theme();
                        cx.notify();
                    });
                }
                "lang" => {
                    self.state.update(cx, |s, cx| {
                        s.toggle_locale();
                        cx.notify();
                    });
                }
                "prompt" => {
                    self.packer_state.update(cx, |p, cx| {
                        p.set_prompt_modal_open(true);
                        cx.notify();
                    });
                }
                "filters" => {
                    self.packer_state.update(cx, |p, cx| {
                        p.set_regex_filter_open(true);
                        cx.notify();
                    });
                }
                "setup" => {
                    self.settings_state.update(cx, |ss, cx| {
                        ss.open_setup_wizard();
                        cx.notify();
                    });
                    self.state.update(cx, |s, cx| {
                        s.set_settings_open(true);
                        cx.notify();
                    });
                }
                _ => {
                    self.state.update(cx, |s, cx| {
                        s.set_settings_open(true);
                        cx.notify();
                    });
                }
            }
            return;
        }

        let target_path_opt = {
            let ps = self.packer_state.read(cx);
            let q_lower = trimmed.to_ascii_lowercase();
            ps.tree_projection()
                .visible_nodes()
                .iter()
                .filter(|n| {
                    !n.is_dir()
                        && (q_lower.is_empty() || n.path.to_ascii_lowercase().contains(&q_lower))
                })
                .nth(sel_idx)
                .map(|n| n.path.clone())
        };
        if let Some(rel_path) = target_path_opt {
            self.packer_state.update(cx, |p, cx| {
                let _ = p.jump_to_file_in_preview(&rel_path);
                cx.notify();
            });
        }
    }

    fn collect_command_palette_items(
        &self,
        trimmed: &str,
        labels: &'static crate::locale::UiLabels,
        cx: &Context<Self>,
    ) -> Vec<(String, String)> {
        let mut items: Vec<(String, String)> = Vec::with_capacity(10);
        if let Some(cmd_q) = trimmed.strip_prefix('>') {
            let q = cmd_q.trim().to_ascii_lowercase();
            let all_cmds = [
                ("> copy", labels.cmd_desc_copy),
                ("> apply", labels.cmd_desc_apply),
                ("> undo", labels.cmd_desc_undo),
                ("> rescan", labels.cmd_desc_rescan),
                ("> terminal", labels.cmd_desc_terminal),
                ("> prompt", labels.cmd_desc_prompt),
                ("> filters", labels.cmd_desc_filters),
                ("> theme", labels.cmd_desc_theme),
                ("> lang", labels.cmd_desc_lang),
                ("> setup", labels.cmd_desc_setup),
                ("> settings", labels.cmd_desc_settings),
            ];
            for (code, desc) in all_cmds {
                if q.is_empty() || code.contains(&q) || desc.to_lowercase().contains(&q) {
                    items.push((desc.to_owned(), code.to_owned()));
                }
            }
        } else {
            let q = trimmed.to_ascii_lowercase();
            let ps = self.packer_state.read(cx);
            for node in ps.tree_projection().visible_nodes() {
                if !node.is_dir() && (q.is_empty() || node.path.to_ascii_lowercase().contains(&q)) {
                    items.push((node.display_name.clone(), node.path.clone()));
                    if items.len() >= 10 {
                        break;
                    }
                }
            }
        }
        items
    }

    pub(crate) fn render_command_palette_modal(
        &self,
        locale: AppLocale,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = locale.labels();
        let cp = self.state.read(cx).command_palette().clone();
        let trimmed = cp.query.trim();
        let is_cmd_mode = trimmed.starts_with('>');
        let items = self.collect_command_palette_items(trimmed, labels, cx);

        let mut list = div().w_full().flex().flex_col().gap(px(2.0));
        for (idx, (title, subtitle)) in items.into_iter().enumerate() {
            let is_sel = idx == cp.selected_index;
            let view_entity = cx.entity().clone();
            let state_sel = self.state.clone();
            list = list.child(
                div()
                    .id(("cmd-palette-item", idx))
                    .w_full()
                    .h(px(28.0))
                    .px(px(theme.spacing.md))
                    .rounded(px(theme.radius.sm))
                    .bg(if is_sel { colors.bg_elevated } else { colors.bg_panel })
                    .border_1()
                    .border_color(if is_sel { colors.cyan } else { colors.bg_panel })
                    .cursor_pointer()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .on_click(move |_ev, _win, cx| {
                        state_sel.update(cx, |s, _cx| {
                            s.command_palette_mut().selected_index = idx;
                        });
                        view_entity.update(cx, |this, cx| {
                            this.execute_selected_command_palette_item(cx);
                        });
                    })
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_primary)
                            .child(SharedString::from(title)),
                    )
                    .child(
                        div()
                            .font_family(FONT_FAMILY_MONO)
                            .text_xs()
                            .text_color(colors.text_muted)
                            .child(SharedString::from(subtitle)),
                    ),
            );
        }

        let state_close = self.state.clone();
        let placeholder = if is_cmd_mode {
            labels.cmd_palette_placeholder_cmd
        } else {
            labels.cmd_palette_placeholder_file
        };

        ModalDialogShell::new(
            "cmd-palette-backdrop",
            "cmd-palette-dialog",
            "cmd-palette-close-btn",
            labels.cmd_palette_title,
        )
        .width(520.0)
        .leading_icon(UiIcon::Search)
        .with_theme(theme)
        .on_close(move |_ev, _win, cx| {
            state_close.update(cx, |s, cx| {
                s.close_command_palette();
                cx.notify();
            });
        })
        .child(
            crate::shared::containers::form_controls::TextInputField::new(
                "cmd-palette-input",
                cp.query,
                placeholder,
                true,
            )
            .leading_icon(UiIcon::Search)
            .with_theme(theme),
        )
        .child(list)
    }
}
