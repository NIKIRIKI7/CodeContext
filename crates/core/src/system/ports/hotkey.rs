use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_WAYLAND_SHORTCUT_HINT: &str = "codecontext --project .";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum HotkeyActionId {
    #[default]
    ToggleOrFocusWindow,
    QuickPackProject,
    QuickApplyClipboard,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HotkeyModifiers(u8);

impl HotkeyModifiers {
    pub const NONE: Self = Self(0);
    pub const CTRL: Self = Self(1 << 0);
    pub const ALT: Self = Self(1 << 1);
    pub const SHIFT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    #[inline]
    pub const fn empty() -> Self {
        Self::NONE
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[must_use]
    #[inline]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn ctrl(self) -> bool {
        self.contains(Self::CTRL)
    }

    #[inline]
    pub const fn alt(self) -> bool {
        self.contains(Self::ALT)
    }

    #[inline]
    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    #[inline]
    pub const fn super_key(self) -> bool {
        self.contains(Self::SUPER)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyKey {
    Char(char),
    F(u8),
    Space,
    Enter,
    Escape,
    Tab,
    Backquote,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Insert,
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy)]
pub struct ModifierDescriptor {
    pub aliases: &'static [&'static str],
    pub flag: HotkeyModifiers,
    pub canonical_prefix: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct NamedKeyDescriptor {
    pub aliases: &'static [&'static str],
    pub key: HotkeyKey,
    pub canonical_name: &'static str,
}

pub const MODIFIER_REGISTRY: &[ModifierDescriptor] = &[
    ModifierDescriptor {
        aliases: &["ctrl", "control"],
        flag: HotkeyModifiers::CTRL,
        canonical_prefix: "Ctrl+",
    },
    ModifierDescriptor {
        aliases: &["alt", "option"],
        flag: HotkeyModifiers::ALT,
        canonical_prefix: "Alt+",
    },
    ModifierDescriptor {
        aliases: &["shift"],
        flag: HotkeyModifiers::SHIFT,
        canonical_prefix: "Shift+",
    },
    ModifierDescriptor {
        aliases: &["super", "cmd", "command", "meta", "win"],
        flag: HotkeyModifiers::SUPER,
        canonical_prefix: "Super+",
    },
];

pub const NAMED_KEY_REGISTRY: &[NamedKeyDescriptor] = &[
    NamedKeyDescriptor {
        aliases: &["space", "spacebar"],
        key: HotkeyKey::Space,
        canonical_name: "Space",
    },
    NamedKeyDescriptor {
        aliases: &["enter", "return"],
        key: HotkeyKey::Enter,
        canonical_name: "Enter",
    },
    NamedKeyDescriptor {
        aliases: &["esc", "escape"],
        key: HotkeyKey::Escape,
        canonical_name: "Escape",
    },
    NamedKeyDescriptor { aliases: &["tab"], key: HotkeyKey::Tab, canonical_name: "Tab" },
    NamedKeyDescriptor {
        aliases: &["backquote", "grave", "`", "~"],
        key: HotkeyKey::Backquote,
        canonical_name: "Backquote",
    },
    NamedKeyDescriptor { aliases: &["up", "arrowup"], key: HotkeyKey::Up, canonical_name: "Up" },
    NamedKeyDescriptor {
        aliases: &["down", "arrowdown"],
        key: HotkeyKey::Down,
        canonical_name: "Down",
    },
    NamedKeyDescriptor {
        aliases: &["left", "arrowleft"],
        key: HotkeyKey::Left,
        canonical_name: "Left",
    },
    NamedKeyDescriptor {
        aliases: &["right", "arrowright"],
        key: HotkeyKey::Right,
        canonical_name: "Right",
    },
    NamedKeyDescriptor { aliases: &["home"], key: HotkeyKey::Home, canonical_name: "Home" },
    NamedKeyDescriptor { aliases: &["end"], key: HotkeyKey::End, canonical_name: "End" },
    NamedKeyDescriptor {
        aliases: &["pageup", "pgup"],
        key: HotkeyKey::PageUp,
        canonical_name: "PageUp",
    },
    NamedKeyDescriptor {
        aliases: &["pagedown", "pgdn"],
        key: HotkeyKey::PageDown,
        canonical_name: "PageDown",
    },
    NamedKeyDescriptor {
        aliases: &["backspace", "bksp"],
        key: HotkeyKey::Backspace,
        canonical_name: "Backspace",
    },
    NamedKeyDescriptor {
        aliases: &["delete", "del"],
        key: HotkeyKey::Delete,
        canonical_name: "Delete",
    },
    NamedKeyDescriptor {
        aliases: &["insert", "ins"],
        key: HotkeyKey::Insert,
        canonical_name: "Insert",
    },
    NamedKeyDescriptor {
        aliases: &["plus", "equal", "="],
        key: HotkeyKey::Plus,
        canonical_name: "Plus",
    },
    NamedKeyDescriptor {
        aliases: &["minus", "hyphen", "-"],
        key: HotkeyKey::Minus,
        canonical_name: "Minus",
    },
];

impl HotkeyKey {
    #[inline]
    pub const fn canonical_str(self) -> &'static str {
        match self {
            Self::Char(ch) => ascii_char_to_static_str(ch),
            Self::F(num) => fn_num_to_static_str(num),
            Self::Space => "Space",
            Self::Enter => "Enter",
            Self::Escape => "Escape",
            Self::Tab => "Tab",
            Self::Backquote => "Backquote",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::Insert => "Insert",
            Self::Plus => "Plus",
            Self::Minus => "Minus",
        }
    }
}

#[inline]
const fn ascii_char_to_static_str(ch: char) -> &'static str {
    match ch {
        'A' | 'a' => "A",
        'B' | 'b' => "B",
        'C' | 'c' => "C",
        'D' | 'd' => "D",
        'E' | 'e' => "E",
        'F' | 'f' => "F",
        'G' | 'g' => "G",
        'H' | 'h' => "H",
        'I' | 'i' => "I",
        'J' | 'j' => "J",
        'K' | 'k' => "K",
        'L' | 'l' => "L",
        'M' | 'm' => "M",
        'N' | 'n' => "N",
        'O' | 'o' => "O",
        'P' | 'p' => "P",
        'Q' | 'q' => "Q",
        'R' | 'r' => "R",
        'S' | 's' => "S",
        'T' | 't' => "T",
        'U' | 'u' => "U",
        'V' | 'v' => "V",
        'W' | 'w' => "W",
        'X' | 'x' => "X",
        'Y' | 'y' => "Y",
        'Z' | 'z' => "Z",
        '0' => "0",
        '1' => "1",
        '2' => "2",
        '3' => "3",
        '4' => "4",
        '5' => "5",
        '6' => "6",
        '7' => "7",
        '8' => "8",
        '9' => "9",
        _ => "?",
    }
}

#[inline]
const fn fn_num_to_static_str(num: u8) -> &'static str {
    match num {
        1 => "F1",
        2 => "F2",
        3 => "F3",
        4 => "F4",
        5 => "F5",
        6 => "F6",
        7 => "F7",
        8 => "F8",
        9 => "F9",
        10 => "F10",
        11 => "F11",
        _ => "F12",
    }
}

pub type KeyTokenMatcherFn = fn(&str) -> Option<HotkeyKey>;

const DEFAULT_KEY_MATCHER_CHAIN: &[KeyTokenMatcherFn] = &[
    rule_single_ascii_alnum,
    rule_function_key,
    rule_prefixed_key_or_digit,
    rule_named_key_registry,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum HotkeyParseError {
    #[error("Пустая строка комбинации клавиш")]
    EmptyInput,
    #[error("Комбинация должна содержать хотя бы один модификатор (Ctrl, Alt, Shift, Super)")]
    MissingModifier,
    #[error("Комбинация должна содержать основную клавишу")]
    MissingKey,
    #[error("Указано более одной основной клавиши")]
    MultipleKeys,
    #[error("Неизвестный токен клавиши или модификатора")]
    UnknownToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HotkeyShortcut {
    pub modifiers: HotkeyModifiers,
    pub key: HotkeyKey,
}

impl Default for HotkeyShortcut {
    fn default() -> Self {
        Self::default_ctrl_shift_c()
    }
}

impl fmt::Display for HotkeyShortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for spec in MODIFIER_REGISTRY {
            if self.modifiers.contains(spec.flag) {
                f.write_str(spec.canonical_prefix)?;
            }
        }
        f.write_str(self.key.canonical_str())
    }
}

impl HotkeyShortcut {
    #[inline]
    pub const fn new(modifiers: HotkeyModifiers, key: HotkeyKey) -> Self {
        Self { modifiers, key }
    }

    #[inline]
    pub const fn default_ctrl_shift_c() -> Self {
        Self {
            modifiers: HotkeyModifiers::CTRL.with(HotkeyModifiers::SHIFT),
            key: HotkeyKey::Char('C'),
        }
    }

    #[inline]
    pub const fn default_platform() -> Self {
        if cfg!(target_os = "macos") {
            Self {
                modifiers: HotkeyModifiers::SUPER.with(HotkeyModifiers::SHIFT),
                key: HotkeyKey::Char('C'),
            }
        } else {
            Self::default_ctrl_shift_c()
        }
    }

    #[inline]
    pub fn parse(raw: &str) -> Result<Self, HotkeyParseError> {
        Self::parse_with_chain(raw, DEFAULT_KEY_MATCHER_CHAIN)
    }

    pub fn parse_with_chain(
        raw: &str,
        key_matchers: &[KeyTokenMatcherFn],
    ) -> Result<Self, HotkeyParseError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(HotkeyParseError::EmptyInput);
        }

        let mut modifiers = HotkeyModifiers::empty();
        let mut key_opt: Option<HotkeyKey> = None;

        for part in trimmed.split('+') {
            let token = part.trim();
            if token.is_empty() {
                return Err(HotkeyParseError::UnknownToken);
            }

            if token.len() >= 3
                && let Some(mod_flag) = parse_modifier_token(token)
            {
                modifiers = modifiers.with(mod_flag);
            } else if let Some(parsed_key) = parse_key_token_with_chain(token, key_matchers) {
                if key_opt.is_some() {
                    return Err(HotkeyParseError::MultipleKeys);
                }
                key_opt = Some(parsed_key);
            } else {
                return Err(HotkeyParseError::UnknownToken);
            }
        }

        if modifiers.is_empty() {
            return Err(HotkeyParseError::MissingModifier);
        }
        let Some(key) = key_opt else {
            return Err(HotkeyParseError::MissingKey);
        };

        Ok(Self { modifiers, key })
    }

    #[inline]
    pub fn to_canonical_string(self) -> String {
        let mut out = String::with_capacity(24);
        for spec in MODIFIER_REGISTRY {
            if self.modifiers.contains(spec.flag) {
                out.push_str(spec.canonical_prefix);
            }
        }
        out.push_str(self.key.canonical_str());
        out
    }
}

#[inline]
fn parse_modifier_token(token: &str) -> Option<HotkeyModifiers> {
    let first_upper = token.as_bytes().first().copied()?.to_ascii_uppercase();
    match first_upper {
        b'C' => {
            if token.eq_ignore_ascii_case("ctrl") || token.eq_ignore_ascii_case("control") {
                Some(HotkeyModifiers::CTRL)
            } else if token.eq_ignore_ascii_case("cmd") || token.eq_ignore_ascii_case("command") {
                Some(HotkeyModifiers::SUPER)
            } else if token.eq_ignore_ascii_case("cmdorctrl")
                || token.eq_ignore_ascii_case("commandorcontrol")
            {
                Some(if cfg!(target_os = "macos") {
                    HotkeyModifiers::SUPER
                } else {
                    HotkeyModifiers::CTRL
                })
            } else {
                None
            }
        }
        b'S' => {
            if token.eq_ignore_ascii_case("shift") {
                Some(HotkeyModifiers::SHIFT)
            } else if token.eq_ignore_ascii_case("super") {
                Some(HotkeyModifiers::SUPER)
            } else {
                None
            }
        }
        b'A' if token.eq_ignore_ascii_case("alt") => Some(HotkeyModifiers::ALT),
        b'O' if token.eq_ignore_ascii_case("option") => Some(HotkeyModifiers::ALT),
        b'M' if token.eq_ignore_ascii_case("meta") => Some(HotkeyModifiers::SUPER),
        b'W' if token.eq_ignore_ascii_case("win") => Some(HotkeyModifiers::SUPER),
        _ => None,
    }
}

#[inline]
fn parse_key_token_with_chain(token: &str, chain: &[KeyTokenMatcherFn]) -> Option<HotkeyKey> {
    for &matcher in chain {
        if let Some(key) = matcher(token) {
            return Some(key);
        }
    }
    None
}

#[inline]
fn rule_single_ascii_alnum(token: &str) -> Option<HotkeyKey> {
    let [b] = token.as_bytes() else {
        return None;
    };
    if b.is_ascii_alphabetic() {
        Some(HotkeyKey::Char(b.to_ascii_uppercase() as char))
    } else if b.is_ascii_digit() {
        Some(HotkeyKey::Char(*b as char))
    } else {
        match *b {
            b'`' | b'~' => Some(HotkeyKey::Backquote),
            b'=' => Some(HotkeyKey::Plus),
            b'-' => Some(HotkeyKey::Minus),
            _ => None,
        }
    }
}

#[inline]
fn rule_function_key(token: &str) -> Option<HotkeyKey> {
    let bytes = token.as_bytes();
    if (bytes.len() == 2 || bytes.len() == 3)
        && bytes.first().is_some_and(|&b| b == b'f' || b == b'F')
        && let Some(num_str) = token.get(1..)
        && let Ok(num) = num_str.parse::<u8>()
        && (1..=12).contains(&num)
    {
        return Some(HotkeyKey::F(num));
    }
    None
}

#[inline]
fn rule_prefixed_key_or_digit(token: &str) -> Option<HotkeyKey> {
    let bytes = token.as_bytes();
    if bytes.len() == 4
        && token.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("key"))
        && let Some(&last) = bytes.get(3)
        && last.is_ascii_alphabetic()
    {
        return Some(HotkeyKey::Char(last.to_ascii_uppercase() as char));
    }

    if bytes.len() == 6
        && token.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("digit"))
        && let Some(&last) = bytes.get(5)
        && last.is_ascii_digit()
    {
        return Some(HotkeyKey::Char(last as char));
    }
    None
}

#[inline]
fn rule_named_key_registry(token: &str) -> Option<HotkeyKey> {
    if token.len() < 2 {
        return None;
    }
    for spec in NAMED_KEY_REGISTRY {
        if spec.aliases.iter().any(|alias| token.eq_ignore_ascii_case(alias)) {
            return Some(spec.key);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HotkeyRegistrationStatus {
    #[default]
    NotRegistered,
    RegisteredActive {
        shortcut: Arc<str>,
    },
    ConflictWithOtherApp {
        shortcut: Arc<str>,
    },
    UnsupportedByDisplayServer {
        hint_command: Arc<str>,
    },
    InvalidShortcut {
        raw: Arc<str>,
    },
}

impl HotkeyRegistrationStatus {
    #[inline]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::RegisteredActive { .. })
    }

    #[inline]
    pub const fn is_conflict(&self) -> bool {
        matches!(self, Self::ConflictWithOtherApp { .. })
    }

    #[inline]
    pub const fn is_unsupported_display(&self) -> bool {
        matches!(self, Self::UnsupportedByDisplayServer { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HotkeyPortError {
    #[error(transparent)]
    InvalidShortcut(#[from] HotkeyParseError),
    #[error("Комбинация клавиш `{0}` уже занята другим приложением")]
    Conflict(String),
    #[error("Глобальные хоткеи ограничены композитором экрана: используйте `{0}`")]
    UnsupportedDisplayServer(String),
    #[error("Ошибка системного менеджера горячих клавиш: {0}")]
    PlatformError(String),
}

pub trait GlobalHotkeyPort: Send + Sync {
    fn status(&self) -> HotkeyRegistrationStatus;

    fn register_shortcut(
        &self,
        shortcut: HotkeyShortcut,
    ) -> Result<HotkeyRegistrationStatus, HotkeyPortError>;

    fn register_str(
        &self,
        raw_shortcut: &str,
    ) -> Result<HotkeyRegistrationStatus, HotkeyPortError> {
        let parsed = HotkeyShortcut::parse(raw_shortcut)?;
        self.register_shortcut(parsed)
    }

    fn rebind_str(&self, raw_shortcut: &str) -> Result<HotkeyRegistrationStatus, HotkeyPortError> {
        let parsed = HotkeyShortcut::parse(raw_shortcut)?;
        self.register_shortcut(parsed)
    }

    fn unregister(&self) -> Result<(), HotkeyPortError>;

    fn try_recv_hotkey_trigger(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::{HotkeyKey, HotkeyModifiers, HotkeyParseError, HotkeyShortcut};

    #[test]
    fn parses_and_normalizes_modifier_order_without_case_sensitivity() {
        let s1 = HotkeyShortcut::parse("shift + ctrl + keyc").unwrap();
        let s2 = HotkeyShortcut::parse("Ctrl+Shift+C").unwrap();
        assert_eq!(s1, s2);
        assert_eq!(s1.to_canonical_string(), "Ctrl+Shift+C");
        assert_eq!(s1.key, HotkeyKey::Char('C'));
        assert_eq!(s1.key.canonical_str(), "C");
        assert!(s1.modifiers.ctrl());
        assert!(s1.modifiers.shift());
        assert!(!s1.modifiers.alt());
    }

    #[test]
    fn rejects_invalid_shortcuts_cleanly() {
        assert_eq!(HotkeyShortcut::parse(""), Err(HotkeyParseError::EmptyInput));
        assert_eq!(HotkeyShortcut::parse("C"), Err(HotkeyParseError::MissingModifier));
        assert_eq!(HotkeyShortcut::parse("Ctrl+Shift"), Err(HotkeyParseError::MissingKey));
        assert_eq!(HotkeyShortcut::parse("Ctrl+A+B"), Err(HotkeyParseError::MultipleKeys));
        assert_eq!(HotkeyShortcut::parse("Ctrl+Unknown"), Err(HotkeyParseError::UnknownToken));
    }

    #[test]
    fn supports_function_keys_navigation_keys_and_custom_chain() {
        let hk = HotkeyShortcut::parse("Cmd+Alt+F12").unwrap();
        assert_eq!(
            hk,
            HotkeyShortcut::new(
                HotkeyModifiers::SUPER.with(HotkeyModifiers::ALT),
                HotkeyKey::F(12)
            )
        );
        assert_eq!(hk.to_canonical_string(), "Alt+Super+F12");

        let nav = HotkeyShortcut::parse("Ctrl+Shift+PageUp").unwrap();
        assert_eq!(nav.key, HotkeyKey::PageUp);
        assert_eq!(nav.to_canonical_string(), "Ctrl+Shift+PageUp");
    }
}
