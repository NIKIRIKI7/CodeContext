use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use core::system::ports::hotkey::GlobalHotkeyPort as _;
use core::system::ports::ipc::{IpcCommand, SingleInstanceIpcPort as _};
use core::system::ports::updater::{UpdateChannel, UpdateStage, UpdaterPort as _};
use di::AppContainer;
use gpui::{
    App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _,
    PathPromptOptions, Render, SharedString, Styled as _, Window, div, px,
};

use crate::features::change_applier::state::ChangeApplierState;
use crate::features::change_applier::view::ChangeApplierView;
use crate::features::context_packer::state::ContextPackerState;
use crate::features::context_packer::view::ContextPackerView;
use crate::features::main_window::components::diagnostics_dock::TerminalCommandEntry;
use crate::features::main_window::state::{MainWindowState, WorkspaceTab};
use crate::features::settings_modal::state::SettingsModalState;
use crate::features::settings_modal::view::SettingsModalView;
use crate::locale::AppLocale;
use crate::theme::{FONT_FAMILY_UI, ThemeVariant};

#[derive(Debug, Clone)]
pub struct MainWindowView {
    pub(crate) container: AppContainer,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) state: Entity<MainWindowState>,
    pub(crate) packer_state: Entity<ContextPackerState>,
    pub(crate) applier_state: Entity<ChangeApplierState>,
    pub(crate) settings_state: Entity<SettingsModalState>,
    pub(crate) packer_view: Entity<ContextPackerView>,
    pub(crate) applier_view: Entity<ChangeApplierView>,
    pub(crate) settings_view: Entity<SettingsModalView>,
}

impl MainWindowView {
    pub fn new(
        container: AppContainer,
        initial_project_root: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let state = cx.new(|_cx| MainWindowState::new());
        let initial_locale = AppLocale::default();
        let initial_theme = ThemeVariant::default();

        let container_for_packer = container.clone();
        let root_for_packer = initial_project_root.clone();
        let packer_state =
            cx.new(|_cx| ContextPackerState::new(container_for_packer, root_for_packer));

        let container_for_applier = container.clone();
        let applier_state =
            cx.new(|_cx| ChangeApplierState::new(container_for_applier, initial_project_root));

        let settings_state = cx.new(|_cx| SettingsModalState::new(container.clone()));

        let ps_clone = packer_state.clone();
        let packer_view =
            cx.new(|cx| ContextPackerView::new(ps_clone, initial_locale, initial_theme, cx));

        let as_clone = applier_state.clone();
        let applier_view =
            cx.new(|cx| ChangeApplierView::new(as_clone, initial_locale, initial_theme, cx));

        let ss_clone = settings_state.clone();
        let settings_view =
            cx.new(|cx| SettingsModalView::new(ss_clone, initial_locale, initial_theme, cx));

        let view = Self {
            container,
            focus_handle,
            state,
            packer_state,
            applier_state,
            settings_state,
            packer_view,
            applier_view,
            settings_view,
        };
        view.setup_observers(cx);
        view.spawn_startup_update_check(cx);
        view.spawn_background_ipc_and_hotkey_poller(cx);
        view
    }

    fn setup_observers(&self, cx: &mut Context<Self>) {
        cx.observe(&self.packer_state, |this, model, cx| {
            let (meta_req, packer_root) = model.update(cx, |s, _cx| {
                (s.take_meta_modal_request(), s.project_root().map(Path::to_path_buf))
            });
            let applier_root = this.applier_state.read(cx).project_root().map(Path::to_path_buf);
            if packer_root != applier_root {
                this.applier_state.update(cx, |a, cx| {
                    a.set_project_root(packer_root);
                    cx.notify();
                });
            }
            if meta_req {
                this.state.update(cx, |ms, cx| {
                    ms.set_project_meta_open(true);
                    cx.notify();
                });
            }
            cx.notify();
        })
        .detach();

        cx.observe(&self.applier_state, |_this, _model, cx| {
            cx.notify();
        })
        .detach();

        cx.observe(&self.settings_state, |this, model, cx| {
            let (create_backups, new_locale, new_theme, close_req) = model.update(cx, |m, _cx| {
                (
                    m.global_config().create_backups_by_default,
                    m.locale(),
                    m.theme_variant(),
                    m.take_close_request(),
                )
            });
            this.applier_state.update(cx, |applier, cx| {
                applier.set_backups_enabled(create_backups);
                cx.notify();
            });
            this.state.update(cx, |ms, cx| {
                ms.set_locale(new_locale);
                ms.set_theme_variant(new_theme);
                if close_req {
                    ms.set_settings_open(false);
                }
                cx.notify();
            });
            cx.notify();
        })
        .detach();

        cx.observe(&self.state, |this, model, cx| {
            let snap = model.read(cx);
            let locale = snap.locale();
            let theme_variant = snap.theme_variant();
            this.packer_view.update(cx, |v, cx| {
                v.sync_preferences(locale, theme_variant, cx);
            });
            this.applier_view.update(cx, |v, cx| {
                v.sync_preferences(locale, theme_variant, cx);
            });
            this.settings_view.update(cx, |v, cx| {
                v.sync_preferences(locale, theme_variant, cx);
            });
            cx.notify();
        })
        .detach();
    }

    fn spawn_startup_update_check(&self, cx: &mut Context<Self>) {
        let container_startup_upd = self.container.clone();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(1200)).await;
            let cfg = container_startup_upd.preset_service().load_global_config();
            let ch = UpdateChannel::parse(&cfg.update_channel);
            let repo = cfg.update_repo_url.clone();
            let c_bg = container_startup_upd.clone();
            let stage_res = cx
                .background_executor()
                .spawn(async move { c_bg.updater().check_for_updates(ch, &repo) })
                .await;
            if let Ok(UpdateStage::UpdateAvailable { version, channel }) = stage_res {
                let _ = this.update(cx, |view, cx| {
                    view.settings_state.update(cx, |s, cx| {
                        s.sync_from_container();
                        cx.notify();
                    });
                    view.state.update(cx, |ms, cx| {
                        ms.notify_update_available(version, channel);
                        cx.notify();
                    });
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn spawn_background_ipc_and_hotkey_poller(&self, cx: &mut Context<Self>) {
        let container_poll = self.container.clone();
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(50)).await;
                let has_hk = container_poll.hotkey().try_recv_hotkey_trigger();
                let ipc_opt = container_poll.ipc().try_recv_command();
                if !has_hk && ipc_opt.is_none() {
                    continue;
                }
                let Ok(()) = this.update(cx, |view, cx| {
                    if let Some(IpcCommand::OpenProjectAndFocus { project_root }) = ipc_opt {
                        let root_applier = project_root.clone();
                        view.packer_state.update(cx, |p, cx| {
                            let _ = p.open_project(project_root);
                            cx.notify();
                        });
                        view.applier_state.update(cx, |a, cx| {
                            a.set_project_root(Some(root_applier));
                            cx.notify();
                        });
                    }
                    if has_hk && !view.state.read(cx).is_settings_open() {
                        let active_tab = view.state.read(cx).active_tab();
                        let auto_clip = view
                            .settings_state
                            .read(cx)
                            .global_config()
                            .auto_read_clipboard_on_apply;
                        if active_tab == WorkspaceTab::ContextPacker {
                            view.copy_packed_context_to_os_clipboard(cx);
                        } else if auto_clip {
                            view.applier_state.update(cx, |a, cx| {
                                let _ = a.load_response_from_clipboard_if_changed();
                                cx.notify();
                            });
                        }
                    }
                    cx.notify();
                }) else {
                    break;
                };
            }
        })
        .detach();
    }

    pub(crate) fn prompt_and_open_project_folder(
        packer_state: Entity<ContextPackerState>,
        applier_state: Entity<ChangeApplierState>,
        prompt_label: &'static str,
        cx: &mut App,
    ) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(SharedString::from(prompt_label)),
        });

        cx.spawn(async move |cx| {
            if let Ok(Ok(Some(paths))) = rx.await
                && let Some(chosen_dir) = paths.into_iter().next()
            {
                let applier_dir = chosen_dir.clone();
                let _ = packer_state.update(cx, |s, cx| {
                    let _ = s.open_project(chosen_dir);
                    cx.notify();
                });
                let _ = applier_state.update(cx, |a, cx| {
                    a.set_project_root(Some(applier_dir));
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub(crate) fn spawn_terminal_command(&self, raw_cmd: &str, cx: &mut App) {
        let cmd_str = raw_cmd.trim().to_owned();
        if cmd_str.is_empty() {
            return;
        }
        let work_dir = self
            .packer_state
            .read(cx)
            .project_root()
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        self.state.update(cx, |s, cx| {
            s.bottom_dock_mut().terminal_running = true;
            cx.notify();
        });

        let state_after = self.state.clone();
        let container_after = self.container.clone();
        cx.spawn(async move |cx| {
            let cmd_bg = cmd_str.clone();
            let (output_str, exit_code, elapsed) = cx
                .background_executor()
                .spawn(async move { run_shell_command_captured(&cmd_bg, &work_dir) })
                .await;

            let duration_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
            if exit_code == 0 {
                container_after.telemetry().record_timed(
                    "terminal",
                    "exec_cmd",
                    elapsed,
                    format!("$ {cmd_str} (exit 0)"),
                );
            } else {
                container_after.telemetry().record_warn(
                    "terminal",
                    "exec_cmd",
                    Some(elapsed),
                    format!("$ {cmd_str} (exit {exit_code})"),
                );
            }

            let _ = state_after.update(cx, |s, cx| {
                let dock = s.bottom_dock_mut();
                dock.terminal_running = false;
                dock.terminal_history.push(TerminalCommandEntry {
                    command: cmd_str,
                    output: output_str,
                    exit_code,
                    duration_ms,
                });
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn scroll_preview_to_line(&self, target_line: usize, cx: &mut Context<Self>) {
        self.packer_state.update(cx, |p, cx| {
            let total_lines = p.preview_buffer().line_count();
            let clamped = target_line.min(total_lines.saturating_sub(1));
            let _ = p.take_pending_preview_scroll();
            cx.notify();
            clamped
        });
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.state.read(cx).bottom_dock().resizing_dock.is_some() {
            let curr_y = event.position.y / px(1.0);
            let viewport_h = window.viewport_size().height / px(1.0);
            let max_h = (viewport_h - 140.0).max(140.0);
            self.state.update(cx, |s, cx| {
                s.update_dock_resize(curr_y, max_h);
                cx.notify();
            });
            return;
        }

        if self.packer_state.read(cx).is_resizing_split() {
            let rail_w = 48.0_f32;
            let total_w = ((window.viewport_size().width / px(1.0)) - rail_w).max(400.0);
            let rel_x = ((event.position.x / px(1.0)) - rail_w).max(0.0);
            let ratio = rel_x / total_w;
            self.packer_state.update(cx, |p, cx| {
                p.set_split_ratio(ratio);
                cx.notify();
            });
            return;
        }

        if self.applier_state.read(cx).is_resizing_split() {
            let rail_w = 48.0_f32;
            let total_w = ((window.viewport_size().width / px(1.0)) - rail_w).max(400.0);
            let rel_x = ((event.position.x / px(1.0)) - rail_w).max(0.0);
            let ratio = rel_x / total_w;
            self.applier_state.update(cx, |a, cx| {
                a.set_split_ratio(ratio);
                cx.notify();
            });
        }
    }

    fn handle_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.state.read(cx).bottom_dock().resizing_dock.is_some() {
            self.state.update(cx, |s, cx| {
                s.stop_dock_resize();
                cx.notify();
            });
        }
        if self.packer_state.read(cx).is_resizing_split() {
            self.packer_state.update(cx, |p, cx| {
                p.set_resizing_split(false);
                cx.notify();
            });
        }
        if self.applier_state.read(cx).is_resizing_split() {
            self.applier_state.update(cx, |a, cx| {
                a.set_resizing_split(false);
                cx.notify();
            });
        }
    }

    pub(crate) fn handle_escape_key(&self, cx: &mut Context<Self>) {
        let (settings_open, meta_open, recent_open) = {
            let snap = self.state.read(cx);
            (snap.is_settings_open(), snap.is_project_meta_open(), snap.is_recent_projects_open())
        };
        if settings_open || meta_open || recent_open {
            self.state.update(cx, |s, cx| {
                s.set_settings_open(false);
                s.set_project_meta_open(false);
                s.set_recent_projects_open(false);
                cx.notify();
            });
            return;
        }
        self.packer_state.update(cx, |p, cx| {
            p.clear_search_query();
            p.set_search_focused(false);
            cx.notify();
        });
    }

    pub(crate) fn copy_packed_context_to_os_clipboard(&self, cx: &mut Context<Self>) {
        let mut copied_text_opt: Option<String> = None;
        self.packer_state.update(cx, |s, cx| {
            let _ = s.copy_active_chunk_to_clipboard();
            copied_text_opt = Some(s.preview_buffer().as_str().to_owned());
            cx.notify();
        });
        if let Some(text) = copied_text_opt {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    pub(crate) fn paste_clipboard_into_applier(&self, cx: &mut Context<Self>) {
        let clip_str =
            cx.read_from_clipboard().and_then(|item| item.text()).filter(|t| !t.is_empty());
        self.state.update(cx, |s, cx| {
            s.set_active_tab(WorkspaceTab::ChangeApplier);
            cx.notify();
        });
        self.applier_state.update(cx, |a, cx| {
            if let Some(text) = clip_str {
                a.set_response_text(text);
            } else {
                a.load_response_from_clipboard();
            }
            cx.notify();
        });
    }
}

fn run_shell_command_captured(cmd_str: &str, work_dir: &Path) -> (String, i32, Duration) {
    let t0 = Instant::now();
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("powershell.exe");
        c.args(["-NoProfile", "-NonInteractive", "-Command", cmd_str]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            c.creation_flags(CREATE_NO_WINDOW);
        }
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd_str]);
        c
    };
    cmd.current_dir(work_dir);

    match cmd.output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let mut combined = String::with_capacity(stdout.len() + stderr.len());
            if !stdout.trim().is_empty() {
                combined.push_str(stdout.trim_end());
            }
            if !stderr.trim().is_empty() {
                if !combined.is_empty() {
                    combined.push('\n');
                }
                combined.push_str(stderr.trim_end());
            }
            if combined.len() > 16_384 {
                combined.truncate(16_384);
                combined.push_str("\n… [truncated]");
            }
            (combined, code, t0.elapsed())
        }
        Err(err) => (err.to_string(), -1, t0.elapsed()),
    }
}

impl Render for MainWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
        }

        let snap = self.state.read(cx);
        let active_tab = snap.active_tab();
        let settings_open = snap.is_settings_open();
        let logs_dock_open = snap.is_logs_dock_open();
        let meta_open = snap.is_project_meta_open();
        let recent_open = snap.is_recent_projects_open();
        let update_modal_open = snap.is_update_modal_open();
        let available_update = snap.available_update();
        let locale = snap.locale();
        let theme_variant = snap.theme_variant();
        let theme = theme_variant.tokens();
        let labels = locale.labels();

        let chrome = self.settings_state.read(cx).global_config().ui_chrome.clone();
        let telemetry_summary = self.container.telemetry().summary();

        let packer_snap = self.packer_state.read(cx);
        let project_display = packer_snap
            .project_root()
            .map_or_else(|| labels.no_project_selected.to_owned(), |p| p.display().to_string());

        let rail = self.build_activity_rail(active_tab, settings_open, labels, theme);
        let top_bar = self.render_top_bar(active_tab, locale, theme, &project_display);

        let active_content = match active_tab {
            WorkspaceTab::ContextPacker => self.packer_view.clone().into_any_element(),
            WorkspaceTab::ChangeApplier => self.applier_view.clone().into_any_element(),
        };

        let mut workspace_col = div()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(top_bar)
            .child(div().flex_1().min_h(px(0.0)).overflow_hidden().child(active_content));

        if logs_dock_open {
            workspace_col = workspace_col.child(self.build_diagnostics_dock(
                telemetry_summary,
                &chrome,
                labels,
                theme,
                cx,
            ));
        }

        if chrome.show_bottom_bar && (chrome.show_terminal_button || chrome.show_logs_button) {
            workspace_col = workspace_col.child(self.render_bottom_status_bar(
                telemetry_summary,
                &chrome,
                labels,
                theme,
                cx,
            ));
        }

        let packer_unfocus_search = self.packer_state.clone();
        let state_unfocus_term = self.state.clone();
        let mut root = div()
            .id("main-window-root")
            .track_focus(&self.focus_handle)
            .on_mouse_down(MouseButton::Left, move |_ev: &MouseDownEvent, _win, cx: &mut App| {
                packer_unfocus_search.update(cx, |p, cx| {
                    if p.ui_status().search_focused {
                        p.set_search_focused(false);
                        cx.notify();
                    }
                });
                state_unfocus_term.update(cx, |s, cx| {
                    if s.bottom_dock().terminal_focused {
                        s.bottom_dock_mut().terminal_focused = false;
                        cx.notify();
                    }
                });
            })
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_key_down(cx.listener(Self::handle_key_down))
            .relative()
            .size_full()
            .flex()
            .flex_row()
            .bg(theme.colors.bg_app)
            .font_family(FONT_FAMILY_UI)
            .overflow_hidden()
            .child(rail)
            .child(workspace_col);

        if recent_open {
            root = root.child(self.build_recent_projects_popover(locale, theme, cx));
        }

        if meta_open {
            root = root.child(self.build_meta_modal_overlay(locale, theme, project_display, cx));
        }

        if settings_open {
            root = root.child(self.settings_view.clone());
        }

        if update_modal_open && let Some((new_ver, ch)) = available_update {
            root = root.child(self.render_update_available_modal(new_ver, ch, locale, theme));
        }

        root
    }
}
