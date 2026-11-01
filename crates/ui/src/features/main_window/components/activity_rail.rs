use std::fmt;
use std::sync::Arc;

use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::features::main_window::state::WorkspaceTab;
use crate::locale::UiLabels;
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::ThemeTokens;

pub type SelectWorkspaceTabCallback = Arc<dyn Fn(WorkspaceTab, &mut Window, &mut App) + 'static>;
pub type RailActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct ActivityRailComponent {
    active_tab: WorkspaceTab,
    settings_open: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_select_tab: Option<SelectWorkspaceTabCallback>,
    on_toggle_settings: Option<RailActionCallback>,
}

impl fmt::Debug for ActivityRailComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActivityRailComponent")
            .field("active_tab", &self.active_tab)
            .field("settings_open", &self.settings_open)
            .finish_non_exhaustive()
    }
}

impl ActivityRailComponent {
    #[inline]
    pub const fn new(
        active_tab: WorkspaceTab,
        settings_open: bool,
        labels: &'static UiLabels,
        theme: &'static ThemeTokens,
    ) -> Self {
        Self {
            active_tab,
            settings_open,
            labels,
            theme,
            on_select_tab: None,
            on_toggle_settings: None,
        }
    }

    #[must_use]
    pub fn with_actions(
        mut self,
        on_select_tab: impl Fn(WorkspaceTab, &mut Window, &mut App) + 'static,
        on_toggle_settings: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_tab = Some(Arc::new(on_select_tab));
        self.on_toggle_settings = Some(Arc::new(on_toggle_settings));
        self
    }
}

impl RenderOnce for ActivityRailComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let colors = &theme.colors;
        let labels = self.labels;

        let nav_tabs: &[(&str, UiIcon, &str, WorkspaceTab)] = &[
            (
                "rail-tab-packer",
                UiIcon::Package,
                labels.nav_pack_short,
                WorkspaceTab::ContextPacker,
            ),
            (
                "rail-tab-applier",
                UiIcon::FileDiff,
                labels.nav_apply_short,
                WorkspaceTab::ChangeApplier,
            ),
        ];

        let mut top_col = div().w_full().flex().flex_col().items_center().gap(px(theme.spacing.xs));

        for &(id, icon, caption, tab) in nav_tabs {
            let is_active = self.active_tab == tab && !self.settings_open;
            let mut item = render_rail_button(id, icon, caption, is_active, theme);
            if let Some(ref cb) = self.on_select_tab {
                let cb_clone = Arc::clone(cb);
                item = item.on_click(move |_ev, win, cx| cb_clone(tab, win, cx));
            }
            top_col = top_col.child(item);
        }

        let mut settings_btn = render_rail_button(
            "rail-btn-settings",
            UiIcon::Settings,
            labels.btn_settings,
            self.settings_open,
            theme,
        );
        if let Some(cb) = self.on_toggle_settings {
            settings_btn = settings_btn.on_click(move |ev, w, c| cb(ev, w, c));
        }

        let bottom_col = div().w_full().flex().flex_col().items_center().child(settings_btn);

        div()
            .w(px(48.0))
            .h_full()
            .py(px(theme.spacing.sm))
            .bg(colors.bg_panel)
            .border_r_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .items_center()
            .justify_between()
            .flex_shrink_0()
            .child(top_col)
            .child(bottom_col)
    }
}

fn render_rail_button(
    id: &'static str,
    icon: UiIcon,
    _caption: &'static str,
    active: bool,
    theme: &'static ThemeTokens,
) -> gpui::Stateful<gpui::Div> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let icon_color = if active { colors.cyan } else { colors.text_secondary };
    div()
        .id(id)
        .w(px(36.0))
        .h(px(36.0))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if active { colors.border_active } else { colors.bg_panel })
        .bg(if active { colors.bg_elevated } else { colors.bg_panel })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .items_center()
        .justify_center()
        .child(render_icon(icon, 17.0, icon_color))
}
