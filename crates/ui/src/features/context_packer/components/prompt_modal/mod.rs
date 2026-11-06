use std::fmt;
use std::sync::Arc;

use core::preset::models::template::{PromptBlockItem, PromptCategory};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::features::context_packer::components::filter_modal::FilterModalActionCallback;
use crate::locale::UiLabels;
use crate::shared::containers::form_controls::{TextInputField, render_segmented_tab};
use crate::shared::containers::modal_dialog::ModalDialogShell;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_vector_checkbox};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens};

pub type PromptIndexCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type PromptCategoryCallback =
    Arc<dyn Fn(Option<PromptCategory>, &mut Window, &mut App) + 'static>;
pub type NewPromptCategoryCallback = Arc<dyn Fn(PromptCategory, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct SystemPromptModalProps {
    pub active_category: Option<PromptCategory>,
    pub prompt_category_draft: String,
    pub prompt_category_focused: bool,
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
    toggle_block: Option<PromptIndexCallback>,
    move_up: Option<PromptIndexCallback>,
    move_down: Option<PromptIndexCallback>,
    delete_block: Option<PromptIndexCallback>,
    focus_prompt_category: Option<FilterModalActionCallback>,
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
    pub fn on_focus_prompt_category(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_prompt_category = Some(Arc::new(handler));
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
        let labels = self.props.labels;

        let composer_card = render_prompt_composer_card(&self.props, &self.callbacks);
        let cat_bar = render_prompt_category_bar(&self.props, &self.callbacks);
        let blocks_list = render_prompt_blocks_list(&self.props, &self.callbacks);

        let any_enabled = self.props.blocks.iter().any(|b| b.enabled);
        let footer_row = render_prompt_modal_footer(
            any_enabled,
            labels,
            theme,
            self.callbacks.copy_prompt.as_ref(),
            self.callbacks.reset_builtin.as_ref(),
            self.callbacks.close.as_ref(),
        );

        ModalDialogShell::new(
            "system-prompt-modal-backdrop",
            "system-prompt-modal-dialog",
            "prompt-modal-close-btn",
            labels.prompt_modal_title,
        )
        .width(680.0)
        .leading_icon(UiIcon::FileText)
        .with_theme(theme)
        .on_close_arc(self.callbacks.close)
        .child(composer_card)
        .child(cat_bar)
        .child(blocks_list)
        .child(footer_row)
    }
}

fn render_prompt_composer_card(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let cat_input = TextInputField::new(
        "system-prompt-category-input",
        props.prompt_category_draft.clone(),
        labels.prompt_cat_custom,
        props.prompt_category_focused,
    )
    .monospace(false)
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_prompt_category.clone());

    let title_input = TextInputField::new(
        "system-prompt-title-input",
        props.prompt_title_draft.clone(),
        labels.prompt_field_title_placeholder,
        props.prompt_title_focused,
    )
    .monospace(false)
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_prompt_title.clone());

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
        .child(render_composer_labeled_row(labels.prompt_field_category_label, cat_input, theme))
        .child(render_composer_labeled_row(labels.prompt_field_title_label, title_input, theme))
        .child(content_row)
}

fn render_composer_labeled_row(
    label: &'static str,
    field: impl IntoElement,
    theme: &'static ThemeTokens,
) -> gpui::Div {
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
                .text_color(theme.colors.text_secondary)
                .child(label),
        )
        .child(field)
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
        let cb_opt = callbacks.select_category.clone();
        bar = bar.child(render_segmented_tab(
            id,
            text,
            is_active,
            false,
            true,
            theme,
            move |_ev, w, c| {
                if let Some(ref cb) = cb_opt {
                    cb(cat_opt, w, c);
                }
            },
        ));
    }

    bar
}

fn render_manual_prompt_input_row(
    props: &SystemPromptModalProps,
    callbacks: &PromptModalCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let labels = props.labels;
    let has_draft = !props.prompt_input_draft.is_empty();
    let can_submit = has_draft || !props.prompt_title_draft.is_empty();

    let single_line_preview = props.prompt_input_draft.replace(['\r', '\n'], " ↵ ");
    let input_box = TextInputField::new(
        "system-prompt-modal-input",
        single_line_preview,
        labels.prompt_add_input_placeholder,
        props.prompt_input_focused,
    )
    .leading_icon(UiIcon::FileText)
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_prompt_input.clone());

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

    let controls = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(input_box)
        .child(paste_btn)
        .child(add_btn);

    render_composer_labeled_row(labels.prompt_field_content_label, controls, theme)
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
        .max_h(px(220.0))
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
        let default_cat_label = match block.category {
            PromptCategory::Patcher => labels.prompt_cat_patcher,
            PromptCategory::Architecture => labels.prompt_cat_arch,
            PromptCategory::Behavior => labels.prompt_cat_behavior,
            PromptCategory::Custom => labels.prompt_cat_custom,
        };
        let cat_label = block.custom_category.as_deref().unwrap_or(default_cat_label);
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
    cat_label: &str,
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
            Badge::new(
                SharedString::from(cat_label.to_owned()),
                if is_on { BadgeVariant::Accent } else { BadgeVariant::Neutral },
            )
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
