use std::fmt;
use std::sync::Arc;

pub use core::preset::models::resolution::{TerminalCommandPreset, TerminalTaskCategory};
use core::shared_kernel::execution::telemetry::{
    CRITICAL_LAG_THRESHOLD_US, LogEntry, LogFilterMode, LogLevel, SLOW_OPERATION_THRESHOLD_US,
    TelemetrySummary,
};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, MouseButton, MouseDownEvent,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _,
    Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::containers::form_controls::{TextInputField, render_segmented_tab};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens};

pub const DEFAULT_DOCK_HEIGHT_PX: f32 = 300.0;
pub const MIN_DOCK_HEIGHT_PX: f32 = 140.0;
pub const MAX_DOCK_HEIGHT_PX: f32 = 620.0;
pub const TALL_DOCK_HEIGHT_PX: f32 = 460.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DockToolTab {
    #[default]
    Logs,
    Terminal,
}

impl DockToolTab {
    pub const ALL: &'static [Self] = &[Self::Logs, Self::Terminal];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCommandEntry {
    pub command: String,
    pub output: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

pub type SelectDockTabCallback = Arc<dyn Fn(DockToolTab, &mut Window, &mut App) + 'static>;
pub type SelectLogFilterCallback = Arc<dyn Fn(LogFilterMode, &mut Window, &mut App) + 'static>;
pub type SelectTaskCategoryCallback =
    Arc<dyn Fn(Option<TerminalTaskCategory>, &mut Window, &mut App) + 'static>;
pub type DockActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type RunPresetCommandCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type CopyErrorForAiCallback = Arc<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>;
pub type DeletePresetCallback = Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type StartDockResizeCallback = Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone)]
pub struct DiagnosticsDockProps {
    pub active_tab: DockToolTab,
    pub entries: Vec<LogEntry>,
    pub summary: TelemetrySummary,
    pub filter_mode: LogFilterMode,
    pub task_category: Option<TerminalTaskCategory>,
    pub terminal_presets: Vec<TerminalCommandPreset>,
    pub terminal_input: String,
    pub terminal_focused: bool,
    pub terminal_running: bool,
    pub dock_height_px: f32,
    pub is_resizing_dock: bool,
    pub show_terminal_presets: bool,
    pub show_inline_categories: bool,
    pub terminal_history: Vec<TerminalCommandEntry>,
    pub labels: &'static UiLabels,
    pub theme: &'static ThemeTokens,
}

#[derive(Clone, Default)]
struct DockCallbacks {
    select_tab: Option<SelectDockTabCallback>,
    select_filter: Option<SelectLogFilterCallback>,
    select_task_category: Option<SelectTaskCategoryCallback>,
    focus_terminal_input: Option<DockActionCallback>,
    submit_terminal_cmd: Option<DockActionCallback>,
    save_terminal_preset: Option<DockActionCallback>,
    run_preset_cmd: Option<RunPresetCommandCallback>,
    copy_error_for_ai: Option<CopyErrorForAiCallback>,
    delete_preset_idx: Option<DeletePresetCallback>,
    start_resize: Option<StartDockResizeCallback>,
    copy_active: Option<DockActionCallback>,
    clear_active: Option<DockActionCallback>,
    close: Option<DockActionCallback>,
}

#[derive(IntoElement)]
pub struct DiagnosticsDockComponent {
    props: DiagnosticsDockProps,
    callbacks: DockCallbacks,
}

impl fmt::Debug for DiagnosticsDockComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiagnosticsDockComponent")
            .field("active_tab", &self.props.active_tab)
            .field("entries_count", &self.props.entries.len())
            .finish_non_exhaustive()
    }
}

impl DiagnosticsDockComponent {
    #[inline]
    pub fn new(props: DiagnosticsDockProps) -> Self {
        Self { props, callbacks: DockCallbacks::default() }
    }

    #[must_use]
    #[inline]
    pub fn on_select_tab(
        mut self,
        handler: impl Fn(DockToolTab, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_tab = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_select_task_category(
        mut self,
        handler: impl Fn(Option<TerminalTaskCategory>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_task_category = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_preset_management(
        mut self,
        on_save_preset: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_delete_preset: impl Fn(usize, &mut Window, &mut App) + 'static,
        on_start_resize: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.save_terminal_preset = Some(Arc::new(on_save_preset));
        self.callbacks.delete_preset_idx = Some(Arc::new(on_delete_preset));
        self.callbacks.start_resize = Some(Arc::new(on_start_resize));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_terminal_actions(
        mut self,
        on_focus_input: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_submit_cmd: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_run_preset: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.focus_terminal_input = Some(Arc::new(on_focus_input));
        self.callbacks.submit_terminal_cmd = Some(Arc::new(on_submit_cmd));
        self.callbacks.run_preset_cmd = Some(Arc::new(on_run_preset));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_copy_error_for_ai(
        mut self,
        handler: impl Fn(&str, &str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.copy_error_for_ai = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn with_actions(
        mut self,
        on_select_filter: impl Fn(LogFilterMode, &mut Window, &mut App) + 'static,
        on_copy_active: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_clear_active: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.select_filter = Some(Arc::new(on_select_filter));
        self.callbacks.copy_active = Some(Arc::new(on_copy_active));
        self.callbacks.clear_active = Some(Arc::new(on_clear_active));
        self.callbacks.close = Some(Arc::new(on_close));
        self
    }
}

impl RenderOnce for DiagnosticsDockComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.props.theme;
        let colors = &theme.colors;
        let dock_h = self.props.dock_height_px.clamp(MIN_DOCK_HEIGHT_PX, MAX_DOCK_HEIGHT_PX);
        let hover_col = colors.cyan;
        let top_border_col =
            if self.props.is_resizing_dock { colors.cyan } else { colors.border_active };

        let mut resize_handle = div()
            .id("diagnostics-dock-resize-handle")
            .w_full()
            .h(px(5.0))
            .flex_shrink_0()
            .cursor_ns_resize()
            .bg(if self.props.is_resizing_dock { colors.cyan } else { colors.bg_panel })
            .border_t_1()
            .border_color(top_border_col)
            .hover(move |s| s.bg(hover_col).border_color(hover_col));

        if let Some(ref cb) = self.callbacks.start_resize {
            let cb_clone = Arc::clone(cb);
            resize_handle = resize_handle.on_mouse_down(MouseButton::Left, move |ev, w, c| {
                c.stop_propagation();
                cb_clone(ev, w, c);
            });
        }

        let header = render_dock_header(&self.props, &self.callbacks);
        let body = match self.props.active_tab {
            DockToolTab::Logs => render_logs_body(&self.props).into_any_element(),
            DockToolTab::Terminal => {
                render_terminal_body(&self.props, &self.callbacks).into_any_element()
            }
        };

        div()
            .id("diagnostics-dock-panel")
            .on_mouse_down(MouseButton::Left, |_ev, _win, cx| {
                cx.stop_propagation();
            })
            .on_click(|_ev, _win, cx| {
                cx.stop_propagation();
            })
            .w_full()
            .h(px(dock_h))
            .min_h(px(dock_h))
            .bg(colors.bg_app)
            .flex_shrink_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(resize_handle)
            .child(header)
            .child(div().flex_1().min_h(px(0.0)).w_full().overflow_hidden().child(body))
    }
}

fn render_dock_header(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;

    let left_area = if props.show_inline_categories {
        render_inline_category_tabs_bar(props, callbacks).into_any_element()
    } else {
        match props.active_tab {
            DockToolTab::Logs => render_logs_title_and_stats(props).into_any_element(),
            DockToolTab::Terminal => {
                render_terminal_task_category_tabs(props, callbacks).into_any_element()
            }
        }
    };

    let mut right_actions = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));
    if props.active_tab == DockToolTab::Logs {
        right_actions = right_actions.child(render_logs_header_controls(props, callbacks));
    }

    if let Some(ref cb) = callbacks.copy_active {
        let cb_clone = Arc::clone(cb);
        right_actions = right_actions.child(
            UiButton::icon_only("logs-dock-copy-btn", UiIcon::Copy)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme)
                .on_click(move |ev, w, c| cb_clone(ev, w, c)),
        );
    }

    if let Some(ref cb) = callbacks.clear_active {
        let cb_clone = Arc::clone(cb);
        right_actions = right_actions.child(
            UiButton::icon_only("logs-dock-clear-btn", UiIcon::RotateCcw)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme)
                .on_click(move |ev, w, c| cb_clone(ev, w, c)),
        );
    }

    if let Some(ref cb) = callbacks.close {
        let cb_clone = Arc::clone(cb);
        right_actions = right_actions.child(
            UiButton::icon_only("logs-dock-close-btn", UiIcon::X)
                .variant(ButtonVariant::Ghost)
                .with_theme(theme)
                .on_click(move |ev, w, c| cb_clone(ev, w, c)),
        );
    }

    div()
        .w_full()
        .h(px(32.0))
        .min_h(px(32.0))
        .flex_shrink_0()
        .px(px(theme.spacing.lg))
        .bg(colors.bg_panel)
        .border_b_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.md))
        .child(left_area)
        .child(right_actions)
}

fn render_logs_title_and_stats(props: &DiagnosticsDockProps) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let peak_ms = props.summary.max_duration_us / 1_000;
    let peak_frac = (props.summary.max_duration_us % 1_000) / 10;
    let stats_str = SharedString::from(format!(
        "{} · {}: {peak_ms}.{peak_frac:02} ms",
        props.summary.total_count, labels.logs_peak_prefix
    ));

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_primary)
                .child(labels.logs_dock_title),
        )
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_muted)
                .child(stats_str),
        )
}

fn render_terminal_task_category_tabs(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let categories: &[(&str, &str, Option<TerminalTaskCategory>)] = &[
        ("t-cat-all", labels.prompt_cat_all, None),
        ("t-cat-git", "Git", Some(TerminalTaskCategory::Git)),
        ("t-cat-build", labels.terminal_cat_build, Some(TerminalTaskCategory::Build)),
        ("t-cat-test", labels.terminal_cat_test, Some(TerminalTaskCategory::Test)),
        ("t-cat-custom", labels.prompt_cat_custom, Some(TerminalTaskCategory::Custom)),
    ];

    let mut bar = div()
        .h(px(24.0))
        .p(px(1.0))
        .rounded(px(theme.radius.sm))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for &(id, title, cat_opt) in categories {
        let is_active = props.task_category == cat_opt;
        let cb_opt = callbacks.select_task_category.clone();
        bar = bar.child(render_segmented_tab(
            id,
            title,
            is_active,
            false,
            false,
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

fn render_inline_category_tabs_bar(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let mut row = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));

    for &tab in DockToolTab::ALL {
        let is_active = props.active_tab == tab;
        let hover_bg = colors.bg_hover;
        let (tab_id, icon, title) = match tab {
            DockToolTab::Logs => ("dock-cat-logs", UiIcon::SlidersHorizontal, labels.dock_tab_logs),
            DockToolTab::Terminal => {
                ("dock-cat-terminal", UiIcon::Terminal, labels.dock_tab_terminal)
            }
        };

        let mut btn = div()
            .id(tab_id)
            .h(px(22.0))
            .px(px(theme.spacing.sm))
            .rounded(px(theme.radius.sm))
            .border_1()
            .border_color(if is_active { colors.border_active } else { colors.bg_panel })
            .bg(if is_active { colors.bg_elevated } else { colors.bg_panel })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xs))
            .child(render_icon(icon, 12.0, if is_active { colors.cyan } else { colors.text_muted }))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(if is_active { colors.text_primary } else { colors.text_secondary })
                    .child(title),
            );

        if let Some(ref cb) = callbacks.select_tab {
            let cb_clone = Arc::clone(cb);
            btn = btn.on_click(move |_ev, w, c| cb_clone(tab, w, c));
        }
        row = row.child(btn);
    }

    row
}

fn render_logs_header_controls(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let filters: &[(&str, &str, LogFilterMode)] = &[
        ("logs-filter-all", labels.logs_filter_all, LogFilterMode::All),
        ("logs-filter-slow", labels.logs_filter_slow, LogFilterMode::SlowAndWarnings),
        ("logs-filter-err", labels.logs_filter_errors, LogFilterMode::ErrorsOnly),
    ];

    let mut filter_bar = div()
        .h(px(22.0))
        .p(px(1.0))
        .rounded(px(theme.radius.sm))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0));

    for &(id, text, mode) in filters {
        let is_active = props.filter_mode == mode;
        let cb_opt = callbacks.select_filter.clone();
        filter_bar = filter_bar.child(render_segmented_tab(
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

    filter_bar
}

fn render_terminal_preset_chips_bar(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;

    let mut row = div()
        .id("terminal-presets-bar-scroll")
        .w_full()
        .px(px(theme.spacing.lg))
        .py(px(theme.spacing.xs))
        .bg(colors.bg_panel)
        .border_b_1()
        .border_color(colors.border)
        .flex_shrink_0()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(theme.spacing.xs));

    for (idx, preset) in props.terminal_presets.iter().enumerate() {
        if let Some(cat) = props.task_category
            && preset.category != cat
        {
            continue;
        }
        let hover_bg = colors.bg_hover;
        let mut chip = div()
            .id(("term-preset-chip", idx))
            .h(px(22.0))
            .px(px(theme.spacing.sm))
            .rounded(px(theme.radius.sm))
            .border_1()
            .border_color(if preset.is_custom { colors.border_active } else { colors.border })
            .bg(colors.bg_app)
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme.spacing.xxs))
            .font_family(FONT_FAMILY_MONO)
            .text_xs()
            .text_color(colors.text_primary)
            .child(SharedString::from(preset.name.clone()));

        if let Some(ref cb) = callbacks.run_preset_cmd {
            let cb_clone = Arc::clone(cb);
            let cmd_owned = preset.command.clone();
            chip = chip.on_click(move |_ev, w, c| cb_clone(&cmd_owned, w, c));
        }

        if preset.is_custom
            && let Some(ref del_cb) = callbacks.delete_preset_idx
        {
            let cb_del = Arc::clone(del_cb);
            let del_btn = div()
                .id(("term-preset-del", idx))
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
            chip = chip.child(del_btn);
        }

        row = row.child(chip);
    }
    row
}

fn render_logs_body(props: &DiagnosticsDockProps) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    if props.entries.is_empty() {
        return div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(colors.text_muted)
            .child(labels.logs_empty_hint)
            .into_any_element();
    }

    let mut list = div()
        .id("diagnostics-dock-scroll")
        .size_full()
        .overflow_y_scroll()
        .py(px(2.0))
        .flex()
        .flex_col();

    for (idx, entry) in props.entries.iter().enumerate() {
        list = list.child(render_log_row(idx, entry, theme));
    }
    list.into_any_element()
}

fn render_terminal_body(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;

    let presets_bar_opt =
        props.show_terminal_presets.then(|| render_terminal_preset_chips_bar(props, callbacks));
    let input_bar = render_terminal_input_bar(props, callbacks);

    let history_area = if props.terminal_history.is_empty() {
        div()
            .flex_1()
            .min_h(px(0.0))
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_FAMILY_UI)
            .text_xs()
            .text_color(colors.text_muted)
            .child(labels.terminal_empty_hint)
            .into_any_element()
    } else {
        let mut scroll = div()
            .id("terminal-history-scroll")
            .flex_1()
            .min_h(px(0.0))
            .w_full()
            .overflow_y_scroll()
            .px(px(theme.spacing.lg))
            .py(px(theme.spacing.xs))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xs));

        for (idx, item) in props.terminal_history.iter().enumerate().rev() {
            let status_col = if item.exit_code == 0 { colors.success } else { colors.danger };
            let mut right_status = div().flex().flex_row().items_center().gap(px(theme.spacing.xs));
            if item.exit_code != 0
                && let Some(ref copy_err_cb) = callbacks.copy_error_for_ai
            {
                let cb = Arc::clone(copy_err_cb);
                let cmd_owned = item.command.clone();
                let out_owned = item.output.clone();
                let btn_label = labels.terminal_copy_err_ai_btn;
                right_status = right_status.child(
                    UiButton::new(("term-copy-ai-err", idx), btn_label)
                        .icon(UiIcon::Copy)
                        .variant(ButtonVariant::Danger)
                        .with_theme(theme)
                        .on_click(move |_ev, w, c| cb(&cmd_owned, &out_owned, w, c)),
                );
            }
            right_status = right_status.child(div().text_color(status_col).child(
                SharedString::from(format!("exit {} · {} ms", item.exit_code, item.duration_ms)),
            ));

            scroll = scroll.child(
                div()
                    .id(("term-cmd-entry", idx))
                    .w_full()
                    .flex_shrink_0()
                    .p(px(theme.spacing.xs))
                    .rounded(px(theme.radius.sm))
                    .bg(colors.bg_panel)
                    .border_1()
                    .border_color(colors.border)
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(colors.cyan)
                                    .child(SharedString::from(format!("$ {}", item.command))),
                            )
                            .child(right_status),
                    )
                    .child(
                        div()
                            .text_color(colors.text_secondary)
                            .child(SharedString::from(item.output.clone())),
                    ),
            );
        }
        scroll.into_any_element()
    };

    div()
        .size_full()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .overflow_hidden()
        .children(presets_bar_opt)
        .child(input_bar)
        .child(history_area)
}

fn render_terminal_input_bar(
    props: &DiagnosticsDockProps,
    callbacks: &DockCallbacks,
) -> impl IntoElement + use<> {
    let theme = props.theme;
    let colors = &theme.colors;
    let labels = props.labels;
    let has_text = !props.terminal_input.is_empty();

    let input_box = TextInputField::new(
        "terminal-command-input-box",
        props.terminal_input.clone(),
        labels.terminal_input_placeholder,
        props.terminal_focused,
    )
    .leading_prefix(">_")
    .with_theme(theme)
    .on_focus_arc(callbacks.focus_terminal_input.clone());

    let save_preset_label = labels.terminal_save_preset_btn;
    let mut save_preset_btn = UiButton::new("terminal-save-preset-btn", save_preset_label)
        .variant(ButtonVariant::Secondary)
        .disabled(!has_text)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.save_terminal_preset {
        let cb_clone = Arc::clone(cb);
        save_preset_btn = save_preset_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let run_label = if props.terminal_running {
        labels.terminal_running_status
    } else {
        labels.terminal_btn_run
    };
    let mut run_btn = UiButton::new("terminal-run-cmd-btn", run_label)
        .variant(if has_text && !props.terminal_running {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        })
        .disabled(!has_text || props.terminal_running)
        .with_theme(theme);
    if let Some(ref cb) = callbacks.submit_terminal_cmd {
        let cb_clone = Arc::clone(cb);
        run_btn = run_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let mut row = div()
        .w_full()
        .h(px(38.0))
        .min_h(px(38.0))
        .flex_shrink_0()
        .px(px(theme.spacing.lg))
        .py(px(theme.spacing.xs))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.xs))
        .child(input_box)
        .child(save_preset_btn)
        .child(run_btn);

    if !props.terminal_history.is_empty() {
        row = row.border_b_1().border_color(colors.border);
    }

    row
}

fn render_log_row(
    idx: usize,
    entry: &LogEntry,
    theme: &'static ThemeTokens,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;

    let duration_color = match (entry.level, entry.duration_us) {
        (LogLevel::Error, _) => colors.danger,
        (_, Some(us)) if us >= CRITICAL_LAG_THRESHOLD_US => colors.danger,
        (LogLevel::Slow | LogLevel::Warn, _) => colors.warning,
        (_, Some(us)) if us >= SLOW_OPERATION_THRESHOLD_US => colors.warning,
        (_, Some(_)) => colors.success,
        _ => colors.text_muted,
    };

    div()
        .id(("telemetry-log-row", idx))
        .w_full()
        .h(px(22.0))
        .min_h(px(22.0))
        .flex_shrink_0()
        .px(px(theme.spacing.lg))
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.md))
        .font_family(FONT_FAMILY_MONO)
        .text_xs()
        .overflow_hidden()
        .whitespace_nowrap()
        .child(
            div()
                .w(px(68.0))
                .flex_shrink_0()
                .text_color(colors.text_muted)
                .child(SharedString::from(entry.format_timestamp())),
        )
        .child(
            div()
                .w(px(72.0))
                .flex_shrink_0()
                .text_color(colors.cyan)
                .child(SharedString::from(format!("[{}]", entry.subsystem))),
        )
        .child(
            div()
                .w(px(74.0))
                .flex_shrink_0()
                .flex()
                .justify_end()
                .text_color(duration_color)
                .child(SharedString::from(entry.format_duration())),
        )
        .child(
            div()
                .w(px(124.0))
                .flex_shrink_0()
                .text_color(colors.text_primary)
                .child(entry.operation),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .text_color(colors.text_secondary)
                .child(SharedString::from(entry.message.clone())),
        )
}
