use std::fmt;
use std::sync::Arc;

use core::preset::models::preset::Preset;
use core::preset::models::resolution::ConfigOrigin;
use gpui::{
    App, ClickEvent, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::containers::modal_dialog::ModalDialogShell;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{
    FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, format_bytes_human, format_tokens_estimate,
};

pub type MetaModalActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct ProjectMetaProps {
    pub project_path: String,
    pub origin: ConfigOrigin,
    pub preset: Preset,
    pub total_bytes: u64,
    pub selected_files: usize,
    pub total_indexed_files: usize,
    pub total_lines: usize,
    pub blocked_secrets_count: usize,
    pub skipped_binary_count: usize,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

#[derive(IntoElement)]
pub struct ProjectMetaModalComponent {
    props: ProjectMetaProps,
    on_save_ctxpack: Option<MetaModalActionCallback>,
    on_close: Option<MetaModalActionCallback>,
}

impl fmt::Debug for ProjectMetaModalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectMetaModalComponent")
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}

impl ProjectMetaModalComponent {
    #[inline]
    pub const fn new(props: ProjectMetaProps) -> Self {
        Self { props, on_save_ctxpack: None, on_close: None }
    }

    #[must_use]
    pub fn with_actions(
        mut self,
        on_save_ctxpack: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_save_ctxpack = Some(Arc::new(on_save_ctxpack));
        self.on_close = Some(Arc::new(on_close));
        self
    }

    fn render_metrics_grid(&self, theme: &'static ThemeTokens) -> impl IntoElement + use<> {
        let labels = self.props.labels;
        div()
            .w_full()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(px(theme.spacing.sm))
            .child(
                Badge::new(format_bytes_human(self.props.total_bytes), BadgeVariant::Accent)
                    .with_theme(theme),
            )
            .child(
                Badge::new(
                    format_tokens_estimate(self.props.total_bytes, labels.unit_tokens),
                    BadgeVariant::Accent,
                )
                .with_theme(theme),
            )
            .child(
                Badge::new(
                    format!(
                        "{} / {} {}",
                        self.props.selected_files,
                        self.props.total_indexed_files,
                        labels.unit_files
                    ),
                    BadgeVariant::Neutral,
                )
                .with_theme(theme),
            )
            .child(
                Badge::new(
                    format!("{} {}", self.props.total_lines, labels.unit_lines),
                    BadgeVariant::Neutral,
                )
                .with_theme(theme),
            )
            .child(
                Badge::new(
                    format!("{} {}", self.props.blocked_secrets_count, labels.unit_secrets_hidden),
                    if self.props.blocked_secrets_count > 0 {
                        BadgeVariant::Danger
                    } else {
                        BadgeVariant::Success
                    },
                )
                .with_theme(theme),
            )
            .child(
                Badge::new(
                    format!("{} {}", self.props.skipped_binary_count, labels.unit_binaries_skipped),
                    BadgeVariant::Warning,
                )
                .with_theme(theme),
            )
    }

    fn render_limits_box(&self, theme: &'static ThemeTokens) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = self.props.labels;
        let gitignore_status = if self.props.preset.respect_gitignore {
            labels.rules_active_suffix
        } else {
            labels.applier_backup_off
        };
        let excludes_str = if self.props.preset.exclude_globs.is_empty() {
            labels.exclude_globs_default.to_owned()
        } else {
            self.props.preset.exclude_globs.join(", ")
        };

        div()
            .w_full()
            .p(px(theme.spacing.lg))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xs))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.meta_section_limits),
            )
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(colors.text_primary)
                    .child(SharedString::from(format!(
                        ".gitignore: {gitignore_status} · {} ≤ {} · {} ≤ {}",
                        labels.meta_rule_depth,
                        self.props.preset.max_depth,
                        labels.meta_rule_files,
                        self.props.preset.max_files
                    ))),
            )
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(colors.text_secondary)
                    .child(SharedString::from(format!(
                        "{}: {excludes_str}",
                        labels.meta_rule_excludes
                    ))),
            )
    }
}

impl RenderOnce for ProjectMetaModalComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.props.theme;
        let colors = &theme.colors;
        let labels = self.props.labels;

        let (origin_label, origin_variant) = match self.props.origin {
            ConfigOrigin::BuiltInDefault => (labels.origin_default, BadgeVariant::Neutral),
            ConfigOrigin::GlobalUserConfig => (labels.origin_global, BadgeVariant::Accent),
            ConfigOrigin::LocalProjectOverride => {
                (labels.origin_local_project, BadgeVariant::Warning)
            }
        };

        let metrics_grid = self.render_metrics_grid(theme);
        let limits_box = self.render_limits_box(theme);

        let mut save_btn = UiButton::new("meta-save-ctxpack-btn", labels.btn_save_project_cfg)
            .variant(ButtonVariant::Primary)
            .with_theme(theme);
        if let Some(save_fn) = self.on_save_ctxpack {
            save_btn = save_btn.on_click(move |ev, w, c| save_fn(ev, w, c));
        }

        ModalDialogShell::new(
            "project-meta-modal-backdrop",
            "project-meta-modal-dialog",
            "meta-close-modal-btn",
            labels.meta_modal_title,
        )
        .width(580.0)
        .leading_icon(UiIcon::Info)
        .header_badge(Badge::new(origin_label, origin_variant).with_theme(theme))
        .with_theme(theme)
        .on_close_arc(self.on_close)
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_secondary)
                .child(SharedString::from(self.props.project_path)),
        )
        .child(metrics_grid)
        .child(limits_box)
        .child(div().w_full().flex().flex_row().justify_end().child(save_btn))
    }
}
