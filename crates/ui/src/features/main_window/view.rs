use std::path::PathBuf;
use std::time::Duration;

use core::packing::aggregates::project::Project;
use core::system::ports::hotkey::GlobalHotkeyPort as _;
use core::system::ports::ipc::{IpcCommand, SingleInstanceIpcPort as _};
use di::AppContainer;
use gpui::{
    App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, KeyDownEvent, ParentElement as _, PathPromptOptions, Render, SharedString,
    Styled as _, Window, div, px,
};

use crate::features::change_applier::state::ChangeApplierState;
use crate::features::change_applier::view::ChangeApplierView;
use crate::features::context_packer::state::ContextPackerState;
use crate::features::context_packer::view::ContextPackerView;
use crate::features::main_window::components::activity_rail::ActivityRailComponent;
use crate::features::main_window::components::project_meta_modal::{
    ProjectMetaModalComponent, ProjectMetaProps,
};
use crate::features::main_window::state::{MainWindowState, WorkspaceTab};
use crate::features::settings_modal::state::SettingsModalState;
use crate::features::settings_modal::view::SettingsModalView;

use crate::locale::AppLocale;
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
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
                    if has_hk {
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

        if self.state.read(cx).is_settings_open()
            && self.settings_state.read(cx).is_recording_hotkey()
        {
            if key.eq_ignore_ascii_case("escape") {
                self.settings_state.update(cx, |s, cx| {
                    s.cancel_recording_hotkey();
                    cx.notify();
                });
                return;
            }
            if let Some(shortcut_str) = build_recorded_hotkey_string(event) {
                self.settings_state.update(cx, |s, cx| {
                    let _ = s.rebind_global_hotkey(&shortcut_str);
                    cx.notify();
                });
            }
            return;
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

        if !primary_mod
            && !mods.alt
            && self.state.read(cx).active_tab() == WorkspaceTab::ContextPacker
        {
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    if p.pop_search_char() {
                        cx.notify();
                    }
                });
                return;
            }
            if let Some(ch) = extract_printable_search_char(event) {
                self.packer_state.update(cx, |p, cx| {
                    p.append_search_char(ch);
                    cx.notify();
                });
                return;
            }
        }

        if !primary_mod {
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

    fn handle_escape_key(&self, cx: &mut Context<Self>) {
        let (settings_open, meta_open) = {
            let snap = self.state.read(cx);
            (snap.is_settings_open(), snap.is_project_meta_open())
        };
        if settings_open || meta_open {
            self.state.update(cx, |s, cx| {
                s.set_settings_open(false);
                s.set_project_meta_open(false);
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
        let clip_str = cx.read_from_clipboard().and_then(|item| item.text());
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

    fn drain_external_system_events(&self, window: &mut Window, cx: &mut Context<Self>) {
        let mut focus_requested = false;
        let auto_clip = self.settings_state.read(cx).global_config().auto_read_clipboard_on_apply;
        let active_tab = self.state.read(cx).active_tab();

        while let Some(cmd) = self.container.ipc().try_recv_command() {
            match cmd {
                IpcCommand::OpenProjectAndFocus { project_root } => {
                    let root_for_applier = project_root.clone();
                    self.packer_state.update(cx, |p, cx| {
                        let _ = p.open_project(project_root);
                        cx.notify();
                    });
                    self.applier_state.update(cx, |a, cx| {
                        a.set_project_root(Some(root_for_applier));
                        cx.notify();
                    });
                    focus_requested = true;
                }
                IpcCommand::FocusWindow => {
                    focus_requested = true;
                }
                IpcCommand::Ping => {}
            }
        }

        if self.container.hotkey().try_recv_hotkey_trigger() {
            focus_requested = true;
        }

        if focus_requested {
            window.activate_window();
            self.packer_state.update(cx, |p, cx| {
                if p.sync_with_disk_if_changed() {
                    cx.notify();
                }
            });
            if active_tab == WorkspaceTab::ChangeApplier && auto_clip {
                self.applier_state.update(cx, |a, cx| {
                    if a.load_response_from_clipboard_if_changed() {
                        cx.notify();
                    }
                });
            }
        }
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
        let packer_open = self.packer_state.clone();
        let applier_open = self.applier_state.clone();
        let prompt_label = labels.btn_open_folder_prompt;

        let folder_btn = UiButton::new(
            "nav-btn-open-project-folder",
            SharedString::from(project_display.to_owned()),
        )
        .icon(UiIcon::FolderOpen)
        .variant(ButtonVariant::Secondary)
        .with_theme(theme)
        .on_click(move |_ev, _win, cx| {
            Self::prompt_and_open_project_folder(
                packer_open.clone(),
                applier_open.clone(),
                prompt_label,
                cx,
            );
        });

        div()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.sm))
            .child(folder_btn)
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
        "`" | "~" => "Backquote",
        other => other,
    };
    out.push_str(mapped_key);
    Some(out)
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
        self.drain_external_system_events(window, cx);

        let snap = self.state.read(cx);
        let active_tab = snap.active_tab();
        let settings_open = snap.is_settings_open();
        let meta_open = snap.is_project_meta_open();
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
                move |tab, _win, cx| {
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
                move |_ev, _win, cx| {
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

        let mut root = div()
            .id("main-window-root")
            .track_focus(&self.focus_handle)
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

        if meta_open {
            root = root.child(self.build_meta_modal_overlay(locale, theme, project_display, cx));
        }

        if settings_open {
            root = root.child(self.settings_view.clone());
        }

        root
    }
}
