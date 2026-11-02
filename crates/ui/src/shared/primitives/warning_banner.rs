use std::fmt;
use std::sync::Arc;

use core::preset::models::resolution::ConfigWarning;
use core::shared_kernel::execution::anomaly::Anomaly;
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::{AppLocale, UiLabels};
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{ThemeTokens, active_theme, format_bytes_human};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerSeverity {
    SecurityCritical,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarningBannerItem {
    pub severity: BannerSeverity,
    pub title: String,
    pub detail: String,
}

impl WarningBannerItem {
    #[inline]
    pub fn from_config_warning(warning: &ConfigWarning) -> Self {
        Self::from_config_warning_localized(warning, AppLocale::Ru)
    }

    pub fn from_config_warning_localized(warning: &ConfigWarning, locale: AppLocale) -> Self {
        let labels = locale.labels();
        match warning {
            ConfigWarning::UntrustedSecretUnlockIgnored { blocked_paths } => Self {
                severity: BannerSeverity::SecurityCritical,
                title: labels.banner_secret_unlock_blocked_title.to_owned(),
                detail: format!(
                    "{} {}. {}",
                    labels.banner_secret_unlock_blocked_detail_prefix,
                    blocked_paths.join(", "),
                    labels.banner_secret_unlock_blocked_detail_suffix,
                ),
            },
            ConfigWarning::CorruptedGlobalConfig { path, reason } => Self {
                severity: BannerSeverity::Warning,
                title: labels.banner_corrupted_global_cfg_title.to_owned(),
                detail: format!("{path}: {reason}"),
            },
            ConfigWarning::CorruptedProjectConfig { path, reason } => Self {
                severity: BannerSeverity::Warning,
                title: labels.banner_corrupted_project_cfg_title.to_owned(),
                detail: format!("{path}: {reason}"),
            },
            ConfigWarning::OversizedConfigFile { path, size_bytes, max_bytes } => Self {
                severity: BannerSeverity::Warning,
                title: labels.banner_oversized_cfg_title.to_owned(),
                detail: format!(
                    "{path}: {} > {}",
                    format_bytes_human(*size_bytes),
                    format_bytes_human(*max_bytes)
                ),
            },
            ConfigWarning::ParameterClamped { field, original, clamped_to } => Self {
                severity: BannerSeverity::Warning,
                title: format!("{}: `{field}`", labels.banner_param_clamped_title),
                detail: format!("{original} → {clamped_to}"),
            },
            ConfigWarning::UnknownBasePreset { requested, fallback } => Self {
                severity: BannerSeverity::Warning,
                title: format!("{}: `{requested}`", labels.banner_unknown_base_preset_title),
                detail: format!("→ `{fallback}`"),
            },
            ConfigWarning::NewerSchemaVersion { file_version, supported_version } => Self {
                severity: BannerSeverity::Warning,
                title: labels.banner_newer_schema_title.to_owned(),
                detail: format!("v{file_version} (v{supported_version})"),
            },
        }
    }

    #[inline]
    pub fn from_anomaly(anomaly: &Anomaly) -> Self {
        Self::from_anomaly_localized(anomaly, AppLocale::Ru)
    }

    pub fn from_anomaly_localized(anomaly: &Anomaly, locale: AppLocale) -> Self {
        let labels = locale.labels();
        match anomaly {
            Anomaly::SecretBlocked { rel_path, reason } => {
                let compact_path = crate::theme::format_project_path_smart(rel_path, 24);
                Self {
                    severity: BannerSeverity::SecurityCritical,
                    title: format!("{} `{compact_path}`", labels.banner_secret_blocked_title),
                    detail: reason.clone(),
                }
            }
            Anomaly::LargeFileWarning { rel_path, size_bytes, threshold_bytes } => {
                let compact_path = crate::theme::format_project_path_smart(rel_path, 24);
                Self {
                    severity: BannerSeverity::Warning,
                    title: format!("{} `{compact_path}`", labels.banner_large_file_title),
                    detail: format!(
                        "{} > {}",
                        format_bytes_human(*size_bytes),
                        format_bytes_human(*threshold_bytes)
                    ),
                }
            }
            Anomaly::MaxDepthExceeded { rel_path, depth, max_depth } => {
                let compact_path = crate::theme::format_project_path_smart(rel_path, 24);
                Self {
                    severity: BannerSeverity::Warning,
                    title: format!("{} ({depth} > {max_depth})", labels.banner_max_depth_title),
                    detail: compact_path,
                }
            }
            Anomaly::MaxFilesExceeded { discovered_files, max_files } => Self {
                severity: BannerSeverity::Warning,
                title: labels.banner_max_files_title.to_owned(),
                detail: format!("{discovered_files} > {max_files}"),
            },
            Anomaly::CyclicOrSkippedSymlink { rel_path } => {
                let compact_path = crate::theme::format_project_path_smart(rel_path, 24);
                Self {
                    severity: BannerSeverity::Warning,
                    title: labels.banner_symlink_skipped_title.to_owned(),
                    detail: compact_path,
                }
            }
            Anomaly::UnreadableOrInvalidUtf8 { rel_path, reason } => {
                let compact_path = crate::theme::format_project_path_smart(rel_path, 24);
                Self {
                    severity: BannerSeverity::Warning,
                    title: format!("{} `{compact_path}`", labels.banner_unreadable_file_title),
                    detail: reason.clone(),
                }
            }
        }
    }
}

pub type DismissBannerCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct WarningBanner {
    items: Vec<WarningBannerItem>,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_dismiss: Option<DismissBannerCallback>,
}

impl fmt::Debug for WarningBanner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WarningBanner").field("items", &self.items).finish_non_exhaustive()
    }
}

impl WarningBanner {
    #[inline]
    pub fn new(items: Vec<WarningBannerItem>) -> Self {
        Self { items, labels: AppLocale::Ru.labels(), theme: active_theme(), on_dismiss: None }
    }

    #[must_use]
    #[inline]
    pub const fn with_context(
        mut self,
        labels: &'static UiLabels,
        theme: &'static ThemeTokens,
    ) -> Self {
        self.labels = labels;
        self.theme = theme;
        self
    }

    #[must_use]
    #[inline]
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for WarningBanner {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        if self.items.is_empty() {
            return div();
        }

        let colors = &self.theme.colors;
        let has_critical =
            self.items.iter().any(|i| i.severity == BannerSeverity::SecurityCritical);
        let tone = if has_critical { colors.banner_critical } else { colors.banner_warning };
        let first = self.items.first();

        let summary_title = first.map_or_else(
            || SharedString::from("⚠"),
            |item| SharedString::from(format!("⚠ {} ({})", item.title, self.items.len())),
        );
        let summary_detail =
            first.map(|item| SharedString::from(item.detail.clone())).unwrap_or_default();

        let mut header_row = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(self.theme.spacing.sm))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_xs()
                    .text_color(tone.fg)
                    .child(summary_title),
            );

        if let Some(dismiss_fn) = self.on_dismiss {
            let hover_bg = colors.bg_hover;
            header_row = header_row.child(
                div()
                    .id("dismiss-warning-banner")
                    .flex_shrink_0()
                    .p(px(self.theme.spacing.xs))
                    .rounded(px(self.theme.radius.sm))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(move |ev, win, cx| dismiss_fn(ev, win, cx))
                    .child(render_icon(UiIcon::X, 13.0, colors.text_secondary)),
            );
        }

        div()
            .absolute()
            .bottom(px(self.theme.spacing.xxl))
            .right(px(self.theme.spacing.xxl))
            .w(px(400.0))
            .max_w(px(400.0))
            .overflow_hidden()
            .px(px(self.theme.spacing.xl))
            .py(px(self.theme.spacing.md))
            .rounded(px(self.theme.radius.lg))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(tone.border)
            .flex()
            .flex_col()
            .gap(px(self.theme.spacing.xs))
            .child(header_row)
            .child(
                div()
                    .w_full()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_xs()
                    .text_color(colors.text_primary)
                    .child(summary_detail),
            )
    }
}
