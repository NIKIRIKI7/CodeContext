use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use core::system::ports::hotkey::{
    DEFAULT_WAYLAND_SHORTCUT_HINT, GlobalHotkeyPort, HotkeyKey, HotkeyPortError,
    HotkeyRegistrationStatus, HotkeyShortcut,
};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

thread_local! {
    static THREAD_LOCAL_OS_HOTKEY: RefCell<Option<(GlobalHotKeyManager, Option<HotKey>)>> = const {
        RefCell::new(None)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HotkeyBackendKind {
    InMemory,
    SimulatedWaylandRestricted,
    NativeOs,
}

#[derive(Debug, Default)]
struct HotkeyInnerState {
    status: HotkeyRegistrationStatus,
    active_shortcut: Option<HotkeyShortcut>,
    active_native_id: Option<u32>,
    simulated_conflicts: HashSet<HotkeyShortcut>,
}

#[derive(Debug, Clone)]
pub struct SystemGlobalHotkey {
    backend: HotkeyBackendKind,
    state: Arc<Mutex<HotkeyInnerState>>,
    trigger_counter: Arc<AtomicUsize>,
}

impl Default for SystemGlobalHotkey {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemGlobalHotkey {
    #[inline]
    pub fn new() -> Self {
        let backend = if is_pure_wayland_without_x11() {
            HotkeyBackendKind::SimulatedWaylandRestricted
        } else {
            HotkeyBackendKind::NativeOs
        };
        Self {
            backend,
            state: Arc::new(Mutex::new(HotkeyInnerState::default())),
            trigger_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        Self {
            backend: HotkeyBackendKind::InMemory,
            state: Arc::new(Mutex::new(HotkeyInnerState::default())),
            trigger_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[inline]
    pub fn new_simulated_wayland_restricted() -> Self {
        Self {
            backend: HotkeyBackendKind::SimulatedWaylandRestricted,
            state: Arc::new(Mutex::new(HotkeyInnerState::default())),
            trigger_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn simulate_conflict(&self, shortcut: HotkeyShortcut) {
        if let Ok(mut guard) = self.state.lock() {
            guard.simulated_conflicts.insert(shortcut);
        }
    }

    pub fn clear_simulated_conflicts(&self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.simulated_conflicts.clear();
        }
    }

    #[inline]
    pub fn simulate_trigger(&self) {
        self.trigger_counter.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn active_shortcut(&self) -> Option<HotkeyShortcut> {
        self.state.lock().ok().and_then(|g| g.active_shortcut)
    }
}

impl GlobalHotkeyPort for SystemGlobalHotkey {
    #[inline]
    fn status(&self) -> HotkeyRegistrationStatus {
        self.state.lock().map(|g| g.status.clone()).unwrap_or_default()
    }

    fn register_shortcut(
        &self,
        shortcut: HotkeyShortcut,
    ) -> Result<HotkeyRegistrationStatus, HotkeyPortError> {
        let mut guard =
            self.state.lock().map_err(|e| HotkeyPortError::PlatformError(e.to_string()))?;

        if guard.active_shortcut == Some(shortcut) && guard.status.is_active() {
            return Ok(guard.status.clone());
        }

        if self.backend == HotkeyBackendKind::SimulatedWaylandRestricted {
            let hint: Arc<str> = Arc::from(DEFAULT_WAYLAND_SHORTCUT_HINT);
            let status =
                HotkeyRegistrationStatus::UnsupportedByDisplayServer { hint_command: hint };
            guard.status = status;
            guard.active_shortcut = None;
            guard.active_native_id = None;
            return Err(HotkeyPortError::UnsupportedDisplayServer(
                DEFAULT_WAYLAND_SHORTCUT_HINT.to_owned(),
            ));
        }

        let canonical_str = shortcut.to_canonical_string();
        if guard.simulated_conflicts.contains(&shortcut) {
            let arc_shortcut: Arc<str> = Arc::from(canonical_str.as_str());
            let status = HotkeyRegistrationStatus::ConflictWithOtherApp { shortcut: arc_shortcut };
            guard.status = status;
            guard.active_shortcut = None;
            guard.active_native_id = None;
            return Err(HotkeyPortError::Conflict(canonical_str));
        }

        let native_hk = to_native_hotkey(shortcut);
        if self.backend == HotkeyBackendKind::NativeOs
            && let Err(conflict_err) = try_register_on_thread_local_os_manager(native_hk)
        {
            let arc_shortcut: Arc<str> = Arc::from(canonical_str);
            let status = HotkeyRegistrationStatus::ConflictWithOtherApp { shortcut: arc_shortcut };
            guard.status = status;
            guard.active_shortcut = None;
            guard.active_native_id = None;
            return Err(conflict_err);
        }

        let arc_shortcut: Arc<str> = Arc::from(canonical_str);
        let status = HotkeyRegistrationStatus::RegisteredActive { shortcut: arc_shortcut };
        guard.status = status.clone();
        guard.active_shortcut = Some(shortcut);
        guard.active_native_id = Some(native_hk.id());
        Ok(status)
    }

    fn unregister(&self) -> Result<(), HotkeyPortError> {
        let mut guard =
            self.state.lock().map_err(|e| HotkeyPortError::PlatformError(e.to_string()))?;
        if self.backend == HotkeyBackendKind::NativeOs {
            unregister_on_thread_local_os_manager();
        }
        guard.active_shortcut = None;
        guard.active_native_id = None;
        guard.status = HotkeyRegistrationStatus::NotRegistered;
        Ok(())
    }

    fn try_recv_hotkey_trigger(&self) -> bool {
        if self.backend == HotkeyBackendKind::NativeOs {
            let expected_id = self.state.lock().ok().and_then(|g| g.active_native_id);
            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state == HotKeyState::Pressed
                    && expected_id.is_none_or(|id| id == event.id)
                {
                    self.trigger_counter.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        let count = self.trigger_counter.swap(0, Ordering::Relaxed);
        count > 0
    }
}

#[inline]
pub fn to_native_hotkey(shortcut: HotkeyShortcut) -> HotKey {
    let mut mods = Modifiers::empty();
    if shortcut.modifiers.ctrl() {
        mods |= Modifiers::CONTROL;
    }
    if shortcut.modifiers.alt() {
        mods |= Modifiers::ALT;
    }
    if shortcut.modifiers.shift() {
        mods |= Modifiers::SHIFT;
    }
    if shortcut.modifiers.super_key() {
        mods |= Modifiers::SUPER;
    }

    let code = map_key_to_code(shortcut.key);
    let mods_opt = (!mods.is_empty()).then_some(mods);
    HotKey::new(mods_opt, code)
}

#[inline]
const fn map_key_to_code(key: HotkeyKey) -> Code {
    match key {
        HotkeyKey::Char(ch) => map_char_to_code(ch),
        HotkeyKey::F(n) => map_fn_to_code(n),
        HotkeyKey::Space => Code::Space,
        HotkeyKey::Enter => Code::Enter,
        HotkeyKey::Escape => Code::Escape,
        HotkeyKey::Tab => Code::Tab,
        HotkeyKey::Backquote => Code::Backquote,
        HotkeyKey::Up => Code::ArrowUp,
        HotkeyKey::Down => Code::ArrowDown,
        HotkeyKey::Left => Code::ArrowLeft,
        HotkeyKey::Right => Code::ArrowRight,
        HotkeyKey::Home => Code::Home,
        HotkeyKey::End => Code::End,
        HotkeyKey::PageUp => Code::PageUp,
        HotkeyKey::PageDown => Code::PageDown,
        HotkeyKey::Backspace => Code::Backspace,
        HotkeyKey::Delete => Code::Delete,
        HotkeyKey::Insert => Code::Insert,
        HotkeyKey::Plus => Code::Equal,
        HotkeyKey::Minus => Code::Minus,
    }
}

#[inline]
const fn map_char_to_code(ch: char) -> Code {
    match ch {
        'A' => Code::KeyA,
        'B' => Code::KeyB,
        'D' => Code::KeyD,
        'E' => Code::KeyE,
        'F' => Code::KeyF,
        'G' => Code::KeyG,
        'H' => Code::KeyH,
        'I' => Code::KeyI,
        'J' => Code::KeyJ,
        'K' => Code::KeyK,
        'L' => Code::KeyL,
        'M' => Code::KeyM,
        'N' => Code::KeyN,
        'O' => Code::KeyO,
        'P' => Code::KeyP,
        'Q' => Code::KeyQ,
        'R' => Code::KeyR,
        'S' => Code::KeyS,
        'T' => Code::KeyT,
        'U' => Code::KeyU,
        'V' => Code::KeyV,
        'W' => Code::KeyW,
        'X' => Code::KeyX,
        'Y' => Code::KeyY,
        'Z' => Code::KeyZ,
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,
        _ => Code::KeyC,
    }
}

#[inline]
const fn map_fn_to_code(num: u8) -> Code {
    match num {
        1 => Code::F1,
        2 => Code::F2,
        3 => Code::F3,
        4 => Code::F4,
        5 => Code::F5,
        6 => Code::F6,
        7 => Code::F7,
        8 => Code::F8,
        9 => Code::F9,
        10 => Code::F10,
        11 => Code::F11,
        _ => Code::F12,
    }
}

#[inline]
fn is_pure_wayland_without_x11() -> bool {
    cfg!(target_os = "linux")
        && std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var_os("DISPLAY").is_none()
}

fn try_register_on_thread_local_os_manager(hotkey: HotKey) -> Result<(), HotkeyPortError> {
    THREAD_LOCAL_OS_HOTKEY.with(|slot| {
        let mut borrow = slot.borrow_mut();
        if borrow.is_none()
            && let Ok(mgr) = GlobalHotKeyManager::new()
        {
            *borrow = Some((mgr, None));
        }

        if let Some((ref mgr, ref mut current_hk)) = *borrow {
            if let Some(old) = current_hk.take() {
                let _ = mgr.unregister(old);
            }
            if let Err(err) = mgr.register(hotkey) {
                return Err(HotkeyPortError::Conflict(err.to_string()));
            }
            *current_hk = Some(hotkey);
        }
        Ok(())
    })
}

fn unregister_on_thread_local_os_manager() {
    THREAD_LOCAL_OS_HOTKEY.with(|slot| {
        let mut borrow = slot.borrow_mut();
        if let Some((ref mgr, ref mut current_hk)) = *borrow
            && let Some(old) = current_hk.take()
        {
            let _ = mgr.unregister(old);
        }
    });
}
