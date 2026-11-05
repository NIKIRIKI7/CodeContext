use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use core::packing::aggregates::project::Project;
use core::preset::models::resolution::{
    TerminalCommandPreset, TerminalTaskCategory, UiChromeConfig,
};
use core::system::ports::hotkey::GlobalHotkeyPort as _;
use core::system::ports::ipc::{IpcCommand, SingleInstanceIpcPort as _};
use core::system::ports::updater::{SemanticVersion, UpdateChannel, UpdateStage, UpdaterPort as _};
use di::AppContainer;
use gpui::{
    App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, KeyDownEvent, ParentElement as _, PathPromptOptions, Render, Rgba, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::features::change_applier::state::ChangeApplierState;
use crate::features::change_applier::view::ChangeApplierView;
use crate::features::context_packer::state::ContextPackerState;
use crate::features::context_packer::view::ContextPackerView;
use crate::features::main_window::components::activity_rail::ActivityRailComponent;
use crate::features::main_window::components::diagnostics_dock::{
    DiagnosticsDockComponent, DiagnosticsDockProps, DockToolTab, TerminalCommandEntry,
};
use crate::features::main_window::components::project_meta_modal::{
    ProjectMetaModalComponent, ProjectMetaProps,
};
use crate::features::main_window::components::recent_projects_popover::RecentProjectsPopover;
use crate::features::main_window::state::{MainWindowState, WorkspaceTab};
use crate::features::settings_modal::state::SettingsModalState;
use crate::features::settings_modal::view::SettingsModalView;

use crate::locale::AppLocale;
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{
    FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, ThemeVariant, format_project_path_smart,
};

#[derive(Debug, Clone)]
pub struct MainWindowView {
    container: AppContainer,
    focus_handle: FocusHandle,
    state: Entity<MainWindowState>,
    packer_state: Entity<ContextPackerState>,
    applier_state: Entity<ChangeApplierState>,
    settings_state: Entity<SettingsModalState>,
    packer_view: Entity<ContextPackerView>,
    applier_view: Entity<ChangeApplierView>,
    settings_view: Entity<SettingsModalView>,
}

impl MainWindowView {
    #[allow(clippy::too_many_lines)]
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

        cx.observe(&packer_state, |this, model, cx| {
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

        cx.observe(&applier_state, |_this, _model, cx| {
            cx.notify();
        })
        .detach();

        cx.observe(&settings_state, |this, model, cx| {
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

        cx.observe(&state, |this, model, cx| {
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

        let container_startup_upd = container.clone();
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

        let container_poll = container.clone();
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

        Self {
            container,
            focus_handle,
            state,
            packer_state,
            applier_state,
            settings_state,
            packer_view,
            applier_view,
            settings_view,
        }
    }

    fn prompt_and_open_project_folder(
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

    #[allow(clippy::too_many_lines)]
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mods = event.keystroke.modifiers;
        let primary_mod =
            if cfg!(target_os = "macos") { mods.platform || mods.control } else { mods.control };
        let key = event.keystroke.key.as_str();

        if self.state.read(cx).is_settings_open() {
            if let Some(action) = self.settings_state.read(cx).recording_action() {
                if key.eq_ignore_ascii_case("escape") {
                    self.settings_state.update(cx, |s, cx| {
                        s.cancel_recording_hotkey();
                        cx.notify();
                    });
                    return;
                }
                if let Some(shortcut_str) = build_recorded_hotkey_string(event) {
                    self.settings_state.update(cx, |s, cx| {
                        let _ = s.rebind_action_shortcut(action, &shortcut_str);
                        cx.notify();
                    });
                }
                return;
            }
            if key.eq_ignore_ascii_case("escape")
                && self.settings_state.read(cx).is_hotkeys_modal_open()
            {
                self.settings_state.update(cx, |s, cx| {
                    s.close_hotkeys_modal();
                    cx.notify();
                });
                return;
            }
        }

        if let Some(pressed_shortcut) = build_recorded_hotkey_string(event)
            && self.try_dispatch_custom_keymap(&pressed_shortcut, cx)
        {
            return;
        }

        if self.try_handle_terminal_input_key(event, key, primary_mod, cx) {
            return;
        }

        if self.try_handle_packer_modals_key(event, key, primary_mod, cx) {
            return;
        }

        let is_search_focused = self.packer_state.read(cx).ui_status().search_focused;
        if is_search_focused {
            if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    p.set_search_focused(false);
                    cx.notify();
                });
                return;
            }
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    if p.pop_search_char() {
                        cx.notify();
                    }
                });
                return;
            }
            if !primary_mod && !mods.alt && !key.eq_ignore_ascii_case("f5") {
                if let Some(ch) = extract_printable_search_char(event) {
                    self.packer_state.update(cx, |p, cx| {
                        p.append_search_char(ch);
                        cx.notify();
                    });
                }
                return;
            }
            self.packer_state.update(cx, |p, cx| {
                p.set_search_focused(false);
                cx.notify();
            });
        }

        if key.eq_ignore_ascii_case("escape") {
            self.handle_escape_key(cx);
            return;
        }

        if key.eq_ignore_ascii_case("f12")
            || (primary_mod && matches!(key, "`" | "~" | "ё" | "Ё" | "backquote"))
        {
            self.state.update(cx, |s, cx| {
                s.toggle_logs_dock();
                cx.notify();
            });
            return;
        }

        if key.eq_ignore_ascii_case("f5") || (primary_mod && !mods.shift && is_key_r(key)) {
            self.packer_state.update(cx, |s, cx| {
                let _ = s.rescan_current_project();
                cx.notify();
            });
            return;
        }

        if primary_mod && !mods.shift && is_key_f(key) {
            self.state.update(cx, |s, cx| {
                s.set_active_tab(WorkspaceTab::ContextPacker);
                cx.notify();
            });
            self.packer_state.update(cx, |p, cx| {
                p.set_search_focused(true);
                cx.notify();
            });
            return;
        }

        if !primary_mod && !mods.alt && self.handle_tree_navigation_key(key, mods.shift, cx) {
            return;
        }

        if !primary_mod {
            return;
        }

        if !mods.shift && is_key_k(key) {
            self.copy_packed_context_to_os_clipboard(cx);
            return;
        }

        if !mods.shift && is_key_o(key) {
            let prompt_label = self.state.read(cx).locale().labels().btn_open_folder_prompt;
            Self::prompt_and_open_project_folder(
                self.packer_state.clone(),
                self.applier_state.clone(),
                prompt_label,
                cx,
            );
            return;
        }

        if mods.shift && (is_key_c(key) || is_key_s(key)) {
            self.copy_packed_context_to_os_clipboard(cx);
            return;
        }

        if !mods.shift && is_key_v(key) {
            self.paste_clipboard_into_applier(cx);
            return;
        }

        let active_tab = self.state.read(cx).active_tab();
        if active_tab == WorkspaceTab::ContextPacker && !mods.shift && is_key_c(key) {
            self.copy_packed_context_to_os_clipboard(cx);
            return;
        }

        if active_tab == WorkspaceTab::ChangeApplier {
            if key.eq_ignore_ascii_case("enter") {
                self.applier_state.update(cx, |s, cx| {
                    let _ = s.apply_changes_to_disk();
                    cx.notify();
                });
            } else if !mods.shift && is_key_z(key) {
                self.applier_state.update(cx, |s, cx| {
                    let _ = s.undo_last_apply();
                    cx.notify();
                });
            }
        }
    }

    fn try_handle_terminal_input_key(
        &mut self,
        event: &KeyDownEvent,
        key: &str,
        primary_mod: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let is_term_active = {
            let s = self.state.read(cx);
            s.is_logs_dock_open() && s.dock_active_tab() == DockToolTab::Terminal
        };
        if !is_term_active {
            return false;
        }
        if key.eq_ignore_ascii_case("escape") {
            self.state.update(cx, |s, cx| {
                s.set_logs_dock_open(false);
                cx.notify();
            });
            return true;
        }
        if key.eq_ignore_ascii_case("enter") {
            let cmd = self.state.update(cx, |s, cx| {
                let d = s.bottom_dock_mut();
                let taken = std::mem::take(&mut d.terminal_input);
                cx.notify();
                taken
            });
            self.spawn_terminal_command(&cmd, cx);
            return true;
        }
        if key.eq_ignore_ascii_case("backspace") {
            self.state.update(cx, |s, cx| {
                if s.bottom_dock_mut().terminal_input.pop().is_some() {
                    cx.notify();
                }
            });
            return true;
        }
        let mods = event.keystroke.modifiers;
        if !primary_mod && !mods.alt {
            if key == " " || key.eq_ignore_ascii_case("space") {
                self.state.update(cx, |s, cx| {
                    s.bottom_dock_mut().terminal_input.push(' ');
                    s.bottom_dock_mut().terminal_focused = true;
                    cx.notify();
                });
                return true;
            }
            if let Some(ch) = extract_printable_regex_char(event) {
                self.state.update(cx, |s, cx| {
                    s.bottom_dock_mut().terminal_input.push(ch);
                    s.bottom_dock_mut().terminal_focused = true;
                    cx.notify();
                });
                return true;
            }
        }
        true
    }

    fn spawn_terminal_command(&self, raw_cmd: &str, cx: &mut App) {
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

    #[allow(clippy::too_many_lines)]
    fn try_handle_packer_modals_key(
        &self,
        event: &KeyDownEvent,
        key: &str,
        primary_mod: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let (
            is_filter_open,
            is_folder_focused,
            is_ext_focused,
            is_regex_focused,
            is_regex_name_focused,
            is_prompt_open,
            is_prompt_focused,
        ) = {
            let ps = self.packer_state.read(cx);
            (
                ps.ui_status().regex_filter_open,
                ps.is_folder_input_focused(),
                ps.is_extension_input_focused(),
                ps.is_regex_input_focused(),
                ps.is_regex_name_input_focused(),
                ps.is_prompt_modal_open(),
                ps.is_prompt_input_focused(),
            )
        };
        if is_prompt_open {
            if key.eq_ignore_ascii_case("escape") {
                self.packer_state.update(cx, |p, cx| {
                    p.set_prompt_modal_open(false);
                    cx.notify();
                });
                return true;
            }
            if is_prompt_focused {
                if key.eq_ignore_ascii_case("enter") {
                    self.packer_state.update(cx, |p, cx| {
                        p.submit_manual_prompt_draft();
                        cx.notify();
                    });
                    return true;
                }
                if key.eq_ignore_ascii_case("backspace") {
                    self.packer_state.update(cx, |p, cx| {
                        if p.pop_prompt_input_char() {
                            cx.notify();
                        }
                    });
                    return true;
                }
                let mods = event.keystroke.modifiers;
                if !primary_mod
                    && !mods.alt
                    && let Some(ch) = extract_printable_search_char(event)
                {
                    self.packer_state.update(cx, |p, cx| {
                        p.append_prompt_input_char(ch);
                        cx.notify();
                    });
                    return true;
                }
            }
        }
        if !is_filter_open {
            return false;
        }
        if key.eq_ignore_ascii_case("escape") {
            self.packer_state.update(cx, |p, cx| {
                p.set_regex_filter_open(false);
                cx.notify();
            });
            return true;
        }
        let mods = event.keystroke.modifiers;
        if is_ext_focused {
            if key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    p.submit_manual_extension();
                    cx.notify();
                });
                return true;
            }
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    if p.pop_extension_input_char() {
                        cx.notify();
                    }
                });
                return true;
            }
            if !primary_mod
                && !mods.alt
                && let Some(ch) = extract_printable_regex_char(event)
            {
                self.packer_state.update(cx, |p, cx| {
                    p.append_extension_input_char(ch);
                    cx.notify();
                });
                return true;
            }
        }
        if is_folder_focused {
            if key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    p.submit_manual_folder_exclusion();
                    cx.notify();
                });
                return true;
            }
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    if p.pop_folder_input_char() {
                        cx.notify();
                    }
                });
                return true;
            }
            if !primary_mod
                && !mods.alt
                && let Some(ch) = extract_printable_regex_char(event)
            {
                self.packer_state.update(cx, |p, cx| {
                    p.append_folder_input_char(ch);
                    cx.notify();
                });
                return true;
            }
        }
        if is_regex_name_focused {
            if key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    let _ = p.submit_manual_regex_template();
                    cx.notify();
                });
                return true;
            }
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    if p.pop_regex_name_char() {
                        cx.notify();
                    }
                });
                return true;
            }
            if !primary_mod
                && !mods.alt
                && let Some(ch) = extract_printable_regex_char(event)
            {
                self.packer_state.update(cx, |p, cx| {
                    p.append_regex_name_char(ch);
                    cx.notify();
                });
                return true;
            }
        }
        if is_regex_focused {
            if key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    p.set_regex_input_focused(false);
                    cx.notify();
                });
                return true;
            }
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    if p.pop_regex_char() {
                        cx.notify();
                    }
                });
                return true;
            }
            if !primary_mod
                && !mods.alt
                && let Some(ch) = extract_printable_regex_char(event)
            {
                self.packer_state.update(cx, |p, cx| {
                    p.append_regex_char(ch);
                    cx.notify();
                });
                return true;
            }
        }
        false
    }

    fn try_dispatch_custom_keymap(&self, pressed: &str, cx: &mut Context<Self>) -> bool {
        if self.state.read(cx).is_settings_open()
            || self.packer_state.read(cx).ui_status().search_focused
            || self.state.read(cx).bottom_dock().terminal_focused
        {
            return false;
        }
        let km = self.settings_state.read(cx).global_config().keymap.clone();
        let active_tab = self.state.read(cx).active_tab();

        if pressed.eq_ignore_ascii_case(&km.toggle_terminal) {
            self.state.update(cx, |s, cx| {
                s.toggle_logs_dock();
                cx.notify();
            });
            return true;
        }

        if pressed.eq_ignore_ascii_case(&km.open_project) {
            let prompt_label = self.state.read(cx).locale().labels().btn_open_folder_prompt;
            Self::prompt_and_open_project_folder(
                self.packer_state.clone(),
                self.applier_state.clone(),
                prompt_label,
                cx,
            );
            return true;
        }
        if pressed.eq_ignore_ascii_case(&km.search_files) {
            self.state.update(cx, |s, cx| {
                s.set_active_tab(WorkspaceTab::ContextPacker);
                cx.notify();
            });
            self.packer_state.update(cx, |p, cx| {
                p.set_search_focused(true);
                cx.notify();
            });
            return true;
        }
        if pressed.eq_ignore_ascii_case(&km.rescan_project) {
            self.packer_state.update(cx, |s, cx| {
                let _ = s.rescan_current_project();
                cx.notify();
            });
            return true;
        }
        if pressed.eq_ignore_ascii_case(&km.paste_response) {
            self.paste_clipboard_into_applier(cx);
            return true;
        }
        if active_tab == WorkspaceTab::ContextPacker
            && pressed.eq_ignore_ascii_case(&km.copy_context)
        {
            self.copy_packed_context_to_os_clipboard(cx);
            return true;
        }
        if active_tab == WorkspaceTab::ChangeApplier {
            if pressed.eq_ignore_ascii_case(&km.apply_changes) {
                self.applier_state.update(cx, |s, cx| {
                    let _ = s.apply_changes_to_disk();
                    cx.notify();
                });
                return true;
            }
            if pressed.eq_ignore_ascii_case(&km.undo_changes) {
                self.applier_state.update(cx, |s, cx| {
                    let _ = s.undo_last_apply();
                    cx.notify();
                });
                return true;
            }
        }
        false
    }

    fn handle_tree_navigation_key(&self, key: &str, shift: bool, cx: &mut Context<Self>) -> bool {
        let snap = self.state.read(cx);
        if snap.active_tab() != WorkspaceTab::ContextPacker
            || snap.is_settings_open()
            || snap.is_project_meta_open()
            || snap.is_recent_projects_open()
            || snap.is_logs_dock_open()
        {
            return false;
        }

        if key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("arrowdown") {
            self.packer_state.update(cx, |p, cx| {
                if p.move_tree_focus_down() {
                    cx.notify();
                }
            });
            return true;
        }
        if key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("arrowup") {
            self.packer_state.update(cx, |p, cx| {
                if p.move_tree_focus_up() {
                    cx.notify();
                }
            });
            return true;
        }
        if key == " " || key.eq_ignore_ascii_case("space") {
            // Клавиша Space больше не сбрасывает выделение с файлов или директорий кодовой базы.
            // Вместо деструктивного переключения чекбоксов она безопасно прокручивает
            // страницу предпросмотра вниз (Page Down), а Shift+Space — вверх:
            let delta: usize = 30;
            let cur_line = self.packer_state.read(cx).take_pending_preview_scroll().unwrap_or(0);
            let next_line =
                if shift { cur_line.saturating_sub(delta) } else { cur_line.saturating_add(delta) };
            self.packer_view.update(cx, |_v, cx| {
                cx.notify();
            });
            self.packer_state.update(cx, |_p, cx| {
                cx.notify();
            });
            self.scroll_preview_to_line(next_line, cx);
            return true;
        }
        if key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright") {
            self.packer_state.update(cx, |p, cx| {
                if p.expand_or_step_into_focused_tree_node() {
                    cx.notify();
                }
            });
            return true;
        }
        if key.eq_ignore_ascii_case("left") || key.eq_ignore_ascii_case("arrowleft") {
            self.packer_state.update(cx, |p, cx| {
                if p.collapse_or_step_out_focused_tree_node() {
                    cx.notify();
                }
            });
            return true;
        }
        if key.eq_ignore_ascii_case("enter") {
            self.packer_state.update(cx, |p, cx| {
                if p.activate_focused_tree_node() {
                    cx.notify();
                }
            });
            return true;
        }
        false
    }

    fn scroll_preview_to_line(&self, target_line: usize, cx: &mut Context<Self>) {
        self.packer_state.update(cx, |p, cx| {
            let total_lines = p.preview_buffer().line_count();
            let clamped = target_line.min(total_lines.saturating_sub(1));
            let _ = p.take_pending_preview_scroll();
            if let Some(first_file) = p.tree_projection().visible_nodes().first() {
                let _ = &first_file.path;
            }
            cx.notify();
            clamped
        });
    }

    fn handle_escape_key(&self, cx: &mut Context<Self>) {
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

    fn copy_packed_context_to_os_clipboard(&self, cx: &mut Context<Self>) {
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

    fn paste_clipboard_into_applier(&self, cx: &mut Context<Self>) {
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

    fn render_top_bar(
        &self,
        active_tab: WorkspaceTab,
        locale: AppLocale,
        theme_variant: ThemeVariant,
        project_display: &str,
    ) -> impl IntoElement + use<> {
        let theme = theme_variant.tokens();
        let labels = locale.labels();
        let compact_project = format_project_path_smart(project_display, 48);
        let left_controls = self.render_project_controls(&compact_project, labels, theme);
        let mode_title = match active_tab {
            WorkspaceTab::ContextPacker => labels.tab_context_packer,
            WorkspaceTab::ChangeApplier => labels.tab_change_applier,
        };

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
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(theme.colors.text_muted)
                    .child(mode_title),
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

    fn build_recent_projects_popover(
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

    fn build_meta_modal_overlay(
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

    fn render_update_available_modal(
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
        let backdrop = Rgba { r: 0.02, g: 0.03, b: 0.05, a: 0.72 };

        div()
            .id("update-available-modal-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(backdrop)
            .flex()
            .items_center()
            .justify_center()
            .p(px(theme.spacing.xxl))
            .on_click(move |_ev, _win, cx| {
                state_dismiss.update(cx, |s, cx| {
                    s.dismiss_update_modal();
                    cx.notify();
                });
            })
            .child(
                div()
                    .id("update-available-modal-dialog")
                    .on_click(|_ev, _win, cx| {
                        cx.stop_propagation();
                    })
                    .w(px(460.0))
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
                            .font_family(FONT_FAMILY_UI)
                            .text_sm()
                            .text_color(colors.text_primary)
                            .child(labels.update_modal_title),
                    )
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
                    ),
            )
    }

    fn build_activity_rail(
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

    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    fn render_bottom_bar_buttons(
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
                .bg(if is_term_active { colors.bg_elevated } else { colors.bg_status_bar })
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
                .bg(if is_logs_active { colors.bg_elevated } else { colors.bg_status_bar })
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

        row
    }

    fn render_bottom_status_bar(
        &self,
        telemetry_summary: core::shared_kernel::execution::telemetry::TelemetrySummary,
        chrome: &UiChromeConfig,
        labels: &'static crate::locale::UiLabels,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let buttons = self.render_bottom_bar_buttons(telemetry_summary, chrome, labels, theme, cx);

        div()
            .w_full()
            .h(px(theme.layout.status_bar_height))
            .min_h(px(theme.layout.status_bar_height))
            .px(px(theme.spacing.lg))
            .bg(colors.bg_status_bar)
            .border_t_1()
            .border_color(colors.border)
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .child(buttons)
    }

    #[allow(clippy::too_many_lines)]
    fn build_diagnostics_dock(
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
            is_tall_dock: dock_snap.is_tall,
            show_terminal_presets: chrome.show_terminal_presets,
            show_inline_categories,
            terminal_history: dock_snap.terminal_history,
            labels,
            theme,
        };

        let state_tab = self.state.clone();
        let state_cat = self.state.clone();
        let state_save_preset = self.state.clone();
        let settings_save_preset = self.settings_state.clone();
        let settings_del_preset = self.settings_state.clone();
        let state_height = self.state.clone();
        let state_focus_term = self.state.clone();
        let view_submit_term = cx.entity().clone();
        let view_preset_term = cx.entity().clone();
        let state_filter = self.state.clone();
        let container_copy = self.container.clone();
        let state_copy = self.state.clone();
        let container_clear = self.container.clone();
        let state_clear = self.state.clone();
        let state_close = self.state.clone();

        DiagnosticsDockComponent::new(props)
            .on_select_tab(move |tab, _win, cx| {
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
                move |_ev, _win, cx| {
                    state_height.update(cx, |s, cx| {
                        s.toggle_dock_tall_height();
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
            .with_actions(
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
}

fn build_recorded_hotkey_string(event: &KeyDownEvent) -> Option<String> {
    let key = event.keystroke.key.as_str().trim();
    if key.is_empty()
        || key.eq_ignore_ascii_case("control")
        || key.eq_ignore_ascii_case("ctrl")
        || key.eq_ignore_ascii_case("shift")
        || key.eq_ignore_ascii_case("alt")
        || key.eq_ignore_ascii_case("option")
        || key.eq_ignore_ascii_case("platform")
        || key.eq_ignore_ascii_case("super")
        || key.eq_ignore_ascii_case("meta")
        || key.eq_ignore_ascii_case("win")
        || key.eq_ignore_ascii_case("command")
    {
        return None;
    }

    let mods = event.keystroke.modifiers;
    if !mods.control && !mods.alt && !mods.shift && !mods.platform {
        return None;
    }

    let mut out = String::with_capacity(24);
    if mods.control {
        out.push_str("Ctrl+");
    }
    if mods.alt {
        out.push_str("Alt+");
    }
    if mods.shift {
        out.push_str("Shift+");
    }
    if mods.platform {
        out.push_str("Super+");
    }

    let mapped_key = match key {
        " " => "Space",
        "-" => "Minus",
        "=" | "+" => "Plus",
        "`" | "~" | "ё" | "Ё" => "Backquote",
        other => map_cyrillic_to_qwerty_token(other),
    };
    out.push_str(mapped_key);
    Some(out)
}

#[inline]
fn map_cyrillic_to_qwerty_token(token: &str) -> &str {
    match token {
        "й" | "Й" => "Q",
        "ц" | "Ц" => "W",
        "у" | "У" => "E",
        "к" | "К" => "R",
        "е" | "Е" => "T",
        "н" | "Н" => "Y",
        "г" | "Г" => "U",
        "ш" | "Ш" => "I",
        "щ" | "Щ" => "O",
        "з" | "З" => "P",
        "ф" | "Ф" => "A",
        "ы" | "Ы" => "S",
        "в" | "В" => "D",
        "а" | "А" => "F",
        "п" | "П" => "G",
        "р" | "Р" => "H",
        "о" | "О" => "J",
        "л" | "Л" => "K",
        "д" | "Д" => "L",
        "я" | "Я" => "Z",
        "ч" | "Ч" => "X",
        "с" | "С" => "C",
        "м" | "М" => "V",
        "и" | "И" => "B",
        "т" | "Т" => "N",
        "ь" | "Ь" => "M",
        other => other,
    }
}

#[inline]
fn extract_printable_regex_char(event: &KeyDownEvent) -> Option<char> {
    let key = event.keystroke.key.as_str();
    if key == " " || key.eq_ignore_ascii_case("space") {
        return Some(' ');
    }
    if let Some(ref kc) = event.keystroke.key_char {
        let mut chars = kc.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
            && !ch.is_control()
        {
            return Some(ch);
        }
    }
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_none() && !ch.is_control() { Some(ch) } else { None }
}

#[inline]
fn extract_printable_search_char(event: &KeyDownEvent) -> Option<char> {
    let key = event.keystroke.key.as_str();
    if key == " " || key.eq_ignore_ascii_case("space") {
        return Some(' ');
    }
    if let Some(ref kc) = event.keystroke.key_char {
        let mut chars = kc.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
            && !ch.is_control()
        {
            return Some(ch);
        }
    }
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_none() && (ch.is_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/' | ' '))
    {
        Some(ch)
    } else {
        None
    }
}

#[inline]
fn is_key_c(key: &str) -> bool {
    key.eq_ignore_ascii_case("c") || key == "с" || key == "С"
}

#[inline]
fn is_key_f(key: &str) -> bool {
    key.eq_ignore_ascii_case("f") || key == "а" || key == "А"
}

#[inline]
fn is_key_s(key: &str) -> bool {
    key.eq_ignore_ascii_case("s") || key == "ы" || key == "Ы"
}

#[inline]
fn is_key_v(key: &str) -> bool {
    key.eq_ignore_ascii_case("v") || key == "м" || key == "М"
}

#[inline]
fn is_key_k(key: &str) -> bool {
    key.eq_ignore_ascii_case("k") || key == "л" || key == "Л"
}

#[inline]
fn is_key_o(key: &str) -> bool {
    key.eq_ignore_ascii_case("o") || key == "щ" || key == "Щ"
}

#[inline]
fn is_key_r(key: &str) -> bool {
    key.eq_ignore_ascii_case("r") || key == "к" || key == "К"
}

#[inline]
fn is_key_z(key: &str) -> bool {
    key.eq_ignore_ascii_case("z") || key == "я" || key == "Я"
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
    #[allow(clippy::too_many_lines)]
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
        let top_bar = self.render_top_bar(active_tab, locale, theme_variant, &project_display);

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

        if chrome.show_bottom_bar {
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
            .on_click(move |_ev, _win, cx: &mut App| {
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
