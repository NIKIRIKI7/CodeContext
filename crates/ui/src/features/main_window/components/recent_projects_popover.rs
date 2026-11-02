use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use core::preset::models::resolution::{normalize_recent_project_path, paths_are_equivalent};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::locale::UiLabels;
use crate::shared::primitives::icon::{UiIcon, render_icon};
use crate::theme::{
    FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, format_project_path_smart, platform_shortcuts,
};

pub type SelectRecentProjectCallback = Arc<dyn Fn(PathBuf, &mut Window, &mut App) + 'static>;
pub type RecentPopoverActionCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SubmenuHoverCallback = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct RecentProjectsPopover {
    recent_paths: Vec<String>,
    active_project_path: Option<String>,
    submenu_open: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_select_project: Option<SelectRecentProjectCallback>,
    on_browse_folder: Option<RecentPopoverActionCallback>,
    on_create_project: Option<RecentPopoverActionCallback>,
    on_set_submenu_open: Option<SubmenuHoverCallback>,
    on_close: Option<RecentPopoverActionCallback>,
}

impl fmt::Debug for RecentProjectsPopover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecentProjectsPopover")
            .field("recent_count", &self.recent_paths.len())
            .finish_non_exhaustive()
    }
}

impl RecentProjectsPopover {
    #[inline]
    pub const fn new(
        recent_paths: Vec<String>,
        active_project_path: Option<String>,
        labels: &'static UiLabels,
        theme: &'static ThemeTokens,
    ) -> Self {
        Self {
            recent_paths,
            active_project_path,
            submenu_open: false,
            labels,
            theme,
            on_select_project: None,
            on_browse_folder: None,
            on_create_project: None,
            on_set_submenu_open: None,
            on_close: None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_submenu_open(mut self, submenu_open: bool) -> Self {
        self.submenu_open = submenu_open;
        self
    }

    #[must_use]
    #[inline]
    pub fn on_set_submenu_open(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_set_submenu_open = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn with_actions(
        mut self,
        on_select_project: impl Fn(PathBuf, &mut Window, &mut App) + 'static,
        on_browse_folder: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_create_project: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_project = Some(Arc::new(on_select_project));
        self.on_browse_folder = Some(Arc::new(on_browse_folder));
        self.on_create_project = Some(Arc::new(on_create_project));
        self.on_close = Some(Arc::new(on_close));
        self
    }
}

impl RenderOnce for RecentProjectsPopover {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let colors = &theme.colors;
        let labels = self.labels;

        let actions_section = render_actions_section(
            labels,
            theme,
            self.on_browse_folder.as_ref(),
            self.on_create_project.as_ref(),
            self.on_set_submenu_open.as_ref(),
        );

        let recent_trigger_row = render_recent_submenu_trigger(
            labels,
            theme,
            self.submenu_open,
            self.on_set_submenu_open.as_ref(),
        );

        let main_menu_card = div()
            .id("recent-projects-popover-card")
            .on_click(|_ev, _win, cx| {
                cx.stop_propagation();
            })
            .w(px(236.0))
            .rounded(px(theme.radius.lg))
            .bg(colors.bg_panel)
            .border_1()
            .border_color(colors.border_active)
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(actions_section)
            .child(div().w_full().h(px(1.0)).bg(colors.border))
            .child(div().w_full().p(px(theme.spacing.xs)).child(recent_trigger_row));

        let mut menu_row_container = div()
            .absolute()
            .top(px(theme.layout.top_bar_height + 4.0))
            .left(px(56.0))
            .flex()
            .flex_row()
            .items_start()
            .gap(px(theme.spacing.xs))
            .child(main_menu_card);

        if self.submenu_open {
            menu_row_container = menu_row_container.child(render_recent_flyout_card(
                &self.recent_paths,
                self.active_project_path.as_deref(),
                labels,
                theme,
                self.on_select_project.as_ref(),
            ));
        }

        let mut backdrop =
            div().id("recent-projects-backdrop").absolute().top_0().left_0().size_full();
        if let Some(close_fn) = self.on_close {
            backdrop = backdrop.on_click(move |ev, win, cx| close_fn(ev, win, cx));
        }

        backdrop.child(menu_row_container)
    }
}

fn render_actions_section(
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_browse: Option<&RecentPopoverActionCallback>,
    on_create: Option<&RecentPopoverActionCallback>,
    on_set_submenu_open: Option<&SubmenuHoverCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;

    let mut open_row = div()
        .id("recent-projects-browse-btn")
        .w_full()
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.sm))
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.md))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.sm))
                .child(render_icon(UiIcon::FolderOpen, 13.0, colors.text_secondary))
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_primary)
                        .child(labels.menu_open_project),
                ),
        )
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_muted)
                .child(platform_shortcuts().open_project),
        );

    if let Some(hover_fn) = on_set_submenu_open {
        let cb_hover = Arc::clone(hover_fn);
        open_row = open_row.on_hover(move |&entered, win, cx| {
            if entered {
                cb_hover(false, win, cx);
            }
        });
    }
    if let Some(browse_fn) = on_browse {
        let cb = Arc::clone(browse_fn);
        open_row = open_row.on_click(move |ev, win, cx| {
            cx.stop_propagation();
            cb(ev, win, cx);
        });
    }

    let mut create_row = div()
        .id("recent-projects-create-btn")
        .w_full()
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.sm))
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(render_icon(UiIcon::Plus, 13.0, colors.text_secondary))
        .child(
            div()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_primary)
                .child(labels.menu_create_new_project),
        );

    if let Some(hover_fn) = on_set_submenu_open {
        let cb_hover = Arc::clone(hover_fn);
        create_row = create_row.on_hover(move |&entered, win, cx| {
            if entered {
                cb_hover(false, win, cx);
            }
        });
    }
    if let Some(create_fn) = on_create {
        let cb = Arc::clone(create_fn);
        create_row = create_row.on_click(move |ev, win, cx| {
            cx.stop_propagation();
            cb(ev, win, cx);
        });
    }

    div().w_full().flex().flex_col().p(px(theme.spacing.xs)).child(open_row).child(create_row)
}

fn render_recent_submenu_trigger(
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    submenu_open: bool,
    on_set_submenu_open: Option<&SubmenuHoverCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;

    let mut trigger = div()
        .id("recent-projects-submenu-trigger")
        .w_full()
        .h(px(28.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.sm))
        .bg(if submenu_open { colors.bg_hover } else { colors.bg_panel })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.md))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.sm))
                .child(render_icon(UiIcon::Folder, 13.0, colors.text_secondary))
                .child(
                    div()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(colors.text_primary)
                        .child(labels.menu_section_recent),
                ),
        )
        .child(render_icon(UiIcon::ChevronRight, 12.0, colors.text_muted));

    if let Some(hover_fn) = on_set_submenu_open {
        let cb_hover = Arc::clone(hover_fn);
        let cb_click = Arc::clone(hover_fn);
        trigger = trigger
            .on_hover(move |&entered, win, cx| {
                if entered {
                    cb_hover(true, win, cx);
                }
            })
            .on_click(move |_ev, win, cx| {
                cx.stop_propagation();
                cb_click(!submenu_open, win, cx);
            });
    }

    trigger
}

fn render_recent_flyout_card(
    recent_paths: &[String],
    active_project_path: Option<&str>,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_select_project: Option<&SelectRecentProjectCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let mut deduplicated: Vec<&str> = Vec::with_capacity(recent_paths.len());
    for path in recent_paths {
        let norm = normalize_recent_project_path(path);
        if (norm.contains('/') || norm.contains('\\'))
            && !deduplicated.iter().any(|existing| paths_are_equivalent(existing, norm))
        {
            deduplicated.push(norm);
        }
    }

    let mut list_col = div().w_full().flex().flex_col().p(px(theme.spacing.xs));
    if deduplicated.is_empty() {
        list_col = list_col.child(
            div()
                .w_full()
                .h(px(28.0))
                .px(px(theme.spacing.md))
                .flex()
                .items_center()
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(colors.text_muted)
                .child(labels.menu_recent_empty),
        );
    } else {
        for (idx, norm_path) in deduplicated.into_iter().enumerate() {
            let is_current =
                active_project_path.is_some_and(|act| paths_are_equivalent(act, norm_path));
            list_col = list_col.child(render_recent_project_row(
                idx,
                norm_path,
                is_current,
                theme,
                on_select_project,
            ));
        }
    }

    div()
        .id("recent-projects-flyout-card")
        .on_click(|_ev, _win, cx| {
            cx.stop_propagation();
        })
        .w(px(330.0))
        .rounded(px(theme.radius.lg))
        .bg(colors.bg_panel)
        .border_1()
        .border_color(colors.border_active)
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(list_col)
}

fn render_recent_project_row(
    idx: usize,
    raw_path: &str,
    is_current: bool,
    theme: &'static ThemeTokens,
    on_select_project: Option<&SelectRecentProjectCallback>,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let (folder_name, parent_hint) = split_project_name_and_parent(raw_path);
    let icon = if is_current { UiIcon::FolderOpen } else { UiIcon::Folder };
    let icon_color = if is_current { colors.cyan } else { colors.text_secondary };

    let mut row = div()
        .id(("recent-project-item", idx))
        .w_full()
        .h(px(30.0))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.sm))
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.md))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.sm))
                .overflow_hidden()
                .child(render_icon(icon, 14.0, icon_color))
                .child(
                    div()
                        .flex_shrink_0()
                        .font_family(FONT_FAMILY_UI)
                        .text_xs()
                        .text_color(if is_current {
                            colors.text_primary
                        } else {
                            colors.text_secondary
                        })
                        .child(SharedString::from(folder_name.to_owned())),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .font_family(FONT_FAMILY_MONO)
                        .text_xs()
                        .text_color(colors.text_muted)
                        .child(SharedString::from(parent_hint)),
                ),
        );

    if is_current {
        row = row.child(render_icon(UiIcon::Check, 13.0, colors.cyan));
    }

    if let Some(select_fn) = on_select_project {
        let cb = Arc::clone(select_fn);
        let target = PathBuf::from(raw_path);
        row = row.on_click(move |_ev, win, cx| {
            cx.stop_propagation();
            cb(target.clone(), win, cx);
        });
    }

    row
}

fn split_project_name_and_parent(raw_path: &str) -> (&str, String) {
    let trimmed = normalize_recent_project_path(raw_path);
    if let Some(pos) = trimmed.rfind(['/', '\\']) {
        let name = trimmed.get(pos + 1..).unwrap_or(trimmed);
        let parent = trimmed.get(..pos).unwrap_or("");
        return (name, format_project_path_smart(parent, 28));
    }
    (trimmed, String::new())
}
