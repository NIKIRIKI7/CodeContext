use std::borrow::Cow;

use gpui::{Rgba, SharedString, Styled as _, Svg, px, svg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiIcon {
    Package,
    FileDiff,
    Settings,
    SlidersHorizontal,
    Folder,
    FolderOpen,
    FileCode,
    FileText,
    Lock,
    Unlock,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    ChevronsUpDown,
    ChevronsDownUp,
    Check,
    Minus,
    Search,
    X,
    Copy,
    ClipboardPaste,
    RotateCcw,
    Info,
    Keyboard,
}

impl UiIcon {
    #[inline]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Package => "icons/package.svg",
            Self::FileDiff => "icons/file-diff.svg",
            Self::Settings => "icons/settings.svg",
            Self::SlidersHorizontal => "icons/sliders-horizontal.svg",
            Self::Folder => "icons/folder.svg",
            Self::FolderOpen => "icons/folder-open.svg",
            Self::FileCode => "icons/file-code.svg",
            Self::FileText => "icons/file-text.svg",
            Self::Lock => "icons/lock.svg",
            Self::Unlock => "icons/unlock.svg",
            Self::ChevronRight => "icons/chevron-right.svg",
            Self::ChevronDown => "icons/chevron-down.svg",
            Self::ChevronUp => "icons/chevron-up.svg",
            Self::ChevronsUpDown => "icons/chevrons-up-down.svg",
            Self::ChevronsDownUp => "icons/chevrons-down-up.svg",
            Self::Check => "icons/check.svg",
            Self::Minus => "icons/minus.svg",
            Self::Search => "icons/search.svg",
            Self::X => "icons/x.svg",
            Self::Copy => "icons/copy.svg",
            Self::ClipboardPaste => "icons/clipboard-paste.svg",
            Self::RotateCcw => "icons/rotate-ccw.svg",
            Self::Info => "icons/info.svg",
            Self::Keyboard => "icons/keyboard.svg",
        }
    }
}

#[inline]
pub fn render_icon(icon: UiIcon, size_px: f32, color: Rgba) -> Svg {
    svg().path(icon.path()).size(px(size_px)).text_color(color).flex_shrink_0()
}

#[inline]
pub fn render_vector_checkbox(
    checked: bool,
    indeterminate: bool,
    size_px: f32,
    background: Rgba,
    border: Rgba,
    foreground: Rgba,
    radius_px: f32,
) -> gpui::Div {
    let base = gpui::div()
        .flex_shrink_0()
        .w(px(size_px))
        .h(px(size_px))
        .rounded(px(radius_px))
        .border_1()
        .border_color(border)
        .bg(background)
        .flex()
        .items_center()
        .justify_center();

    if checked {
        gpui::ParentElement::child(base, render_icon(UiIcon::Check, size_px - 3.0, foreground))
    } else if indeterminate {
        gpui::ParentElement::child(base, render_icon(UiIcon::Minus, size_px - 3.0, foreground))
    } else {
        base
    }
}

pub fn load_icon_bytes(path: &str) -> Option<Cow<'static, [u8]>> {
    let svg_str = match path {
        "icons/package.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z"/><path d="M12 22V12"/><path d="m3.3 7 7.703 4.734a2 2 0 0 0 1.994 0L20.7 7"/><path d="m7.5 4.27 9 5.15"/></svg>"#
        }
        "icons/file-diff.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M9 10h6"/><path d="M12 7v6"/><path d="M9 17h6"/></svg>"#
        }
        "icons/settings.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>"#
        }
        "icons/sliders-horizontal.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><line x1="21" x2="14" y1="4" y2="4"/><line x1="10" x2="3" y1="4" y2="4"/><line x1="21" x2="12" y1="12" y2="12"/><line x1="8" x2="3" y1="12" y2="12"/><line x1="21" x2="16" y1="20" y2="20"/><line x1="12" x2="3" y1="20" y2="20"/><line x1="14" x2="14" y1="2" y2="6"/><line x1="8" x2="8" y1="10" y2="14"/><line x1="16" x2="16" y1="18" y2="22"/></svg>"#
        }
        "icons/folder.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>"#
        }
        "icons/folder-open.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/></svg>"#
        }
        "icons/file-code.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="m10 13-2 2 2 2"/><path d="m14 17 2-2-2-2"/></svg>"#
        }
        "icons/file-text.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/></svg>"#
        }
        "icons/lock.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>"#
        }
        "icons/unlock.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 9.9-1"/></svg>"#
        }
        "icons/chevron-right.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>"#
        }
        "icons/chevron-down.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>"#
        }
        "icons/chevron-up.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 15-6-6-6 6"/></svg>"#
        }
        "icons/chevrons-up-down.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="m7 15 5 5 5-5"/><path d="m7 9 5-5 5 5"/></svg>"#
        }
        "icons/chevrons-down-up.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="m7 20 5-5 5 5"/><path d="m7 4 5 5 5-5"/></svg>"#
        }
        "icons/check.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>"#
        }
        "icons/minus.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/></svg>"#
        }
        "icons/search.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>"#
        }
        "icons/x.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#
        }
        "icons/copy.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>"#
        }
        "icons/clipboard-paste.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H9a1 1 0 0 0-1 1v2c0 .6.4 1 1 1h6c.6 0 1-.4 1-1V3a1 1 0 0 0-1-1Z"/><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2M16 4h2a2 2 0 0 1 2 2v2M11 14h10"/><path d="m17 10 4 4-4 4"/></svg>"#
        }
        "icons/rotate-ccw.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>"#
        }
        "icons/info.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>"#
        }
        "icons/keyboard.svg" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M10 8h.01"/><path d="M12 12h.01"/><path d="M14 8h.01"/><path d="M16 12h.01"/><path d="M18 8h.01"/><path d="M6 8h.01"/><path d="M7 16h10"/><path d="M8 12h.01"/><rect width="20" height="16" x="2" y="4" rx="2"/></svg>"#
        }
        _ => return None,
    };
    Some(Cow::Borrowed(svg_str.as_bytes()))
}

#[inline]
pub fn list_icon_paths() -> Vec<SharedString> {
    vec![
        SharedString::from(UiIcon::Package.path()),
        SharedString::from(UiIcon::FileDiff.path()),
        SharedString::from(UiIcon::Settings.path()),
        SharedString::from(UiIcon::SlidersHorizontal.path()),
        SharedString::from(UiIcon::Folder.path()),
        SharedString::from(UiIcon::FolderOpen.path()),
        SharedString::from(UiIcon::FileCode.path()),
        SharedString::from(UiIcon::FileText.path()),
        SharedString::from(UiIcon::Lock.path()),
        SharedString::from(UiIcon::Unlock.path()),
        SharedString::from(UiIcon::ChevronRight.path()),
        SharedString::from(UiIcon::ChevronDown.path()),
        SharedString::from(UiIcon::ChevronUp.path()),
        SharedString::from(UiIcon::ChevronsUpDown.path()),
        SharedString::from(UiIcon::ChevronsDownUp.path()),
        SharedString::from(UiIcon::Check.path()),
        SharedString::from(UiIcon::Minus.path()),
        SharedString::from(UiIcon::Search.path()),
        SharedString::from(UiIcon::X.path()),
        SharedString::from(UiIcon::Copy.path()),
        SharedString::from(UiIcon::ClipboardPaste.path()),
        SharedString::from(UiIcon::RotateCcw.path()),
        SharedString::from(UiIcon::Info.path()),
        SharedString::from(UiIcon::Keyboard.path()),
    ]
}
