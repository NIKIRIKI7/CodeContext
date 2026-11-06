use gpui::{Context, KeyDownEvent, Window};

use crate::features::main_window::components::diagnostics_dock::DockToolTab;
use crate::features::main_window::state::WorkspaceTab;
use crate::features::main_window::view::MainWindowView;

impl MainWindowView {
    pub(crate) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mods = event.keystroke.modifiers;
        let primary_mod =
            if cfg!(target_os = "macos") { mods.platform || mods.control } else { mods.control };
        let key = event.keystroke.key.as_str();

        if self.try_handle_settings_recording_key(event, key, cx) {
            return;
        }
        if self.try_handle_terminal_input_key(event, key, primary_mod, cx) {
            cx.stop_propagation();
            return;
        }
        if self.try_handle_packer_modals_key(event, key, primary_mod, cx) {
            cx.stop_propagation();
            return;
        }
        if self.try_handle_search_input_key(event, key, primary_mod, cx) {
            return;
        }
        if let Some(pressed_shortcut) = build_recorded_hotkey_string(event)
            && self.try_dispatch_custom_keymap(&pressed_shortcut, cx)
        {
            cx.stop_propagation();
            return;
        }
        self.handle_global_workspace_shortcuts(key, primary_mod, mods.shift, mods.alt, cx);
    }

    fn try_handle_settings_recording_key(
        &self,
        event: &KeyDownEvent,
        key: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.state.read(cx).is_settings_open() {
            return false;
        }
        if let Some(action) = self.settings_state.read(cx).recording_action() {
            cx.stop_propagation();
            if key.eq_ignore_ascii_case("escape") {
                self.settings_state.update(cx, |s, cx| {
                    s.cancel_recording_hotkey();
                    cx.notify();
                });
                return true;
            }
            if let Some(shortcut_str) = build_recorded_hotkey_string(event) {
                self.settings_state.update(cx, |s, cx| {
                    let _ = s.rebind_action_shortcut(action, &shortcut_str);
                    cx.notify();
                });
            }
            return true;
        }
        if key.eq_ignore_ascii_case("escape")
            && self.settings_state.read(cx).is_hotkeys_modal_open()
        {
            cx.stop_propagation();
            self.settings_state.update(cx, |s, cx| {
                s.close_hotkeys_modal();
                cx.notify();
            });
            return true;
        }
        false
    }

    fn try_handle_search_input_key(
        &self,
        event: &KeyDownEvent,
        key: &str,
        primary_mod: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.packer_state.read(cx).ui_status().search_focused {
            return false;
        }
        let mods = event.keystroke.modifiers;
        if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("enter") {
            cx.stop_propagation();
            self.packer_state.update(cx, |p, cx| {
                p.set_search_focused(false);
                cx.notify();
            });
            return true;
        }
        if key.eq_ignore_ascii_case("backspace") {
            cx.stop_propagation();
            self.packer_state.update(cx, |p, cx| {
                if p.pop_search_char() {
                    cx.notify();
                }
            });
            return true;
        }
        if primary_mod && !mods.shift && is_key_v(key) {
            cx.stop_propagation();
            if let Some(clip) = cx.read_from_clipboard().and_then(|i| i.text()) {
                let first_line = clip.lines().next().unwrap_or("").trim().to_owned();
                if !first_line.is_empty() {
                    self.packer_state.update(cx, |p, cx| {
                        for ch in first_line.chars() {
                            p.append_search_char(ch);
                        }
                        cx.notify();
                    });
                }
            }
            return true;
        }
        if !primary_mod && !mods.alt && !key.eq_ignore_ascii_case("f5") {
            cx.stop_propagation();
            if let Some(ch) = extract_printable_search_char(event) {
                self.packer_state.update(cx, |p, cx| {
                    p.append_search_char(ch);
                    cx.notify();
                });
            }
            return true;
        }
        self.packer_state.update(cx, |p, cx| {
            p.set_search_focused(false);
            cx.notify();
        });
        false
    }

    fn handle_global_workspace_shortcuts(
        &mut self,
        key: &str,
        primary_mod: bool,
        shift: bool,
        alt: bool,
        cx: &mut Context<Self>,
    ) {
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
        if key.eq_ignore_ascii_case("f5") || (primary_mod && !shift && is_key_r(key)) {
            self.packer_state.update(cx, |s, cx| {
                let _ = s.rescan_current_project();
                cx.notify();
            });
            return;
        }
        if primary_mod && !shift && is_key_f(key) {
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
        if !primary_mod && !alt && self.handle_tree_navigation_key(key, shift, cx) {
            return;
        }
        if !primary_mod {
            return;
        }
        if !shift && is_key_k(key) {
            self.copy_packed_context_to_os_clipboard(cx);
            return;
        }
        if !shift && is_key_o(key) {
            let prompt_label = self.state.read(cx).locale().labels().btn_open_folder_prompt;
            Self::prompt_and_open_project_folder(
                self.packer_state.clone(),
                self.applier_state.clone(),
                prompt_label,
                cx,
            );
            return;
        }
        if shift && (is_key_c(key) || is_key_s(key)) {
            self.copy_packed_context_to_os_clipboard(cx);
            return;
        }
        if !shift && is_key_v(key) {
            self.paste_clipboard_into_applier(cx);
            return;
        }
        let active_tab = self.state.read(cx).active_tab();
        if active_tab == WorkspaceTab::ContextPacker && !shift && is_key_c(key) {
            self.copy_packed_context_to_os_clipboard(cx);
        } else if active_tab == WorkspaceTab::ChangeApplier {
            if key.eq_ignore_ascii_case("enter") {
                self.applier_state.update(cx, |s, cx| {
                    let _ = s.apply_changes_to_disk();
                    cx.notify();
                });
            } else if !shift && is_key_z(key) {
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
        let (is_term_open, is_term_focused) = {
            let s = self.state.read(cx);
            (
                s.is_logs_dock_open() && s.dock_active_tab() == DockToolTab::Terminal,
                s.bottom_dock().terminal_focused,
            )
        };
        if !is_term_open || !is_term_focused {
            return false;
        }
        if key.eq_ignore_ascii_case("escape") {
            self.state.update(cx, |s, cx| {
                s.bottom_dock_mut().terminal_focused = false;
                cx.notify();
            });
            return true;
        }
        if key.eq_ignore_ascii_case("enter") {
            let cmd = self.state.update(cx, |s, cx| {
                let d = s.bottom_dock_mut();
                d.terminal_focused = true;
                let taken = std::mem::take(&mut d.terminal_input);
                cx.notify();
                taken
            });
            self.spawn_terminal_command(&cmd, cx);
            return true;
        }
        if key.eq_ignore_ascii_case("backspace") {
            self.state.update(cx, |s, cx| {
                let d = s.bottom_dock_mut();
                d.terminal_focused = true;
                if d.terminal_input.pop().is_some() {
                    cx.notify();
                }
            });
            return true;
        }
        let mods = event.keystroke.modifiers;
        if primary_mod && !mods.shift && is_key_v(key) {
            if let Some(clip) = cx.read_from_clipboard().and_then(|i| i.text()) {
                let single_line = clip.lines().next().unwrap_or("").trim().to_owned();
                if !single_line.is_empty() {
                    self.state.update(cx, |s, cx| {
                        let d = s.bottom_dock_mut();
                        d.terminal_input.push_str(&single_line);
                        d.terminal_focused = true;
                        cx.notify();
                    });
                }
            }
            return true;
        }
        if !primary_mod && !mods.alt {
            if key == " " || key.eq_ignore_ascii_case("space") {
                self.state.update(cx, |s, cx| {
                    let d = s.bottom_dock_mut();
                    d.terminal_input.push(' ');
                    d.terminal_focused = true;
                    cx.notify();
                });
                return true;
            }
            if let Some(ch) = extract_printable_regex_char(event) {
                self.state.update(cx, |s, cx| {
                    let d = s.bottom_dock_mut();
                    d.terminal_input.push(ch);
                    d.terminal_focused = true;
                    cx.notify();
                });
                return true;
            }
        }
        false
    }

    fn try_handle_packer_modals_key(
        &self,
        event: &KeyDownEvent,
        key: &str,
        primary_mod: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.packer_state.read(cx).is_prompt_modal_open() {
            return self.handle_prompt_modal_keys(event, key, primary_mod, cx);
        }
        if self.packer_state.read(cx).ui_status().regex_filter_open {
            return self.handle_filter_modal_keys(event, key, primary_mod, cx);
        }
        false
    }

    fn handle_prompt_modal_keys(
        &self,
        event: &KeyDownEvent,
        key: &str,
        primary_mod: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if key.eq_ignore_ascii_case("escape") {
            self.packer_state.update(cx, |p, cx| {
                p.set_prompt_modal_open(false);
                cx.notify();
            });
            return true;
        }
        let mods = event.keystroke.modifiers;
        if primary_mod && !mods.shift && is_key_v(key) {
            if let Some(clip) = cx.read_from_clipboard().and_then(|i| i.text()) {
                self.packer_state.update(cx, |p, cx| {
                    p.paste_into_prompt_composer(&clip);
                    cx.notify();
                });
            }
            return true;
        }
        if key.eq_ignore_ascii_case("tab") {
            self.packer_state.update(cx, |p, cx| {
                if p.is_prompt_category_focused() {
                    p.set_prompt_title_focused(true);
                } else if p.is_prompt_title_focused() {
                    p.set_prompt_input_focused(true);
                } else {
                    p.set_prompt_category_focused(true);
                }
                cx.notify();
            });
            return true;
        }
        let (is_cat, is_title, is_body) = {
            let ps = self.packer_state.read(cx);
            (
                ps.is_prompt_category_focused(),
                ps.is_prompt_title_focused(),
                ps.is_prompt_input_focused(),
            )
        };
        if is_cat || is_title {
            if key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    if is_cat {
                        p.set_prompt_title_focused(true);
                    } else {
                        p.set_prompt_input_focused(true);
                    }
                    cx.notify();
                });
                return true;
            }
            if key.eq_ignore_ascii_case("backspace") {
                self.packer_state.update(cx, |p, cx| {
                    let changed = if is_cat {
                        p.pop_prompt_category_char()
                    } else {
                        p.pop_prompt_title_char()
                    };
                    if changed {
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
                    if is_cat {
                        p.append_prompt_category_char(ch);
                    } else {
                        p.append_prompt_title_char(ch);
                    }
                    cx.notify();
                });
            }
            return true;
        }
        if is_body {
            if key.eq_ignore_ascii_case("enter") {
                self.packer_state.update(cx, |p, cx| {
                    if mods.shift {
                        p.append_prompt_input_char('\n');
                    } else {
                        p.submit_manual_prompt_draft();
                    }
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
            if !primary_mod
                && !mods.alt
                && let Some(ch) = extract_printable_regex_char(event)
            {
                self.packer_state.update(cx, |p, cx| {
                    p.append_prompt_input_char(ch);
                    cx.notify();
                });
            }
            return true;
        }
        false
    }

    fn handle_filter_modal_keys(
        &self,
        event: &KeyDownEvent,
        key: &str,
        primary_mod: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if key.eq_ignore_ascii_case("escape") {
            self.packer_state.update(cx, |p, cx| {
                p.set_regex_filter_open(false);
                cx.notify();
            });
            return true;
        }
        let (is_ext, is_folder, is_re_name, is_re) = {
            let ps = self.packer_state.read(cx);
            (
                ps.is_extension_input_focused(),
                ps.is_folder_input_focused(),
                ps.is_regex_name_input_focused(),
                ps.is_regex_input_focused(),
            )
        };
        if !(is_ext || is_folder || is_re_name || is_re) {
            return false;
        }
        if key.eq_ignore_ascii_case("enter") {
            self.packer_state.update(cx, |p, cx| {
                if is_ext {
                    p.submit_manual_extension();
                } else if is_folder {
                    p.submit_manual_folder_exclusion();
                } else if is_re_name {
                    let _ = p.submit_manual_regex_template();
                } else {
                    p.set_regex_input_focused(false);
                }
                cx.notify();
            });
            return true;
        }
        if key.eq_ignore_ascii_case("backspace") {
            self.packer_state.update(cx, |p, cx| {
                let changed = if is_ext {
                    p.pop_extension_input_char()
                } else if is_folder {
                    p.pop_folder_input_char()
                } else if is_re_name {
                    p.pop_regex_name_char()
                } else {
                    p.pop_regex_char()
                };
                if changed {
                    cx.notify();
                }
            });
            return true;
        }
        let mods = event.keystroke.modifiers;
        if !primary_mod
            && !mods.alt
            && let Some(ch) = extract_printable_regex_char(event)
        {
            self.packer_state.update(cx, |p, cx| {
                if is_ext {
                    p.append_extension_input_char(ch);
                } else if is_folder {
                    p.append_folder_input_char(ch);
                } else if is_re_name {
                    p.append_regex_name_char(ch);
                } else {
                    p.append_regex_char(ch);
                }
                cx.notify();
            });
            return true;
        }
        true
    }

    pub(crate) fn try_dispatch_custom_keymap(&self, pressed: &str, cx: &mut Context<Self>) -> bool {
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

    pub(crate) fn handle_tree_navigation_key(
        &self,
        key: &str,
        shift: bool,
        cx: &mut Context<Self>,
    ) -> bool {
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
}

pub(crate) fn build_recorded_hotkey_string(event: &KeyDownEvent) -> Option<String> {
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
pub(crate) fn extract_printable_regex_char(event: &KeyDownEvent) -> Option<char> {
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
pub(crate) fn extract_printable_search_char(event: &KeyDownEvent) -> Option<char> {
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
