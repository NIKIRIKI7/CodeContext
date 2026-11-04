use std::path::PathBuf;
use std::time::Duration;

use core::packing::aggregates::project::Project;
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
                (s.take_meta_modal_request(), s.project_root().map(std::path::Path::to_path_buf))
            });
            let applier_root =
                this.applier_state.read(cx).project_root().map(std::path::Path::to_path_buf);
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

        if !primary_mod && !mods.alt && self.handle_tree_navigation_key(key, cx) {
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
            is_prompt_open,
            is_prompt_focused,
        ) = {
            let ps = self.packer_state.read(cx);
            (
                ps.ui_status().regex_filter_open,
                ps.is_folder_input_focused(),
                ps.is_extension_input_focused(),
                ps.is_regex_input_focused(),
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
        {
            return false;
        }
        let km = self.settings_state.read(cx).global_config().keymap.clone();
        let active_tab = self.state.read(cx).active_tab();

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

    fn handle_tree_navigation_key(&self, key: &str, cx: &mut Context<Self>) -> bool {
        let snap = self.state.read(cx);
        if snap.active_tab() != WorkspaceTab::ContextPacker
            || snap.is_settings_open()
            || snap.is_project_meta_open()
            || snap.is_recent_projects_open()
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
            self.packer_state.update(cx, |p, cx| {
                if p.toggle_focused_tree_node_checkbox() > 0 {
                    cx.notify();
                }
            });
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
    if let Some(ref kc) = event.keystroke.key_char {
        let mut chars = kc.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
            && !ch.is_control()
        {
            return Some(ch);
        }
    }
    let key = event.keystroke.key.as_str();
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_none() && !ch.is_control() { Some(ch) } else { None }
}

#[inline]
fn extract_printable_search_char(event: &KeyDownEvent) -> Option<char> {
    if let Some(ref kc) = event.keystroke.key_char {
        let mut chars = kc.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
            && !ch.is_control()
        {
            return Some(ch);
        }
    }
    let key = event.keystroke.key.as_str();
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_none() && (ch.is_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/')) {
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

impl Render for MainWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
        }

        let snap = self.state.read(cx);
        let active_tab = snap.active_tab();
        let settings_open = snap.is_settings_open();
        let meta_open = snap.is_project_meta_open();
        let recent_open = snap.is_recent_projects_open();
        let update_modal_open = snap.is_update_modal_open();
        let available_update = snap.available_update();
        let locale = snap.locale();
        let theme_variant = snap.theme_variant();
        let theme = theme_variant.tokens();
        let labels = locale.labels();

        let packer_snap = self.packer_state.read(cx);
        let project_display = packer_snap
            .project_root()
            .map_or_else(|| labels.no_project_selected.to_owned(), |p| p.display().to_string());

        let state_rail_tab = self.state.clone();
        let packer_for_tab = self.packer_state.clone();
        let applier_for_auto = self.applier_state.clone();
        let settings_for_auto = self.settings_state.clone();
        let state_rail_settings = self.state.clone();

        let rail = ActivityRailComponent::new(active_tab, settings_open, labels, theme)
            .with_actions(
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
            );

        let top_bar = self.render_top_bar(active_tab, locale, theme_variant, &project_display);

        let active_content = match active_tab {
            WorkspaceTab::ContextPacker => self.packer_view.clone().into_any_element(),
            WorkspaceTab::ChangeApplier => self.applier_view.clone().into_any_element(),
        };

        let packer_unfocus_search = self.packer_state.clone();
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
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(top_bar)
                    .child(div().flex_1().overflow_hidden().child(active_content)),
            );

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
