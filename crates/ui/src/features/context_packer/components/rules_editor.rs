use std::fmt;
use std::sync::Arc;

use core::preset::models::preset::Preset;
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_vector_checkbox};
use crate::theme::{
    FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, active_theme, format_bytes_human,
};

pub const CHUNK_PRESET_512_KB: usize = 524_288;
pub const CHUNK_PRESET_1_MB: usize = 1_048_576;
pub const CHUNK_PRESET_2_MB: usize = 2_097_152;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleToggleKind {
    RespectGitignore,
    MinifyCode,
    NormalizeLineEndings,
    IncludeFileTree,
    FollowSymlinks,
}

pub type RuleToggleCallback = Arc<dyn Fn(RuleToggleKind, &mut Window, &mut App) + 'static>;
pub type ChunkLimitSelectCallback = Arc<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;
pub type PopoverToggleCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct RulesEditorComponent {
    preset: Preset,
    popover_open: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_toggle_rule: Option<RuleToggleCallback>,
    on_select_chunk_limit: Option<ChunkLimitSelectCallback>,
    on_toggle_popover: Option<PopoverToggleCallback>,
}

impl fmt::Debug for RulesEditorComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RulesEditorComponent")
            .field("preset", &self.preset)
            .field("popover_open", &self.popover_open)
            .finish_non_exhaustive()
    }
}

impl RulesEditorComponent {
    #[inline]
    pub const fn new(preset: Preset, labels: &'static UiLabels) -> Self {
        Self {
            preset,
            popover_open: false,
            labels,
            theme: active_theme(),
            on_toggle_rule: None,
            on_select_chunk_limit: None,
            on_toggle_popover: None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_theme(mut self, theme: &'static ThemeTokens) -> Self {
        self.theme = theme;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_popover_open(mut self, open: bool) -> Self {
        self.popover_open = open;
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_rule(
        mut self,
        handler: impl Fn(RuleToggleKind, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_rule = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_select_chunk_limit(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_chunk_limit = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_popover(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_popover = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for RulesEditorComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let labels = self.labels;

        let active_count = usize::from(self.preset.respect_gitignore)
            + usize::from(self.preset.minify_code)
            + usize::from(self.preset.normalize_line_endings)
            + usize::from(self.preset.template.include_file_tree)
            + usize::from(self.preset.follow_symlinks);

        let trigger_label = SharedString::from(format!("{} ({active_count})", labels.rules_header));

        let mut trigger_btn = UiButton::new("btn-toggle-filters-inspector", trigger_label)
            .icon(UiIcon::SlidersHorizontal)
            .variant(ButtonVariant::FilterPill)
            .with_theme(theme)
            .active(self.popover_open);

        if let Some(ref cb) = self.on_toggle_popover {
            let cb_clone = Arc::clone(cb);
            trigger_btn = trigger_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
        }

        let volume_summary = match self.preset.template.max_chunk_bytes {
            None => SharedString::from(labels.chunk_single_volume),
            Some(CHUNK_PRESET_512_KB) => {
                SharedString::from(format!("{}: 512K", labels.chunk_selector_prefix))
            }
            Some(CHUNK_PRESET_1_MB) => {
                SharedString::from(format!("{}: 1M", labels.chunk_selector_prefix))
            }
            Some(CHUNK_PRESET_2_MB) => {
                SharedString::from(format!("{}: 2M", labels.chunk_selector_prefix))
            }
            Some(custom) => SharedString::from(format_bytes_human(custom as u64)),
        };

        let mut volume_hint = div()
            .id("btn-volume-summary-hint")
            .cursor_pointer()
            .font_family(FONT_FAMILY_MONO)
            .text_xs()
            .text_color(theme.colors.text_muted)
            .child(volume_summary);

        if let Some(cb) = self.on_toggle_popover {
            volume_hint = volume_hint.on_click(move |ev, w, c| cb(ev, w, c));
        }

        let inspector_drawer = if self.popover_open {
            render_categorized_filter_inspector(
                &self.preset,
                labels,
                theme,
                self.on_toggle_rule.as_ref(),
                self.on_select_chunk_limit.as_ref(),
            )
            .into_any_element()
        } else {
            div().into_any_element()
        };

        div()
            .w_full()
            .bg(theme.colors.bg_panel)
            .border_t_1()
            .border_color(theme.colors.border)
            .flex()
            .flex_col()
            .child(inspector_drawer)
            .child(
                div()
                    .w_full()
                    .h(px(36.0))
                    .px(px(theme.spacing.lg))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.sm))
                    .child(trigger_btn)
                    .child(volume_hint),
            )
    }
}

fn render_categorized_filter_inspector(
    preset: &Preset,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_toggle: Option<&RuleToggleCallback>,
    on_select_chunk_limit: Option<&ChunkLimitSelectCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let fs_items: &[(&str, &str, bool, RuleToggleKind)] = &[
        (
            "rule-gitignore",
            labels.rule_gitignore,
            preset.respect_gitignore,
            RuleToggleKind::RespectGitignore,
        ),
        (
            "rule-symlinks",
            labels.rule_symlinks,
            preset.follow_symlinks,
            RuleToggleKind::FollowSymlinks,
        ),
    ];

    let transform_items: &[(&str, &str, bool, RuleToggleKind)] = &[
        ("rule-minify", labels.rule_minify, preset.minify_code, RuleToggleKind::MinifyCode),
        (
            "rule-crlf",
            labels.rule_crlf_to_lf,
            preset.normalize_line_endings,
            RuleToggleKind::NormalizeLineEndings,
        ),
        (
            "rule-tree",
            labels.rule_ascii_tree,
            preset.template.include_file_tree,
            RuleToggleKind::IncludeFileTree,
        ),
    ];

    let mut container = div()
        .w_full()
        .px(px(theme.spacing.lg))
        .py(px(theme.spacing.md))
        .bg(colors.bg_elevated)
        .border_b_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.xs))
        .child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_muted)
                .child(labels.rules_section_fs),
        );

    for &(id, label, is_on, kind) in fs_items {
        container =
            container.child(render_inspector_switch_row(id, label, is_on, kind, theme, on_toggle));
    }

    container = container.child(
        div()
            .pt(px(theme.spacing.xs))
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(colors.text_muted)
            .child(labels.rules_section_transform),
    );

    for &(id, label, is_on, kind) in transform_items {
        container =
            container.child(render_inspector_switch_row(id, label, is_on, kind, theme, on_toggle));
    }

    let chunks_row = render_chunk_size_selector(
        preset.template.max_chunk_bytes,
        labels,
        theme,
        on_select_chunk_limit,
    );

    container
        .child(
            div()
                .pt(px(theme.spacing.xs))
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_muted)
                .child(labels.rules_section_volumes),
        )
        .child(chunks_row)
}

fn render_inspector_switch_row(
    id: &'static str,
    label: &'static str,
    is_on: bool,
    kind: RuleToggleKind,
    theme: &'static ThemeTokens,
    on_toggle: Option<&RuleToggleCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let (check_background, check_border, check_foreground) = if is_on {
        (colors.badge_accent.bg, colors.cyan, colors.cyan)
    } else {
        (colors.bg_elevated, colors.border_active, colors.text_muted)
    };

    let mut row = div()
        .id(id)
        .w_full()
        .h(px(theme.layout.control_height))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_panel)
        .border_1()
        .border_color(if is_on { colors.border_active } else { colors.border })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(if is_on { colors.text_primary } else { colors.text_secondary })
                .child(label),
        )
        .child(render_vector_checkbox(
            is_on,
            false,
            14.0,
            check_background,
            check_border,
            check_foreground,
            theme.radius.sm,
        ));

    if let Some(cb) = on_toggle {
        let cb_clone = Arc::clone(cb);
        row = row.on_click(move |_ev, win, cx| cb_clone(kind, win, cx));
    }
    row
}

fn render_chunk_size_selector(
    current_limit: Option<usize>,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_select: Option<&ChunkLimitSelectCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let presets: &[(&str, Option<usize>)] = &[
        (labels.chunk_single_volume, None),
        ("512K", Some(CHUNK_PRESET_512_KB)),
        ("1M", Some(CHUNK_PRESET_1_MB)),
        ("2M", Some(CHUNK_PRESET_2_MB)),
    ];

    let mut row = div()
        .w_full()
        .h(px(32.0))
        .p(px(2.0))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_panel)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for (idx, &(label, limit_opt)) in presets.iter().enumerate() {
        let is_selected = current_limit == limit_opt;
        let hover_bg = colors.bg_hover;
        let mut seg = div()
            .id(("chunk-limit-opt", idx))
            .flex_1()
            .h_full()
            .rounded(px(theme.radius.sm))
            .border_1()
            .border_color(if is_selected { colors.border_active } else { colors.bg_panel })
            .bg(if is_selected { colors.bg_elevated } else { colors.bg_panel })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_MONO)
            .text_xs()
            .text_color(if is_selected { colors.text_primary } else { colors.text_secondary })
            .child(label);

        if let Some(cb) = on_select {
            let cb_clone = Arc::clone(cb);
            seg = seg.on_click(move |_ev, win, cx| cb_clone(limit_opt, win, cx));
        }
        row = row.child(seg);
    }

    if let Some(custom) = current_limit
        && !presets.iter().any(|&(_, p)| p == Some(custom))
    {
        row = row.child(
            Badge::new(format_bytes_human(custom as u64), BadgeVariant::Accent).with_theme(theme),
        );
    }

    row
}
