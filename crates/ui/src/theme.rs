use std::sync::OnceLock;

use core::shared_kernel::fs::file_kind::FileKind;
use gpui::Rgba;
use serde::Deserialize;

pub const FONT_FAMILY_UI: &str = "Inter";
pub const FONT_FAMILY_MONO: &str = "JetBrains Mono";
pub const EMBEDDED_DESIGN_SYSTEM_JSON: &str = include_str!("../assets/design_system.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformShortcuts {
    pub open_project: &'static str,
    pub rescan_index: &'static str,
    pub filter_files: &'static str,
    pub copy_context: &'static str,
    pub paste_clipboard: &'static str,
    pub apply_changes: &'static str,
    pub undo_changes: &'static str,
}

pub const WINDOWS_LINUX_SHORTCUTS: PlatformShortcuts = PlatformShortcuts {
    open_project: "Ctrl+O",
    rescan_index: "Ctrl+R",
    filter_files: "Ctrl+F",
    copy_context: "Ctrl+Shift+C",
    paste_clipboard: "Ctrl+V",
    apply_changes: "Ctrl+Enter",
    undo_changes: "Ctrl+Z",
};

pub const MACOS_SHORTCUTS: PlatformShortcuts = PlatformShortcuts {
    open_project: "⌘O",
    rescan_index: "⌘R",
    filter_files: "⌘F",
    copy_context: "⌘⇧C",
    paste_clipboard: "⌘V",
    apply_changes: "⌘↵",
    undo_changes: "⌘Z",
};

impl PlatformShortcuts {
    #[inline]
    pub const fn current() -> &'static Self {
        if cfg!(target_os = "macos") { &MACOS_SHORTCUTS } else { &WINDOWS_LINUX_SHORTCUTS }
    }
}

#[inline]
pub const fn platform_shortcuts() -> &'static PlatformShortcuts {
    PlatformShortcuts::current()
}

pub fn format_hotkey_for_current_os(canonical: &str) -> String {
    if !cfg!(target_os = "macos") {
        return canonical.replace("Super+", "Win+");
    }
    canonical
        .replace("Ctrl+", "⌃")
        .replace("Alt+", "⌥")
        .replace("Shift+", "⇧")
        .replace("Super+", "⌘")
}

#[inline]
pub const fn const_rgb(hex: u32) -> Rgba {
    let [_, r, g, b] = hex.to_be_bytes();
    Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    #[default]
    Dark,
    Light,
}

impl ThemeVariant {
    #[must_use]
    #[inline]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    #[inline]
    pub fn tokens(self) -> &'static ThemeTokens {
        let (dark, light) = loaded_themes();
        match self {
            Self::Dark => dark,
            Self::Light => light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceTone {
    pub bg: Rgba,
    pub fg: Rgba,
    pub border: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorTokens {
    pub bg_app: Rgba,
    pub bg_panel: Rgba,
    pub bg_elevated: Rgba,
    pub bg_hover: Rgba,
    pub bg_status_bar: Rgba,
    pub border: Rgba,
    pub border_active: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub cyan: Rgba,
    pub cyan_hover: Rgba,
    pub text_on_cyan: Rgba,
    pub text_on_accent: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub kind_rust: Rgba,
    pub kind_config: Rgba,
    pub badge_neutral: SurfaceTone,
    pub badge_accent: SurfaceTone,
    pub badge_success: SurfaceTone,
    pub badge_warning: SurfaceTone,
    pub badge_danger: SurfaceTone,
    pub button_danger: SurfaceTone,
    pub button_success: SurfaceTone,
    pub banner_critical: SurfaceTone,
    pub banner_warning: SurfaceTone,
}

impl ColorTokens {
    #[inline]
    pub const fn color_for_file_kind(&self, kind: FileKind) -> Rgba {
        match kind {
            FileKind::SourceCode => self.kind_rust,
            FileKind::Configuration => self.warning,
            FileKind::Documentation => self.cyan,
            FileKind::Secret => self.danger,
            FileKind::Binary | FileKind::SymlinkSkipped => self.text_muted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SpacingTokens {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct RadiusTokens {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct LayoutTokens {
    pub window_default_width: f32,
    pub window_default_height: f32,
    pub window_min_width: f32,
    pub window_min_height: f32,
    pub top_bar_height: f32,
    pub pane_header_height: f32,
    pub status_bar_height: f32,
    pub sub_footer_height: f32,
    #[serde(default = "default_control_height")]
    pub control_height: f32,
    #[serde(default = "default_control_height_sm")]
    pub control_height_sm: f32,
    pub tree_row_height: f32,
    pub tree_indent_step: f32,
    pub tree_indent_base: f32,
    pub tree_chevron_width: f32,
    pub tree_checkbox_size: f32,
    pub code_line_height: f32,
    pub line_number_width: f32,
    pub border_thickness: f32,
    pub packer_split_ratio: f32,
    pub applier_split_ratio: f32,
    pub split_ratio_min: f32,
    pub split_ratio_max: f32,
}

const fn default_control_height() -> f32 {
    28.0
}

const fn default_control_height_sm() -> f32 {
    24.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeTokens {
    pub colors: ColorTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub layout: LayoutTokens,
}

pub const DEFAULT_SPACING: SpacingTokens =
    SpacingTokens { xxs: 2.0, xs: 4.0, sm: 6.0, md: 8.0, lg: 10.0, xl: 12.0, xxl: 16.0 };

pub const DEFAULT_RADIUS: RadiusTokens = RadiusTokens { sm: 3.0, md: 4.0, lg: 6.0 };

pub const DEFAULT_LAYOUT: LayoutTokens = LayoutTokens {
    window_default_width: 1180.0,
    window_default_height: 720.0,
    window_min_width: 920.0,
    window_min_height: 560.0,
    top_bar_height: 40.0,
    pane_header_height: 40.0,
    status_bar_height: 24.0,
    sub_footer_height: 28.0,
    control_height: 28.0,
    control_height_sm: 24.0,
    tree_row_height: 24.0,
    tree_indent_step: 12.0,
    tree_indent_base: 10.0,
    tree_chevron_width: 14.0,
    tree_checkbox_size: 14.0,
    code_line_height: 20.0,
    line_number_width: 44.0,
    border_thickness: 1.0,
    packer_split_ratio: 0.36,
    applier_split_ratio: 0.34,
    split_ratio_min: 0.20,
    split_ratio_max: 0.80,
};

pub const DARK_THEME_TOKENS: ThemeTokens = ThemeTokens {
    colors: ColorTokens {
        bg_app: const_rgb(0x000e_1116),
        bg_panel: const_rgb(0x0014_1820),
        bg_elevated: const_rgb(0x001c_222d),
        bg_hover: const_rgb(0x0025_2d3b),
        bg_status_bar: const_rgb(0x000a_0d12),
        border: const_rgb(0x0026_2e3d),
        border_active: const_rgb(0x0043_5168),
        text_primary: const_rgb(0x00ed_f2f7),
        text_secondary: const_rgb(0x00a0_aec0),
        text_muted: const_rgb(0x0064_748b),
        accent: const_rgb(0x0025_63eb),
        accent_hover: const_rgb(0x003b_82f6),
        cyan: const_rgb(0x003b_82f6),
        cyan_hover: const_rgb(0x0060_a5fa),
        text_on_cyan: const_rgb(0x00ff_ffff),
        text_on_accent: const_rgb(0x00ff_ffff),
        success: const_rgb(0x0010_b981),
        warning: const_rgb(0x00f5_9e0b),
        danger: const_rgb(0x00ef_4444),
        kind_rust: const_rgb(0x00e0_8250),
        kind_config: const_rgb(0x0094_a3b8),
        badge_neutral: SurfaceTone {
            bg: const_rgb(0x001c_222d),
            fg: const_rgb(0x00a0_aec0),
            border: const_rgb(0x002d_3748),
        },
        badge_accent: SurfaceTone {
            bg: const_rgb(0x0017_253b),
            fg: const_rgb(0x0093_c5fd),
            border: const_rgb(0x0029_446b),
        },
        badge_success: SurfaceTone {
            bg: const_rgb(0x0006_2e22),
            fg: const_rgb(0x0034_d399),
            border: const_rgb(0x000f_5c42),
        },
        badge_warning: SurfaceTone {
            bg: const_rgb(0x0033_2207),
            fg: const_rgb(0x00fb_bf24),
            border: const_rgb(0x0066_440e),
        },
        badge_danger: SurfaceTone {
            bg: const_rgb(0x003b_0d11),
            fg: const_rgb(0x00ff_b4ab),
            border: const_rgb(0x007d_1d24),
        },
        button_danger: SurfaceTone {
            bg: const_rgb(0x007f_1d1d),
            fg: const_rgb(0x00ff_ffff),
            border: const_rgb(0x00ef_4444),
        },
        button_success: SurfaceTone {
            bg: const_rgb(0x0005_9669),
            fg: const_rgb(0x00ff_ffff),
            border: const_rgb(0x0010_b981),
        },
        banner_critical: SurfaceTone {
            bg: const_rgb(0x002d_090d),
            fg: const_rgb(0x00ff_b4ab),
            border: const_rgb(0x00ef_4444),
        },
        banner_warning: SurfaceTone {
            bg: const_rgb(0x002b_1d06),
            fg: const_rgb(0x00fb_bf24),
            border: const_rgb(0x00d2_9922),
        },
    },
    spacing: DEFAULT_SPACING,
    radius: DEFAULT_RADIUS,
    layout: DEFAULT_LAYOUT,
};

pub const LIGHT_THEME_TOKENS: ThemeTokens = ThemeTokens {
    colors: ColorTokens {
        bg_app: const_rgb(0x00f8_fafc),
        bg_panel: const_rgb(0x00f1_f5f9),
        bg_elevated: const_rgb(0x00e2_e8f0),
        bg_hover: const_rgb(0x00cb_d5e1),
        bg_status_bar: const_rgb(0x00e2_e8f0),
        border: const_rgb(0x00cb_d5e1),
        border_active: const_rgb(0x0064_748b),
        text_primary: const_rgb(0x000f_172a),
        text_secondary: const_rgb(0x0033_4155),
        text_muted: const_rgb(0x0064_748b),
        accent: const_rgb(0x0025_63eb),
        accent_hover: const_rgb(0x001d_4ed8),
        cyan: const_rgb(0x0002_84c7),
        cyan_hover: const_rgb(0x0003_69a1),
        text_on_cyan: const_rgb(0x00ff_ffff),
        text_on_accent: const_rgb(0x00ff_ffff),
        success: const_rgb(0x0005_9669),
        warning: const_rgb(0x00d9_7706),
        danger: const_rgb(0x00dc_2626),
        kind_rust: const_rgb(0x00ea_580c),
        kind_config: const_rgb(0x007e_22ce),
        badge_neutral: SurfaceTone {
            bg: const_rgb(0x00e2_e8f0),
            fg: const_rgb(0x0033_4155),
            border: const_rgb(0x00cb_d5e1),
        },
        badge_accent: SurfaceTone {
            bg: const_rgb(0x00e0_f2fe),
            fg: const_rgb(0x0003_69a1),
            border: const_rgb(0x007d_d3fc),
        },
        badge_success: SurfaceTone {
            bg: const_rgb(0x00d1_fae5),
            fg: const_rgb(0x0006_5f46),
            border: const_rgb(0x006e_e7b7),
        },
        badge_warning: SurfaceTone {
            bg: const_rgb(0x00fe_f3c7),
            fg: const_rgb(0x0092_400e),
            border: const_rgb(0x00fc_d34d),
        },
        badge_danger: SurfaceTone {
            bg: const_rgb(0x00fe_e2e2),
            fg: const_rgb(0x0099_1b1b),
            border: const_rgb(0x00fc_a5a5),
        },
        button_danger: SurfaceTone {
            bg: const_rgb(0x00dc_2626),
            fg: const_rgb(0x00ff_ffff),
            border: const_rgb(0x0099_1b1b),
        },
        button_success: SurfaceTone {
            bg: const_rgb(0x0005_9669),
            fg: const_rgb(0x00ff_ffff),
            border: const_rgb(0x0004_7857),
        },
        banner_critical: SurfaceTone {
            bg: const_rgb(0x00fe_e2e2),
            fg: const_rgb(0x0099_1b1b),
            border: const_rgb(0x00dc_2626),
        },
        banner_warning: SurfaceTone {
            bg: const_rgb(0x00fe_f3c7),
            fg: const_rgb(0x0092_400e),
            border: const_rgb(0x00d9_7706),
        },
    },
    spacing: DEFAULT_SPACING,
    radius: DEFAULT_RADIUS,
    layout: DEFAULT_LAYOUT,
};

#[derive(Debug, Clone, Deserialize)]
struct SurfaceToneDto {
    bg: String,
    fg: String,
    border: String,
}

impl SurfaceToneDto {
    fn to_tokens(&self, fallback: SurfaceTone) -> SurfaceTone {
        SurfaceTone {
            bg: parse_hex_color(&self.bg).unwrap_or(fallback.bg),
            fg: parse_hex_color(&self.fg).unwrap_or(fallback.fg),
            border: parse_hex_color(&self.border).unwrap_or(fallback.border),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ColorTokensDto {
    bg_app: String,
    bg_panel: String,
    bg_elevated: String,
    bg_hover: String,
    bg_status_bar: String,
    border: String,
    border_active: String,
    text_primary: String,
    text_secondary: String,
    text_muted: String,
    accent: String,
    accent_hover: String,
    cyan: String,
    cyan_hover: String,
    text_on_cyan: String,
    text_on_accent: String,
    success: String,
    warning: String,
    danger: String,
    kind_rust: String,
    kind_config: String,
    badge_neutral: SurfaceToneDto,
    badge_accent: SurfaceToneDto,
    badge_success: SurfaceToneDto,
    badge_warning: SurfaceToneDto,
    badge_danger: SurfaceToneDto,
    button_danger: SurfaceToneDto,
    button_success: SurfaceToneDto,
    banner_critical: SurfaceToneDto,
    banner_warning: SurfaceToneDto,
}

impl ColorTokensDto {
    #[allow(clippy::too_many_lines)]
    fn to_tokens(&self, fb: &ColorTokens) -> ColorTokens {
        ColorTokens {
            bg_app: parse_hex_color(&self.bg_app).unwrap_or(fb.bg_app),
            bg_panel: parse_hex_color(&self.bg_panel).unwrap_or(fb.bg_panel),
            bg_elevated: parse_hex_color(&self.bg_elevated).unwrap_or(fb.bg_elevated),
            bg_hover: parse_hex_color(&self.bg_hover).unwrap_or(fb.bg_hover),
            bg_status_bar: parse_hex_color(&self.bg_status_bar).unwrap_or(fb.bg_status_bar),
            border: parse_hex_color(&self.border).unwrap_or(fb.border),
            border_active: parse_hex_color(&self.border_active).unwrap_or(fb.border_active),
            text_primary: parse_hex_color(&self.text_primary).unwrap_or(fb.text_primary),
            text_secondary: parse_hex_color(&self.text_secondary).unwrap_or(fb.text_secondary),
            text_muted: parse_hex_color(&self.text_muted).unwrap_or(fb.text_muted),
            accent: parse_hex_color(&self.accent).unwrap_or(fb.accent),
            accent_hover: parse_hex_color(&self.accent_hover).unwrap_or(fb.accent_hover),
            cyan: parse_hex_color(&self.cyan).unwrap_or(fb.cyan),
            cyan_hover: parse_hex_color(&self.cyan_hover).unwrap_or(fb.cyan_hover),
            text_on_cyan: parse_hex_color(&self.text_on_cyan).unwrap_or(fb.text_on_cyan),
            text_on_accent: parse_hex_color(&self.text_on_accent).unwrap_or(fb.text_on_accent),
            success: parse_hex_color(&self.success).unwrap_or(fb.success),
            warning: parse_hex_color(&self.warning).unwrap_or(fb.warning),
            danger: parse_hex_color(&self.danger).unwrap_or(fb.danger),
            kind_rust: parse_hex_color(&self.kind_rust).unwrap_or(fb.kind_rust),
            kind_config: parse_hex_color(&self.kind_config).unwrap_or(fb.kind_config),
            badge_neutral: self.badge_neutral.to_tokens(fb.badge_neutral),
            badge_accent: self.badge_accent.to_tokens(fb.badge_accent),
            badge_success: self.badge_success.to_tokens(fb.badge_success),
            badge_warning: self.badge_warning.to_tokens(fb.badge_warning),
            badge_danger: self.badge_danger.to_tokens(fb.badge_danger),
            button_danger: self.button_danger.to_tokens(fb.button_danger),
            button_success: self.button_success.to_tokens(fb.button_success),
            banner_critical: self.banner_critical.to_tokens(fb.banner_critical),
            banner_warning: self.banner_warning.to_tokens(fb.banner_warning),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DesignSystemJsonDto {
    spacing: SpacingTokens,
    radius: RadiusTokens,
    layout: LayoutTokens,
    dark: ColorTokensDto,
    light: ColorTokensDto,
}

fn parse_hex_color(raw: &str) -> Option<Rgba> {
    let hex = raw.trim().strip_prefix('#').unwrap_or(raw.trim());
    if hex.len() != 6 {
        return None;
    }
    let val = u32::from_str_radix(hex, 16).ok()?;
    Some(const_rgb(val))
}

static LOADED_DESIGN_SYSTEM: OnceLock<(ThemeTokens, ThemeTokens)> = OnceLock::new();

fn loaded_themes() -> &'static (ThemeTokens, ThemeTokens) {
    LOADED_DESIGN_SYSTEM.get_or_init(|| {
        let external_json = std::fs::read_to_string("crates/ui/assets/design_system.json")
            .or_else(|_| std::fs::read_to_string("assets/design_system.json"))
            .ok();
        let raw_json = external_json.as_deref().unwrap_or(EMBEDDED_DESIGN_SYSTEM_JSON);
        if let Ok(dto) = serde_json::from_str::<DesignSystemJsonDto>(raw_json) {
            let dark = ThemeTokens {
                colors: dto.dark.to_tokens(&DARK_THEME_TOKENS.colors),
                spacing: dto.spacing,
                radius: dto.radius,
                layout: dto.layout,
            };
            let light = ThemeTokens {
                colors: dto.light.to_tokens(&LIGHT_THEME_TOKENS.colors),
                spacing: dto.spacing,
                radius: dto.radius,
                layout: dto.layout,
            };
            (dark, light)
        } else {
            (DARK_THEME_TOKENS, LIGHT_THEME_TOKENS)
        }
    })
}

#[inline]
pub const fn active_theme() -> &'static ThemeTokens {
    &DARK_THEME_TOKENS
}

pub fn format_bytes_human(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * 1_024;
    if bytes >= MB {
        let whole = bytes / MB;
        let frac = ((bytes % MB) * 100) / MB;
        format!("{whole}.{frac:02} MB")
    } else if bytes >= KB {
        let whole = bytes / KB;
        let frac = ((bytes % KB) * 10) / KB;
        format!("{whole}.{frac} KB")
    } else {
        format!("{bytes} B")
    }
}

pub fn format_tokens_estimate(bytes: u64, suffix: &str) -> String {
    let approx_tokens = bytes / 4;
    if approx_tokens >= 1_000_000 {
        let m = approx_tokens / 1_000_000;
        let rem = (approx_tokens % 1_000_000) / 100_000;
        format!("~{m}.{rem}M {suffix}")
    } else if approx_tokens >= 1_000 {
        let k = approx_tokens / 1_000;
        format!("~{k}k {suffix}")
    } else {
        format!("~{approx_tokens} {suffix}")
    }
}

pub fn format_project_path_smart(path: &str, max_chars: usize) -> String {
    let char_count = path.chars().count();
    if char_count <= max_chars {
        return path.to_owned();
    }
    let segments: Vec<&str> = path.split(['/', '\\']).filter(|s| !s.is_empty()).collect();
    if segments.len() >= 2 {
        let sep = if path.contains('\\') { '\\' } else { '/' };
        let last = segments.last().copied().unwrap_or("");
        let prev = segments.get(segments.len() - 2).copied().unwrap_or("");
        let candidate = format!("…{sep}{prev}{sep}{last}");
        if candidate.chars().count() <= max_chars + 6 {
            return candidate;
        }
    }
    let keep = max_chars.saturating_sub(1).max(8);
    let tail: String = path.chars().skip(char_count.saturating_sub(keep)).collect();
    format!("…{tail}")
}
