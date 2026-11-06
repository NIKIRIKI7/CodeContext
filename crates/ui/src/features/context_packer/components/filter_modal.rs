use std::fmt;
use std::sync::Arc;

pub use core::preset::models::preset::{RegexTargetScope, RegexTemplateItem};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

pub use super::prompt_modal::{
    NewPromptCategoryCallback, PromptCategoryCallback, PromptIndexCallback,
    SystemPromptModalComponent, SystemPromptModalProps,
};
use crate::locale::UiLabels;
use crate::shared::containers::form_controls::{TextInputField, render_segmented_tab};
use crate::shared::containers::modal_dialog::ModalDialogShell;
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
        let labels = self.props.labels;
        let badge_variant = if self.props.active_rules_count > 0 {
            BadgeVariant::Accent
        } else {
            BadgeVariant::Neutral
        };
        let active_badge = Badge::new(
            format!("{} {}", self.props.active_rules_count, labels.rules_active_suffix),
            badge_variant,
        )
        .with_theme(theme);

        let ext_section = render_extensions_section(&self.props, &self.callbacks);
        let folders_section = render_folders_section(&self.props, &self.callbacks);
        let regex_section = render_regex_section(&self.props, &self.callbacks);
        let footer = render_modal_footer(&self.props, labels, &self.callbacks);

        ModalDialogShell::new(
            "filter-modal-backdrop",
            "filter-modal-dialog",
            "filter-modal-close-btn",
            labels.filter_modal_title,
        )
        .width(620.0)
        .leading_icon(UiIcon::SlidersHorizontal)
        .header_badge(active_badge)
        .with_theme(theme)
        .on_close_arc(self.callbacks.close)
        .child(ext_section)
        .child(folders_section)
        .child(regex_section)
        .child(footer)
    }
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
        let cb_opt = callbacks.select_ext_mode.clone();
        mode_bar = mode_bar.child(render_segmented_tab(
            id,
            text,
            is_active,
            false,
            false,
            theme,
            move |_ev, w, c| {
                if let Some(ref cb) = cb_opt {
                    cb(mode, w, c);
                }
            },
        ));
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
    let labels = props.labels;
    let has_draft = !props.extension_input_draft.is_empty();

    let input_box = TextInputField::new(
        "filter-modal-extension-input",
        props.extension_input_draft.clone(),
        labels.filter_ext_input_placeholder,
        props.extension_input_focused,
    )
    .leading_icon(UiIcon::FileCode)
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_extension_input.clone());

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
    let labels = props.labels;
    let has_draft = !props.folder_input_draft.is_empty();

    let input_box = TextInputField::new(
        "filter-modal-folder-input",
        props.folder_input_draft.clone(),
        labels.filter_folder_input_placeholder,
        props.folder_input_focused,
    )
    .leading_icon(UiIcon::Folder)
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_folder_input.clone());

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
        let cb_opt = callbacks.select_regex_scope.clone();
        bar = bar.child(render_segmented_tab(
            id,
            text,
            is_active,
            false,
            false,
            theme,
            move |_ev, w, c| {
                if let Some(ref cb) = cb_opt {
                    cb(scope, w, c);
                }
            },
        ));
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

    let mut trailing_row = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));
    if has_pattern {
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
        trailing_row = trailing_row.child(btn);
    }

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
    trailing_row = trailing_row.child(case_btn);

    TextInputField::new(
        "filter-modal-regex-input",
        props.regex_pattern.clone(),
        labels.tree_regex_placeholder,
        props.regex_focused,
    )
    .invalid(!props.regex_is_valid)
    .leading_prefix("/")
    .trailing(trailing_row)
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_regex_input.clone())
}

fn render_regex_template_name_box(
    props: &FilterModalProps,
    callbacks: &FilterModalCallbacks,
) -> impl IntoElement + use<> {
    TextInputField::new(
        "filter-modal-regex-name-input",
        props.regex_name_draft.clone(),
        props.labels.filter_regex_name_placeholder,
        props.regex_name_focused,
    )
    .monospace(false)
    .fixed_width(148.0)
    .with_theme(props.theme)
    .on_focus_arc(callbacks.focus_regex_name_input.clone())
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
