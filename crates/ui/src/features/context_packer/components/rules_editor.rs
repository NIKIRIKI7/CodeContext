use std::fmt;
use std::sync::Arc;

use core::preset::models::preset::Preset;
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::containers::form_controls::{render_checkbox_switch_row, render_segmented_tab};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon};
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
    active_prompts_count: usize,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_toggle_rule: Option<RuleToggleCallback>,
    on_select_chunk_limit: Option<ChunkLimitSelectCallback>,
    on_toggle_popover: Option<PopoverToggleCallback>,
    on_open_prompts_modal: Option<PopoverToggleCallback>,
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
            active_prompts_count: 0,
            labels,
            theme: active_theme(),
            on_toggle_rule: None,
            on_select_chunk_limit: None,
            on_toggle_popover: None,
            on_open_prompts_modal: None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_prompts_count(mut self, active_prompts_count: usize) -> Self {
        self.active_prompts_count = active_prompts_count;
        self
    }

    #[must_use]
    #[inline]
    pub fn on_open_prompts_modal(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_prompts_modal = Some(Arc::new(handler));
        self
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

        let trigger_label =
            SharedString::from(format!("{} ({active_count})", labels.rules_short_title));

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
            Some(CHUNK_PRESET_512_KB) => SharedString::from(format!(
                "{}: {}",
                labels.chunk_selector_prefix, labels.chunk_preset_512k
            )),
            Some(CHUNK_PRESET_1_MB) => SharedString::from(format!(
                "{}: {}",
                labels.chunk_selector_prefix, labels.chunk_preset_1m
            )),
            Some(CHUNK_PRESET_2_MB) => SharedString::from(format!(
                "{}: {}",
                labels.chunk_selector_prefix, labels.chunk_preset_2m
            )),
            Some(custom) => SharedString::from(format_bytes_human(custom as u64)),
        };

        let mut volume_hint = div()
            .id("btn-volume-summary-hint")
            .flex_shrink_0()
            .whitespace_nowrap()
            .cursor_pointer()
            .font_family(FONT_FAMILY_MONO)
            .text_xs()
            .text_color(theme.colors.text_muted)
            .child(volume_summary);

        if let Some(ref cb) = self.on_toggle_popover {
            let cb_clone = Arc::clone(cb);
            volume_hint = volume_hint.on_click(move |ev, w, c| cb_clone(ev, w, c));
        }

        let prompts_label = if self.active_prompts_count > 0 {
            SharedString::from(format!(
                "{} ({active_prompts_count})",
                labels.btn_footer_prompts,
                active_prompts_count = self.active_prompts_count
            ))
        } else {
            SharedString::from(labels.btn_footer_prompts)
        };
        let mut prompts_btn = UiButton::new("btn-footer-open-prompts-modal", prompts_label)
            .icon(UiIcon::FileText)
            .variant(ButtonVariant::FilterPill)
            .with_theme(theme)
            .active(self.active_prompts_count > 0);
        if let Some(ref cb) = self.on_open_prompts_modal {
            let cb_clone = Arc::clone(cb);
            prompts_btn = prompts_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
        }

        let left_buttons = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xs))
            .child(trigger_btn)
            .child(prompts_btn);

        let inspector_drawer = if self.popover_open {
            self.render_categorized_filter_inspector().into_any_element()
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
                    .child(left_buttons)
                    .child(volume_hint),
            )
    }
}

impl RulesEditorComponent {
    fn render_categorized_filter_inspector(&self) -> impl IntoElement + use<> {
        let theme = self.theme;
        let labels = self.labels;
        let colors = &theme.colors;
        let on_toggle = self.on_toggle_rule.as_ref();

        let fs_items: &[(&str, &str, bool, RuleToggleKind)] = &[
            (
                "rule-gitignore",
                labels.rule_gitignore,
                self.preset.respect_gitignore,
                RuleToggleKind::RespectGitignore,
            ),
            (
                "rule-symlinks",
                labels.rule_symlinks,
                self.preset.follow_symlinks,
                RuleToggleKind::FollowSymlinks,
            ),
        ];

        let transform_items: &[(&str, &str, bool, RuleToggleKind)] = &[
            (
                "rule-minify",
                labels.rule_minify,
                self.preset.minify_code,
                RuleToggleKind::MinifyCode,
            ),
            (
                "rule-crlf",
                labels.rule_crlf_to_lf,
                self.preset.normalize_line_endings,
                RuleToggleKind::NormalizeLineEndings,
            ),
            (
                "rule-tree",
                labels.rule_ascii_tree,
                self.preset.template.include_file_tree,
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
                    .pb(px(theme.spacing.xxs))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(theme.spacing.xs))
                    .child(render_icon(UiIcon::Settings, 13.0, colors.text_secondary))
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_primary)
                            .child(labels.rules_header),
                    ),
            )
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.rules_section_fs),
            );

        for &(id, label, is_on, kind) in fs_items {
            container = container
                .child(render_inspector_switch_row(id, label, is_on, kind, theme, on_toggle));
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
            container = container
                .child(render_inspector_switch_row(id, label, is_on, kind, theme, on_toggle));
        }

        let chunks_row = render_chunk_size_selector(
            self.preset.template.max_chunk_bytes,
            labels,
            theme,
            self.on_select_chunk_limit.as_ref(),
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
}

fn render_inspector_switch_row(
    id: &'static str,
    label: &'static str,
    is_on: bool,
    kind: RuleToggleKind,
    theme: &'static ThemeTokens,
    on_toggle: Option<&RuleToggleCallback>,
) -> impl IntoElement + use<> {
    let mut row = render_checkbox_switch_row(id, label, is_on, theme);
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
        (labels.chunk_preset_512k, Some(CHUNK_PRESET_512_KB)),
        (labels.chunk_preset_1m, Some(CHUNK_PRESET_1_MB)),
        (labels.chunk_preset_2m, Some(CHUNK_PRESET_2_MB)),
    ];

    let mut row = div()
        .w_full()
        .h(px(32.0))
        .p(px(2.0))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for (idx, &(label, limit_opt)) in presets.iter().enumerate() {
        let is_selected = current_limit == limit_opt;
        let cb_opt = on_select.cloned();
        row = row.child(render_segmented_tab(
            ("chunk-limit-opt", idx),
            label,
            is_selected,
            true,
            true,
            theme,
            move |_ev, win, cx| {
                if let Some(ref cb) = cb_opt {
                    cb(limit_opt, win, cx);
                }
            },
        ));
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
