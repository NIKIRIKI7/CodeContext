use std::fmt;
use std::sync::Arc;

pub use core::preset::models::preset::{RegexTargetScope, RegexTemplateItem};
use core::preset::models::template::{PromptBlockItem, PromptCategory};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Rgba,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon, render_vector_checkbox};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtensionFilterMode {
    #[default]
    All,
    IncludeOnly,
    ExcludeSelected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionStatItem {
    pub ext: String,
    pub count: usize,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRuleItem {
    pub name: String,
    pub is_excluded: bool,
    pub is_project_folder: bool,
}

#[derive(Debug, Clone)]
pub struct FilterModalProps {
    pub ext_mode: ExtensionFilterMode,
    pub extensions: Vec<ExtensionStatItem>,
    pub extension_input_draft: String,
    pub extension_input_focused: bool,
    pub folders: Vec<FolderRuleItem>,
    pub folder_input_draft: String,
    pub folder_input_focused: bool,
    pub regex_scope: RegexTargetScope,
    pub regex_pattern: String,
    pub regex_focused: bool,
    pub regex_name_draft: String,
    pub regex_name_focused: bool,
    pub regex_case_sensitive: bool,
    pub regex_is_valid: bool,
    pub regex_templates: Vec<RegexTemplateItem>,
    pub active_rules_count: usize,
    pub selected_files_count: usize,
    pub total_files_count: usize,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

pub type FilterModalActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ExtModeSelectCallback = Arc<dyn Fn(ExtensionFilterMode, &mut Window, &mut App) + 'static>;
pub type StringSelectCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type RegexScopeSelectCallback = Arc<dyn Fn(RegexTargetScope, &mut Window, &mut App) + 'static>;
pub type RegexTemplateIndexCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

#[derive(Clone, Default)]
struct FilterModalCallbacks {
    close: Option<FilterModalActionCallback>,
    reset_all: Option<FilterModalActionCallback>,
    select_ext_mode: Option<ExtModeSelectCallback>,
    toggle_extension: Option<StringSelectCallback>,
    focus_extension_input: Option<FilterModalActionCallback>,
    submit_extension_input: Option<FilterModalActionCallback>,
    toggle_folder: Option<StringSelectCallback>,
    focus_folder_input: Option<FilterModalActionCallback>,
    submit_folder_input: Option<FilterModalActionCallback>,
    select_regex_scope: Option<RegexScopeSelectCallback>,
    focus_regex_input: Option<FilterModalActionCallback>,
    focus_regex_name_input: Option<FilterModalActionCallback>,
    submit_regex_template: Option<FilterModalActionCallback>,
    clear_regex_input: Option<FilterModalActionCallback>,
    toggle_regex_case: Option<FilterModalActionCallback>,
    apply_regex_preset: Option<StringSelectCallback>,
    apply_regex_template_idx: Option<RegexTemplateIndexCallback>,
    delete_regex_template_idx: Option<RegexTemplateIndexCallback>,
}

#[derive(IntoElement)]
pub struct FilterModalComponent {
    props: FilterModalProps,
    callbacks: FilterModalCallbacks,
}

impl fmt::Debug for FilterModalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterModalComponent")
            .field("active_rules_count", &self.props.active_rules_count)
            .finish_non_exhaustive()
    }
}

impl FilterModalComponent {
    #[inline]
    pub fn new(props: FilterModalProps) -> Self {
        Self { props, callbacks: FilterModalCallbacks::default() }
    }

    #[must_use]
    #[inline]
    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.close = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_reset_all(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.reset_all = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_select_ext_mode(
        mut self,
        handler: impl Fn(ExtensionFilterMode, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_ext_mode = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_extension(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_extension = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_focus_extension_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_extension_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_submit_extension_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.submit_extension_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_folder(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_folder = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_focus_folder_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_folder_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_submit_folder_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.submit_folder_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_select_regex_scope(
        mut self,
        handler: impl Fn(RegexTargetScope, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_regex_scope = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_focus_regex_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_regex_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_clear_regex_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.clear_regex_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_regex_case(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_regex_case = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_focus_regex_name_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_regex_name_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_submit_regex_template(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.submit_regex_template = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_apply_regex_preset(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.apply_regex_preset = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_apply_regex_template_idx(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.apply_regex_template_idx = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_delete_regex_template_idx(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.delete_regex_template_idx = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for FilterModalComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.props.theme;
        let colors = &theme.colors;
        let labels = self.props.labels;

        let header = render_modal_header(&self.props, &self.callbacks);
        let ext_section = render_extensions_section(&self.props, &self.callbacks);
        let folders_section = render_folders_section(&self.props, &self.callbacks);
        let regex_section = render_regex_section(&self.props, &self.callbacks);
        let footer = render_modal_footer(&self.props, labels, &self.callbacks);

        let backdrop_color = Rgba { r: 0.02, g: 0.03, b: 0.05, a: 0.72 };
        let mut backdrop = div()
            .id("filter-modal-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(backdrop_color)
            .flex()
            .items_center()
            .justify_center()
            .p(px(theme.spacing.xxl));

        if let Some(ref close_fn) = self.callbacks.close {
            let cb = Arc::clone(close_fn);
            backdrop = backdrop.on_click(move |ev, w, c| cb(ev, w, c));
        }

        backdrop.child(
            div()
                .id("filter-modal-dialog")
                .on_click(|_ev, _win, cx| {
                    cx.stop_propagation();
                })
                .w(px(620.0))
                .p(px(theme.spacing.xxl))
                .rounded(px(theme.radius.lg))
                .bg(colors.bg_panel)
                .border_1()
                .border_color(colors.border_active)
                .flex()
                .flex_col()
                .gap(px(theme.spacing.lg))
                .child(header)
                .child(ext_section)
                .child(folders_section)
                .child(regex_section)
                .child(footer),
        )
    }
}

fn render_modal_header(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let mut close_btn = UiButton::icon_only("filter-modal-close-btn", UiIcon::X)
        .variant(ButtonVariant::Ghost)
        .with_theme(theme);
    if let Some(ref close_fn) = callbacks.close {
        let cb = Arc::clone(close_fn);
        close_btn = close_btn.on_click(move |ev, w, c| cb(ev, w, c));
    }

    let badge_variant =
        if props.active_rules_count > 0 { BadgeVariant::Accent } else { BadgeVariant::Neutral };

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.sm))
                .child(render_icon(UiIcon::SlidersHorizontal, 15.0, colors.cyan))
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_sm()
                        .text_color(colors.text_primary)
                        .child(labels.filter_modal_title),
                )
                .child(
                    Badge::new(
                        format!("{} {}", props.active_rules_count, labels.rules_active_suffix),
                        badge_variant,
                    )
                    .with_theme(theme),
                ),
        )
        .child(close_btn)
}

fn render_extensions_section(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let mode_tabs: &[(&str, &str, ExtensionFilterMode)] = &[
        ("ext-mode-all", labels.filter_ext_mode_all, ExtensionFilterMode::All),
        ("ext-mode-include", labels.filter_ext_mode_include, ExtensionFilterMode::IncludeOnly),
        ("ext-mode-exclude", labels.filter_ext_mode_exclude, ExtensionFilterMode::ExcludeSelected),
    ];

    let mut mode_bar = div()
        .h(px(26.0))
        .p(px(2.0))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for &(id, text, mode) in mode_tabs {
        let is_active = props.ext_mode == mode;
        let hover_bg = colors.bg_hover;
        let mut tab = div()
            .id(id)
            .px(px(theme.spacing.md))
            .h_full()
            .rounded(px(theme.radius.sm))
            .bg(if is_active { colors.bg_elevated } else { colors.bg_app })
            .border_1()
            .border_color(if is_active { colors.border_active } else { colors.bg_app })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(if is_active { colors.text_primary } else { colors.text_secondary })
            .child(text);

        if let Some(ref cb) = callbacks.select_ext_mode {
            let cb_clone = Arc::clone(cb);
            tab = tab.on_click(move |_ev, w, c| cb_clone(mode, w, c));
        }
        mode_bar = mode_bar.child(tab);
    }

    let add_ext_row = render_manual_extension_input_row(props, callbacks);

    let mut grid = div()
        .id("filter-modal-extensions-scroll")
        .w_full()
        .max_h(px(96.0))
        .overflow_y_scroll()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(theme.spacing.xs));

    for (idx, item) in props.extensions.iter().enumerate() {
        grid = grid.child(render_extension_chip(idx, item, props.ext_mode, theme, callbacks));
    }

    div()
        .w_full()
        .p(px(theme.spacing.lg))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_elevated)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_primary)
                        .child(labels.filter_section_extensions),
                )
                .child(mode_bar),
        )
        .child(add_ext_row)
        .child(grid)
}

fn render_manual_extension_input_row(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_draft = !props.extension_input_draft.is_empty();

    let display_text = if props.extension_input_focused && has_draft {
        SharedString::from(format!("{}▏", props.extension_input_draft))
    } else if has_draft {
        SharedString::from(props.extension_input_draft.clone())
    } else if props.extension_input_focused {
        SharedString::from(format!("▏{}", labels.filter_ext_input_placeholder))
    } else {
        SharedString::from(labels.filter_ext_input_placeholder)
    };

    let mut input_box = div()
        .id("filter-modal-extension-input")
        .flex_1()
        .min_w(px(0.0))
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if props.extension_input_focused { colors.cyan } else { colors.border })
        .bg(colors.bg_app)
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(render_icon(UiIcon::FileCode, 13.0, colors.text_muted))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if has_draft || props.extension_input_focused {
                    colors.text_primary
                } else {
                    colors.text_muted
                })
                .child(display_text),
        );

    if let Some(ref cb) = callbacks.focus_extension_input {
        let cb_clone = Arc::clone(cb);
        input_box = input_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let mut add_btn = UiButton::new("filter-modal-add-ext-btn", labels.filter_ext_add_btn)
        .variant(if has_draft { ButtonVariant::Primary } else { ButtonVariant::Secondary })
        .with_theme(theme);
    if let Some(ref cb) = callbacks.submit_extension_input {
        let cb_clone = Arc::clone(cb);
        add_btn = add_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(input_box)
        .child(add_btn)
}

fn render_extension_chip(
    idx: usize,
    item: &ExtensionStatItem,
    mode: ExtensionFilterMode,
    theme: &'static ThemeTokens,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let is_selected = item.selected;

    let (border_col, bg_col, text_col, check_fg) = match (mode, is_selected) {
        (ExtensionFilterMode::ExcludeSelected, true) => (
            colors.badge_danger.border,
            colors.badge_danger.bg,
            colors.badge_danger.fg,
            colors.danger,
        ),
        (_, true) => (colors.cyan, colors.badge_accent.bg, colors.text_primary, colors.cyan),
        (_, false) => (colors.border, colors.bg_panel, colors.text_secondary, colors.text_muted),
    };

    let mut chip = div()
        .id(("filter-modal-ext-chip", idx))
        .h(px(26.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(border_col)
        .bg(bg_col)
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(render_vector_checkbox(
            is_selected,
            false,
            13.0,
            colors.bg_app,
            border_col,
            check_fg,
            theme.radius.sm,
        ))
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(text_col)
                .child(SharedString::from(format!(".{}", item.ext))),
        )
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_muted)
                .child(SharedString::from(format!("{}", item.count))),
        );

    if let Some(ref cb) = callbacks.toggle_extension {
        let cb_clone = Arc::clone(cb);
        let ext_owned = item.ext.clone();
        chip = chip.on_click(move |_ev, w, c| cb_clone(&ext_owned, w, c));
    }

    chip
}

fn render_folders_section(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let add_folder_row = render_manual_folder_input_row(props, callbacks);

    let mut folders_grid = div()
        .id("filter-modal-folders-scroll")
        .w_full()
        .max_h(px(84.0))
        .overflow_y_scroll()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(theme.spacing.xs));

    for (idx, folder) in props.folders.iter().enumerate() {
        let hover_bg = colors.bg_hover;
        let is_exc = folder.is_excluded;
        let (border_col, bg_col, fg_col) = if is_exc {
            (colors.badge_danger.border, colors.badge_danger.bg, colors.badge_danger.fg)
        } else {
            (colors.border, colors.bg_panel, colors.text_secondary)
        };

        let mut chip = div()
            .id(("filter-modal-folder-chip", idx))
            .h(px(26.0))
            .px(px(theme.spacing.md))
            .rounded(px(theme.radius.md))
            .border_1()
            .border_color(border_col)
            .bg(bg_col)
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xs))
            .child(render_icon(
                if is_exc { UiIcon::X } else { UiIcon::Folder },
                12.0,
                if is_exc { colors.danger } else { colors.text_muted },
            ))
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(fg_col)
                    .child(SharedString::from(folder.name.clone())),
            );

        if let Some(ref cb) = callbacks.toggle_folder {
            let cb_clone = Arc::clone(cb);
            let folder_name = folder.name.clone();
            chip = chip.on_click(move |_ev, w, c| cb_clone(&folder_name, w, c));
        }
        folders_grid = folders_grid.child(chip);
    }

    div()
        .w_full()
        .p(px(theme.spacing.lg))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_elevated)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_primary)
                        .child(labels.filter_section_folders),
                )
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_muted)
                        .child(labels.filter_folders_hint),
                ),
        )
        .child(add_folder_row)
        .child(folders_grid)
}

fn render_manual_folder_input_row(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_draft = !props.folder_input_draft.is_empty();

    let display_text = if props.folder_input_focused && has_draft {
        SharedString::from(format!("{}▏", props.folder_input_draft))
    } else if has_draft {
        SharedString::from(props.folder_input_draft.clone())
    } else if props.folder_input_focused {
        SharedString::from(format!("▏{}", labels.filter_folder_input_placeholder))
    } else {
        SharedString::from(labels.filter_folder_input_placeholder)
    };

    let mut input_box = div()
        .id("filter-modal-folder-input")
        .flex_1()
        .min_w(px(0.0))
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if props.folder_input_focused { colors.cyan } else { colors.border })
        .bg(colors.bg_app)
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(render_icon(UiIcon::Folder, 13.0, colors.text_muted))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if has_draft || props.folder_input_focused {
                    colors.text_primary
                } else {
                    colors.text_muted
                })
                .child(display_text),
        );

    if let Some(ref cb) = callbacks.focus_folder_input {
        let cb_clone = Arc::clone(cb);
        input_box = input_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let mut add_btn = UiButton::new("filter-modal-add-folder-btn", labels.filter_folder_add_btn)
        .variant(if has_draft { ButtonVariant::Primary } else { ButtonVariant::Secondary })
        .with_theme(theme);
    if let Some(ref cb) = callbacks.submit_folder_input {
        let cb_clone = Arc::clone(cb);
        add_btn = add_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(input_box)
        .child(add_btn)
}

fn render_regex_section(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let scope_bar = render_regex_scope_bar(props, callbacks);
    let input_row = render_regex_input_row(props, callbacks);
    let presets_row = render_regex_presets_row(props, callbacks);

    div()
        .w_full()
        .p(px(theme.spacing.lg))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_elevated)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_primary)
                        .child(labels.filter_section_regex),
                )
                .child(scope_bar),
        )
        .child(input_row)
        .child(presets_row)
}

fn render_regex_scope_bar(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let scopes: &[(&str, &str, RegexTargetScope)] = &[
        ("re-scope-f-exc", labels.filter_regex_files_exclude, RegexTargetScope::FilesExclude),
        ("re-scope-f-inc", labels.filter_regex_files_include, RegexTargetScope::FilesInclude),
        ("re-scope-d-exc", labels.filter_regex_folders_exclude, RegexTargetScope::FoldersExclude),
        ("re-scope-d-inc", labels.filter_regex_folders_include, RegexTargetScope::FoldersInclude),
    ];

    let mut bar = div()
        .h(px(26.0))
        .p(px(2.0))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for &(id, text, scope) in scopes {
        let is_active = props.regex_scope == scope;
        let hover_bg = colors.bg_hover;
        let mut item = div()
            .id(id)
            .px(px(theme.spacing.sm))
            .h_full()
            .rounded(px(theme.radius.sm))
            .bg(if is_active { colors.bg_elevated } else { colors.bg_app })
            .border_1()
            .border_color(if is_active { colors.border_active } else { colors.bg_app })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(if is_active { colors.text_primary } else { colors.text_secondary })
            .child(text);

        if let Some(ref cb) = callbacks.select_regex_scope {
            let cb_clone = Arc::clone(cb);
            item = item.on_click(move |_ev, w, c| cb_clone(scope, w, c));
        }
        bar = bar.child(item);
    }

    bar
}

fn render_regex_input_row(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let labels = props.labels;
    let input_box = render_regex_pattern_box(props, callbacks);
    let name_box = render_regex_template_name_box(props, callbacks);
    let can_save = !props.regex_pattern.is_empty() && props.regex_is_valid;

    let mut add_btn = UiButton::new("filter-modal-add-regex-btn", labels.filter_regex_add_btn)
        .variant(if can_save { ButtonVariant::Primary } else { ButtonVariant::Secondary })
        .disabled(!can_save)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.submit_regex_template {
        let cb_clone = Arc::clone(cb);
        add_btn = add_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(input_box)
        .child(name_box)
        .child(add_btn)
}

fn render_regex_pattern_box(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_pattern = !props.regex_pattern.is_empty();

    let display_text = if props.regex_focused && has_pattern {
        SharedString::from(format!("{}▏", props.regex_pattern))
    } else if has_pattern {
        SharedString::from(props.regex_pattern.clone())
    } else if props.regex_focused {
        SharedString::from(format!("▏{}", labels.tree_regex_placeholder))
    } else {
        SharedString::from(labels.tree_regex_placeholder)
    };

    let border_col = if !props.regex_is_valid {
        colors.danger
    } else if props.regex_focused {
        colors.cyan
    } else {
        colors.border
    };

    let clear_el = if has_pattern {
        let mut btn = div()
            .id("filter-modal-regex-clear")
            .p(px(2.0))
            .rounded(px(theme.radius.sm))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .child(render_icon(UiIcon::X, 12.0, colors.text_secondary));
        if let Some(ref cb) = callbacks.clear_regex_input {
            let cb_clone = Arc::clone(cb);
            btn = btn.on_click(move |ev, w, c| {
                c.stop_propagation();
                cb_clone(ev, w, c);
            });
        }
        btn.into_any_element()
    } else {
        div().into_any_element()
    };

    let hover_bg = colors.bg_hover;
    let mut case_btn = div()
        .id("filter-modal-regex-case-btn")
        .h(px(22.0))
        .px(px(theme.spacing.sm))
        .rounded(px(theme.radius.sm))
        .border_1()
        .border_color(if props.regex_case_sensitive { colors.cyan } else { colors.border })
        .bg(if props.regex_case_sensitive { colors.badge_accent.bg } else { colors.bg_elevated })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_FAMILY_MONO)
        .text_xs()
        .text_color(if props.regex_case_sensitive { colors.cyan } else { colors.text_muted })
        .child("Aa");
    if let Some(ref cb) = callbacks.toggle_regex_case {
        let cb_clone = Arc::clone(cb);
        case_btn = case_btn.on_click(move |ev, w, c| {
            c.stop_propagation();
            cb_clone(ev, w, c);
        });
    }

    let mut input_box = div()
        .id("filter-modal-regex-input")
        .flex_1()
        .min_w(px(0.0))
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(border_col)
        .bg(colors.bg_app)
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.sm))
        .child(div().font_family(FONT_FAMILY_MONO).text_xs().text_color(colors.cyan).child("/"))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if has_pattern || props.regex_focused {
                    colors.text_primary
                } else {
                    colors.text_muted
                })
                .child(display_text),
        )
        .child(clear_el)
        .child(case_btn);

    if let Some(ref cb) = callbacks.focus_regex_input {
        let cb_clone = Arc::clone(cb);
        input_box = input_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    input_box
}

fn render_regex_template_name_box(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_name = !props.regex_name_draft.is_empty();

    let name_display = if props.regex_name_focused && has_name {
        SharedString::from(format!("{}▏", props.regex_name_draft))
    } else if has_name {
        SharedString::from(props.regex_name_draft.clone())
    } else if props.regex_name_focused {
        SharedString::from(format!("▏{}", labels.filter_regex_name_placeholder))
    } else {
        SharedString::from(labels.filter_regex_name_placeholder)
    };

    let mut name_box = div()
        .id("filter-modal-regex-name-input")
        .w(px(148.0))
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if props.regex_name_focused { colors.cyan } else { colors.border })
        .bg(colors.bg_app)
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .overflow_hidden()
        .whitespace_nowrap()
        .font_family(FONT_FAMILY_UI)
        .text_xs()
        .text_color(if has_name || props.regex_name_focused {
            colors.text_primary
        } else {
            colors.text_muted
        })
        .child(name_display);

    if let Some(ref cb) = callbacks.focus_regex_name_input {
        let cb_clone = Arc::clone(cb);
        name_box = name_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    name_box
}

fn render_regex_presets_row(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let is_english = props.labels.btn_copy_context == "Copy";

    let mut row = div()
        .id("filter-modal-regex-presets-scroll")
        .w_full()
        .max_h(px(64.0))
        .overflow_y_scroll()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(theme.spacing.xs));

    for (idx, item) in props.regex_templates.iter().enumerate() {
        row = row.child(render_regex_template_chip(idx, item, is_english, props, theme, callbacks));
    }

    row
}

fn render_regex_template_chip(
    idx: usize,
    item: &RegexTemplateItem,
    is_english: bool,
    props: &FilterModalProps,
    theme: &'static ThemeTokens,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let is_current = props.regex_pattern == item.pattern && props.regex_scope == item.scope;
    let hover_bg = colors.bg_hover;
    let title = SharedString::from(item.display_name(is_english).to_owned());

    let mut pill = div()
        .id(("re-preset-chip", idx))
        .h(px(24.0))
        .px(px(theme.spacing.sm))
        .rounded(px(theme.radius.sm))
        .border_1()
        .border_color(if is_current { colors.cyan } else { colors.border })
        .bg(if is_current { colors.badge_accent.bg } else { colors.bg_panel })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xxs))
        .font_family(FONT_FAMILY_UI)
        .text_xs()
        .text_color(if is_current { colors.cyan } else { colors.text_secondary })
        .child(title);

    if let Some(ref cb) = callbacks.apply_regex_template_idx {
        let cb_clone = Arc::clone(cb);
        pill = pill.on_click(move |_ev, w, c| cb_clone(idx, w, c));
    } else if let Some(ref cb) = callbacks.apply_regex_preset {
        let cb_clone = Arc::clone(cb);
        let pat_owned = item.pattern.clone();
        pill = pill.on_click(move |_ev, w, c| cb_clone(&pat_owned, w, c));
    }

    if item.is_custom
        && let Some(ref del_cb) = callbacks.delete_regex_template_idx
    {
        let cb_del = Arc::clone(del_cb);
        let del_btn = div()
            .id(("re-preset-del", idx))
            .p(px(2.0))
            .rounded(px(theme.radius.sm))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .on_click(move |_ev, w, c| {
                c.stop_propagation();
                cb_del(idx, w, c);
            })
            .child(render_icon(UiIcon::X, 10.0, colors.text_muted));
        pill = pill.child(del_btn);
    }

    pill
}

fn render_modal_footer(
    props: &FilterModalProps,
    labels: &'static UiLabels,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;

    let mut actions = div().flex().flex_row().items_center().gap(px(theme.spacing.sm));

    if props.active_rules_count > 0 {
        let mut reset_btn = UiButton::new("filter-modal-reset-btn", labels.btn_reset_filters)
            .icon(UiIcon::RotateCcw)
            .variant(ButtonVariant::Secondary)
            .with_theme(theme);
        if let Some(ref cb) = callbacks.reset_all {
            let cb_clone = Arc::clone(cb);
            reset_btn = reset_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
        }
        actions = actions.child(reset_btn);
    }

    let mut done_btn = UiButton::new("filter-modal-done-btn", labels.status_ready)
        .icon(UiIcon::Check)
        .variant(ButtonVariant::Primary)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.close {
        let cb_clone = Arc::clone(cb);
        done_btn = done_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }
    actions = actions.child(done_btn);

    div()
        .w_full()
        .pt(px(theme.spacing.xs))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div().font_family(FONT_FAMILY_MONO).text_xs().text_color(colors.text_secondary).child(
                SharedString::from(format!(
                    "{}: {} / {} {}",
                    labels.unit_selected,
                    props.selected_files_count,
                    props.total_files_count,
                    labels.unit_files
                )),
            ),
        )
        .child(actions)
}

pub type PromptIndexCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type PromptCategoryCallback =
    Arc<dyn Fn(Option<PromptCategory>, &mut Window, &mut App) + 'static>;
pub type NewPromptCategoryCallback = Arc<dyn Fn(PromptCategory, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct SystemPromptModalProps {
    pub active_category: Option<PromptCategory>,
    pub new_prompt_category: PromptCategory,
    pub prompt_title_draft: String,
    pub prompt_title_focused: bool,
    pub prompt_input_draft: String,
    pub prompt_input_focused: bool,
    pub blocks: Vec<PromptBlockItem>,
    pub effective_prompt_text: String,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

#[derive(Clone, Default)]
struct PromptModalCallbacks {
    select_category: Option<PromptCategoryCallback>,
    select_new_category: Option<NewPromptCategoryCallback>,
    toggle_block: Option<PromptIndexCallback>,
    move_up: Option<PromptIndexCallback>,
    move_down: Option<PromptIndexCallback>,
    delete_block: Option<PromptIndexCallback>,
    focus_prompt_title: Option<FilterModalActionCallback>,
    focus_prompt_input: Option<FilterModalActionCallback>,
    submit_prompt_input: Option<FilterModalActionCallback>,
    paste_custom: Option<FilterModalActionCallback>,
    copy_prompt: Option<FilterModalActionCallback>,
    reset_builtin: Option<FilterModalActionCallback>,
    close: Option<FilterModalActionCallback>,
}

#[derive(IntoElement)]
pub struct SystemPromptModalComponent {
    props: SystemPromptModalProps,
    callbacks: PromptModalCallbacks,
}

impl fmt::Debug for SystemPromptModalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemPromptModalComponent")
            .field("blocks_count", &self.props.blocks.len())
            .finish_non_exhaustive()
    }
}

impl SystemPromptModalComponent {
    #[inline]
    pub fn new(props: SystemPromptModalProps) -> Self {
        Self { props, callbacks: PromptModalCallbacks::default() }
    }

    #[must_use]
    #[inline]
    pub fn on_select_category(
        mut self,
        handler: impl Fn(Option<PromptCategory>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_category = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_select_new_category(
        mut self,
        handler: impl Fn(PromptCategory, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_new_category = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_block(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_block = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_move_up(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.callbacks.move_up = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_move_down(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.move_down = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_delete_block(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.delete_block = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_focus_prompt_title(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_prompt_title = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_focus_prompt_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_prompt_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_submit_prompt_input(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.submit_prompt_input = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn with_actions(
        mut self,
        on_paste_custom: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_copy_prompt: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_reset_builtin: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.paste_custom = Some(Arc::new(on_paste_custom));
        self.callbacks.copy_prompt = Some(Arc::new(on_copy_prompt));
        self.callbacks.reset_builtin = Some(Arc::new(on_reset_builtin));
        self.callbacks.close = Some(Arc::new(on_close));
        self
    }
}

impl RenderOnce for SystemPromptModalComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.props.theme;
        let colors = &theme.colors;
        let labels = self.props.labels;

        let composer_card = render_prompt_composer_card(&self.props, &self.callbacks);
        let cat_bar = render_prompt_category_bar(&self.props, &self.callbacks);
        let blocks_list = render_prompt_blocks_list(&self.props, &self.callbacks);

        let preview_text = if self.props.effective_prompt_text.is_empty() {
            SharedString::from(labels.prompt_preview_empty)
        } else {
            SharedString::from(self.props.effective_prompt_text.clone())
        };

        let preview_box = div()
            .id("system-prompt-preview-scroll")
            .w_full()
            .h(px(110.0))
            .overflow_y_scroll()
            .p(px(theme.spacing.md))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_app)
            .border_1()
            .border_color(colors.border)
            .font_family(FONT_FAMILY_MONO)
            .text_xs()
            .text_color(if self.props.effective_prompt_text.is_empty() {
                colors.text_muted
            } else {
                colors.text_primary
            })
            .child(preview_text);

        let any_enabled = self.props.blocks.iter().any(|b| b.enabled);
        let footer_row = render_prompt_modal_footer(
            any_enabled,
            labels,
            theme,
            self.callbacks.copy_prompt.as_ref(),
            self.callbacks.reset_builtin.as_ref(),
            self.callbacks.close.as_ref(),
        );

        let mut close_btn = UiButton::icon_only("prompt-modal-close-btn", UiIcon::X)
            .variant(ButtonVariant::Ghost)
            .with_theme(theme);
        if let Some(ref close_fn) = self.callbacks.close {
            let cb = Arc::clone(close_fn);
            close_btn = close_btn.on_click(move |ev, w, c| cb(ev, w, c));
        }

        let backdrop_color = Rgba { r: 0.02, g: 0.03, b: 0.05, a: 0.72 };
        let mut backdrop = div()
            .id("system-prompt-modal-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(backdrop_color)
            .flex()
            .items_center()
            .justify_center()
            .p(px(theme.spacing.xxl));

        if let Some(close_fn) = self.callbacks.close {
            backdrop = backdrop.on_click(move |ev, w, c| close_fn(ev, w, c));
        }

        backdrop.child(
            div()
                .id("system-prompt-modal-dialog")
                .on_click(|_ev, _win, cx| {
                    cx.stop_propagation();
                })
                .w(px(680.0))
                .p(px(theme.spacing.xxl))
                .rounded(px(theme.radius.lg))
                .bg(colors.bg_panel)
                .border_1()
                .border_color(colors.border_active)
                .flex()
                .flex_col()
                .gap(px(theme.spacing.sm))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_FAMILY_UI)
                                .text_sm()
                                .text_color(colors.text_primary)
                                .child(labels.prompt_modal_title),
                        )
                        .child(close_btn),
                )
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_secondary)
                        .child(labels.prompt_modal_subtitle),
                )
                .child(composer_card)
                .child(cat_bar)
                .child(blocks_list)
                .child(preview_box)
                .child(footer_row),
        )
    }
}

fn render_prompt_composer_card(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let cat_row = render_new_prompt_category_row(props, callbacks);
    let title_row = render_new_prompt_title_row(props, callbacks);
    let content_row = render_manual_prompt_input_row(props, callbacks);

    div()
        .w_full()
        .p(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_elevated)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.xs))
        .child(cat_row)
        .child(title_row)
        .child(content_row)
}

fn render_new_prompt_category_row(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let categories: &[(&str, &str, PromptCategory)] = &[
        ("new-p-cat-patcher", labels.prompt_cat_patcher, PromptCategory::Patcher),
        ("new-p-cat-arch", labels.prompt_cat_arch, PromptCategory::Architecture),
        ("new-p-cat-behavior", labels.prompt_cat_behavior, PromptCategory::Behavior),
        ("new-p-cat-custom", labels.prompt_cat_custom, PromptCategory::Custom),
    ];

    let mut bar = div()
        .flex_1()
        .h(px(26.0))
        .p(px(2.0))
        .rounded(px(theme.radius.sm))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for &(id, text, cat) in categories {
        let is_active = props.new_prompt_category == cat;
        let hover_bg = colors.bg_hover;
        let mut pill = div()
            .id(id)
            .flex_1()
            .h_full()
            .rounded(px(theme.radius.sm))
            .bg(if is_active { colors.bg_elevated } else { colors.bg_app })
            .border_1()
            .border_color(if is_active { colors.cyan } else { colors.bg_app })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(if is_active { colors.text_primary } else { colors.text_secondary })
            .child(text);

        if let Some(ref cb) = callbacks.select_new_category {
            let cb_clone = Arc::clone(cb);
            pill = pill.on_click(move |_ev, w, c| cb_clone(cat, w, c));
        }
        bar = bar.child(pill);
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .w(px(86.0))
                .flex_shrink_0()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_secondary)
                .child(labels.prompt_field_category_label),
        )
        .child(bar)
}

fn render_new_prompt_title_row(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_title = !props.prompt_title_draft.is_empty();

    let display_text = if props.prompt_title_focused && has_title {
        SharedString::from(format!("{}▏", props.prompt_title_draft))
    } else if has_title {
        SharedString::from(props.prompt_title_draft.clone())
    } else if props.prompt_title_focused {
        SharedString::from(format!("▏{}", labels.prompt_field_title_placeholder))
    } else {
        SharedString::from(labels.prompt_field_title_placeholder)
    };

    let mut title_box = div()
        .id("system-prompt-title-input")
        .flex_1()
        .min_w(px(0.0))
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if props.prompt_title_focused { colors.cyan } else { colors.border })
        .bg(colors.bg_app)
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .overflow_hidden()
        .whitespace_nowrap()
        .font_family(FONT_FAMILY_UI)
        .text_xs()
        .text_color(if has_title || props.prompt_title_focused {
            colors.text_primary
        } else {
            colors.text_muted
        })
        .child(display_text);

    if let Some(ref cb) = callbacks.focus_prompt_title {
        let cb_clone = Arc::clone(cb);
        title_box = title_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .w(px(86.0))
                .flex_shrink_0()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_secondary)
                .child(labels.prompt_field_title_label),
        )
        .child(title_box)
}

fn render_prompt_category_bar(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let categories: &[(&str, &str, Option<PromptCategory>)] = &[
        ("p-cat-all", labels.prompt_cat_all, None),
        ("p-cat-patcher", labels.prompt_cat_patcher, Some(PromptCategory::Patcher)),
        ("p-cat-arch", labels.prompt_cat_arch, Some(PromptCategory::Architecture)),
        ("p-cat-behavior", labels.prompt_cat_behavior, Some(PromptCategory::Behavior)),
        ("p-cat-custom", labels.prompt_cat_custom, Some(PromptCategory::Custom)),
    ];

    let mut bar = div()
        .w_full()
        .h(px(28.0))
        .p(px(2.0))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for &(id, text, cat_opt) in categories {
        let is_active = props.active_category == cat_opt;
        let hover_bg = colors.bg_hover;
        let mut tab = div()
            .id(id)
            .flex_1()
            .h_full()
            .rounded(px(theme.radius.sm))
            .bg(if is_active { colors.bg_elevated } else { colors.bg_app })
            .border_1()
            .border_color(if is_active { colors.cyan } else { colors.bg_app })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(if is_active { colors.text_primary } else { colors.text_secondary })
            .child(text);

        if let Some(ref cb) = callbacks.select_category {
            let cb_clone = Arc::clone(cb);
            tab = tab.on_click(move |_ev, w, c| cb_clone(cat_opt, w, c));
        }
        bar = bar.child(tab);
    }

    bar
}

fn render_manual_prompt_input_row(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_draft = !props.prompt_input_draft.is_empty();
    let can_submit = has_draft || !props.prompt_title_draft.is_empty();

    let display_text = if props.prompt_input_focused && has_draft {
        SharedString::from(format!("{}▏", props.prompt_input_draft))
    } else if has_draft {
        SharedString::from(props.prompt_input_draft.clone())
    } else if props.prompt_input_focused {
        SharedString::from(format!("▏{}", labels.prompt_add_input_placeholder))
    } else {
        SharedString::from(labels.prompt_add_input_placeholder)
    };

    let mut input_box = div()
        .id("system-prompt-modal-input")
        .flex_1()
        .min_w(px(0.0))
        .min_h(px(46.0))
        .max_h(px(76.0))
        .overflow_y_scroll()
        .px(px(theme.spacing.md))
        .py(px(theme.spacing.xs))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if props.prompt_input_focused { colors.cyan } else { colors.border })
        .bg(colors.bg_app)
        .cursor_pointer()
        .font_family(FONT_FAMILY_MONO)
        .text_xs()
        .text_color(if has_draft || props.prompt_input_focused {
            colors.text_primary
        } else {
            colors.text_muted
        })
        .child(display_text);

    if let Some(ref cb) = callbacks.focus_prompt_input {
        let cb_clone = Arc::clone(cb);
        input_box = input_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let mut paste_btn = UiButton::icon_only("prompt-modal-inline-paste", UiIcon::ClipboardPaste)
        .variant(ButtonVariant::Secondary)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.paste_custom {
        let cb_clone = Arc::clone(cb);
        paste_btn = paste_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let mut add_btn = UiButton::new("prompt-modal-submit-btn", labels.prompt_add_btn)
        .variant(if can_submit { ButtonVariant::Primary } else { ButtonVariant::Secondary })
        .disabled(!can_submit)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.submit_prompt_input {
        let cb_clone = Arc::clone(cb);
        add_btn = add_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_start()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .w(px(86.0))
                .pt(px(theme.spacing.xs))
                .flex_shrink_0()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_secondary)
                .child(labels.prompt_field_content_label),
        )
        .child(input_box)
        .child(div().flex().flex_col().gap(px(theme.spacing.xs)).child(add_btn).child(paste_btn))
}

fn render_prompt_blocks_list(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let total_len = props.blocks.len();

    let mut list = div()
        .id("system-prompt-blocks-scroll")
        .w_full()
        .max_h(px(148.0))
        .overflow_y_scroll()
        .p(px(theme.spacing.sm))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_elevated)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.xs));

    let mut active_order_num = 0_usize;
    let mut visible_count = 0_usize;
    for (idx, block) in props.blocks.iter().enumerate() {
        if block.enabled {
            active_order_num += 1;
        }
        if let Some(filter_cat) = props.active_category
            && block.category != filter_cat
        {
            continue;
        }
        visible_count += 1;
        let order_badge =
            if block.enabled { format!("#{active_order_num}") } else { String::new() };
        let cat_label = match block.category {
            PromptCategory::Patcher => labels.prompt_cat_patcher,
            PromptCategory::Architecture => labels.prompt_cat_arch,
            PromptCategory::Behavior => labels.prompt_cat_behavior,
            PromptCategory::Custom => labels.prompt_cat_custom,
        };
        list = list.child(render_prompt_block_row(
            idx,
            total_len,
            block,
            &order_badge,
            cat_label,
            theme,
            callbacks,
        ));
    }

    if visible_count == 0 {
        list = list.child(
            div()
                .w_full()
                .h(px(36.0))
                .flex()
                .items_center()
                .justify_center()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_muted)
                .child(labels.prompt_category_empty_hint),
        );
    }

    list
}

fn render_prompt_block_row(
    idx: usize,
    total_len: usize,
    block: &PromptBlockItem,
    order_badge: &str,
    cat_label: &'static str,
    theme: &'static ThemeTokens,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let is_on = block.enabled;

    let (box_bg, box_border, check_glyph) = if is_on {
        (colors.badge_accent.bg, colors.cyan, colors.cyan)
    } else {
        (colors.bg_app, colors.border_active, colors.text_muted)
    };

    let mut left_click_area = div()
        .id(("prompt-block-toggle", idx))
        .flex_1()
        .min_w(px(0.0))
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(render_vector_checkbox(
            is_on,
            false,
            14.0,
            box_bg,
            box_border,
            check_glyph,
            theme.radius.sm,
        ))
        .child(
            div()
                .w(px(22.0))
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if is_on { colors.cyan } else { colors.text_muted })
                .child(SharedString::from(order_badge.to_owned())),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(if is_on { colors.text_primary } else { colors.text_secondary })
                .child(SharedString::from(block.title.clone())),
        )
        .child(
            Badge::new(cat_label, if is_on { BadgeVariant::Accent } else { BadgeVariant::Neutral })
                .with_theme(theme),
        );

    if let Some(ref cb) = callbacks.toggle_block {
        let cb_clone = Arc::clone(cb);
        left_click_area = left_click_area.on_click(move |_ev, w, c| cb_clone(idx, w, c));
    }

    let is_custom_item = block.id.starts_with("custom_");
    let mut actions = div().flex_shrink_0().flex().flex_row().items_center().gap(px(2.0));

    if is_custom_item {
        let mut del_btn = UiButton::icon_only(("prompt-block-del", idx), UiIcon::X)
            .variant(ButtonVariant::Ghost)
            .with_theme(theme);
        if let Some(ref cb) = callbacks.delete_block {
            let cb_clone = Arc::clone(cb);
            del_btn = del_btn.on_click(move |_ev, w, c| {
                c.stop_propagation();
                cb_clone(idx, w, c);
            });
        }
        actions = actions.child(del_btn);
    }

    let mut up_btn = UiButton::icon_only(("prompt-block-up", idx), UiIcon::ChevronUp)
        .variant(ButtonVariant::Ghost)
        .disabled(idx == 0)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.move_up {
        let cb_clone = Arc::clone(cb);
        up_btn = up_btn.on_click(move |_ev, w, c| {
            c.stop_propagation();
            cb_clone(idx, w, c);
        });
    }

    let mut down_btn = UiButton::icon_only(("prompt-block-down", idx), UiIcon::ChevronDown)
        .variant(ButtonVariant::Ghost)
        .disabled(idx + 1 >= total_len)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.move_down {
        let cb_clone = Arc::clone(cb);
        down_btn = down_btn.on_click(move |_ev, w, c| {
            c.stop_propagation();
            cb_clone(idx, w, c);
        });
    }

    div()
        .w_full()
        .h(px(30.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.sm))
        .bg(colors.bg_panel)
        .border_1()
        .border_color(if is_on { colors.border_active } else { colors.border })
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.sm))
        .child(left_click_area)
        .child(actions.child(up_btn).child(down_btn))
}

fn render_prompt_modal_footer(
    has_custom_override: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_copy_prompt: Option<&FilterModalActionCallback>,
    on_reset_builtin: Option<&FilterModalActionCallback>,
    on_close: Option<&FilterModalActionCallback>,
) -> impl IntoElement + use<> {
    let mut left_actions = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));

    let mut copy_btn = UiButton::new("prompt-modal-copy-btn", labels.prompt_copy_template_btn)
        .icon(UiIcon::Copy)
        .variant(ButtonVariant::Ghost)
        .with_theme(theme);
    if let Some(cb) = on_copy_prompt {
        let cb_clone = Arc::clone(cb);
        copy_btn = copy_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }
    left_actions = left_actions.child(copy_btn);

    if has_custom_override {
        let mut reset_btn =
            UiButton::new("prompt-modal-reset-btn", labels.prompt_reset_default_btn)
                .icon(UiIcon::RotateCcw)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme);
        if let Some(cb) = on_reset_builtin {
            let cb_clone = Arc::clone(cb);
            reset_btn = reset_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
        }
        left_actions = left_actions.child(reset_btn);
    }

    let mut done_btn = UiButton::new("prompt-modal-done-btn", labels.status_ready)
        .icon(UiIcon::Check)
        .variant(ButtonVariant::Primary)
        .with_theme(theme);
    if let Some(cb) = on_close {
        let cb_clone = Arc::clone(cb);
        done_btn = done_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.sm))
        .child(left_actions)
        .child(done_btn)
}
