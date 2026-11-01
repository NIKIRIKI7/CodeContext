use std::collections::HashSet;
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

use core::packing::aggregates::file_entry::FileEntry;
use core::packing::aggregates::project::Project;
use core::shared_kernel::fs::file_kind::{FileKind, contains_ignore_ascii_case};
use gpui::{
    App, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, UniformListScrollHandle, Window,
    div, px, uniform_list,
};

use crate::locale::UiLabels;
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_icon, render_vector_checkbox};
use crate::theme::{ThemeTokens, active_theme, format_bytes_human};

const TAG_SECRET: &str = "SEC";
const TAG_BINARY: &str = "BIN";
const TAG_SYMLINK: &str = "SYM";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    Checked,
    Unchecked,
    Indeterminate,
}

impl TriState {
    #[inline]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Checked => "[✓]",
            Self::Unchecked => "[ ]",
            Self::Indeterminate => "[-]",
        }
    }

    #[inline]
    pub const fn next_target_selection(self) -> bool {
        !matches!(self, Self::Checked)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeNodeKind {
    Directory { file_count: usize, selected_count: usize, size_bytes: u64 },
    File { kind: FileKind, size_bytes: u64, lock_reason: Option<String>, is_secret_unlocked: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatTreeNode {
    pub path: String,
    pub display_name: String,
    pub depth: u8,
    pub kind: TreeNodeKind,
    pub is_expanded: bool,
    pub check_state: TriState,
}

impl FlatTreeNode {
    #[inline]
    pub const fn is_dir(&self) -> bool {
        matches!(self.kind, TreeNodeKind::Directory { .. })
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlatTreeProjection {
    visible_nodes: Arc<[FlatTreeNode]>,
    total_indexed_files: usize,
    selected_files: usize,
}

impl FlatTreeProjection {
    pub fn build(project: &Project, collapsed_dirs: &HashSet<String>, search_query: &str) -> Self {
        let entries = project.entries();
        let clean_query = search_query.trim();
        let has_query = !clean_query.is_empty();
        let mut nodes = Vec::with_capacity(entries.len() + (entries.len() / 8) + 8);
        let mut active_dir_stack: Vec<String> = Vec::with_capacity(16);
        let mut dir_prefixes_buf: Vec<&str> = Vec::with_capacity(16);

        let mut idx = 0;
        while idx < entries.len() {
            let Some(entry) = entries.get(idx) else {
                break;
            };
            if has_query && !contains_ignore_ascii_case(&entry.rel_path, clean_query) {
                idx += 1;
                continue;
            }

            let skipped_children = emit_parent_directories_for_entry(
                idx,
                project,
                collapsed_dirs,
                has_query,
                &mut active_dir_stack,
                &mut dir_prefixes_buf,
                &mut nodes,
            );

            if !has_query && skipped_children > 0 {
                idx += skipped_children;
                continue;
            }

            if !has_query && is_under_collapsed_dir(&entry.rel_path, collapsed_dirs) {
                idx += 1;
                continue;
            }

            let (file_name, depth) = extract_name_and_depth(&entry.rel_path);
            let is_secret_unlocked = entry.kind == FileKind::Secret
                && project.preset().is_secret_explicitly_allowed(&entry.rel_path);

            nodes.push(FlatTreeNode {
                path: entry.rel_path.clone(),
                display_name: file_name.to_owned(),
                depth,
                kind: TreeNodeKind::File {
                    kind: entry.kind,
                    size_bytes: entry.size_bytes,
                    lock_reason: entry.lock_reason.clone(),
                    is_secret_unlocked,
                },
                is_expanded: false,
                check_state: if entry.is_selected {
                    TriState::Checked
                } else {
                    TriState::Unchecked
                },
            });
            idx += 1;
        }

        Self {
            visible_nodes: Arc::from(nodes),
            total_indexed_files: entries.len(),
            selected_files: project.selected_files_count(),
        }
    }

    #[inline]
    pub fn visible_nodes(&self) -> &[FlatTreeNode] {
        &self.visible_nodes
    }

    #[inline]
    pub fn visible_nodes_arc(&self) -> Arc<[FlatTreeNode]> {
        Arc::clone(&self.visible_nodes)
    }

    #[inline]
    pub const fn total_indexed_files(&self) -> usize {
        self.total_indexed_files
    }

    #[inline]
    pub const fn selected_files(&self) -> usize {
        self.selected_files
    }

    #[inline]
    pub const fn master_check_state(&self) -> TriState {
        if self.selected_files == 0 || self.total_indexed_files == 0 {
            TriState::Unchecked
        } else if self.selected_files >= self.total_indexed_files {
            TriState::Checked
        } else {
            TriState::Indeterminate
        }
    }
}

fn emit_parent_directories_for_entry<'a>(
    entry_idx: usize,
    project: &'a Project,
    collapsed_dirs: &HashSet<String>,
    ignore_collapsed: bool,
    active_dir_stack: &mut Vec<String>,
    dir_prefixes: &mut Vec<&'a str>,
    out_nodes: &mut Vec<FlatTreeNode>,
) -> usize {
    let entries = project.entries();
    let Some(entry) = entries.get(entry_idx) else {
        return 0;
    };
    let rel_path = entry.rel_path.as_str();
    dir_prefixes.clear();
    for (byte_pos, ch) in rel_path.char_indices() {
        if ch == '/'
            && byte_pos > 0
            && let Some(prefix) = rel_path.get(..byte_pos)
        {
            dir_prefixes.push(prefix);
        }
    }

    let common_len = active_dir_stack
        .iter()
        .zip(dir_prefixes.iter())
        .take_while(|(a, b)| a.as_str() == **b)
        .count();
    active_dir_stack.truncate(common_len);

    for &dir_prefix in dir_prefixes.get(common_len..).unwrap_or(&[]) {
        active_dir_stack.push(dir_prefix.to_owned());
        if !ignore_collapsed && is_parent_collapsed(dir_prefix, collapsed_dirs) {
            continue;
        }
        let (dir_name, depth) = extract_name_and_depth(dir_prefix);
        let (file_count, selected_count, size_bytes, check_state) =
            compute_directory_stats(entries, entry_idx, dir_prefix, project);
        let is_expanded = ignore_collapsed || !collapsed_dirs.contains(dir_prefix);

        out_nodes.push(FlatTreeNode {
            path: dir_prefix.to_owned(),
            display_name: dir_name.to_owned(),
            depth,
            kind: TreeNodeKind::Directory { file_count, selected_count, size_bytes },
            is_expanded,
            check_state,
        });

        if !is_expanded && file_count > 0 {
            return file_count;
        }
    }
    0
}

fn compute_directory_stats(
    entries: &[FileEntry],
    start_idx: usize,
    dir_prefix: &str,
    project: &Project,
) -> (usize, usize, u64, TriState) {
    let mut total_files = 0_usize;
    let mut selectable_files = 0_usize;
    let mut selected_files = 0_usize;
    let mut total_bytes = 0_u64;
    let tail = entries.get(start_idx..).unwrap_or(&[]);
    let has_allowed_secrets = !project.preset().explicitly_allowed_secrets.is_empty();

    for entry in tail {
        if !is_inside_directory(&entry.rel_path, dir_prefix) {
            break;
        }
        total_files += 1;
        total_bytes = total_bytes.saturating_add(entry.size_bytes);
        let is_allowed_secret =
            has_allowed_secrets && project.preset().is_secret_explicitly_allowed(&entry.rel_path);
        if entry.can_select(is_allowed_secret) {
            selectable_files += 1;
            if entry.is_selected {
                selected_files += 1;
            }
        }
    }

    let check_state = if selected_files == 0 {
        TriState::Unchecked
    } else if selectable_files > 0 && selected_files == selectable_files {
        TriState::Checked
    } else {
        TriState::Indeterminate
    };

    (total_files, selected_files, total_bytes, check_state)
}

#[inline]
fn is_inside_directory(rel_path: &str, dir_prefix: &str) -> bool {
    rel_path.starts_with(dir_prefix)
        && rel_path.as_bytes().get(dir_prefix.len()).copied() == Some(b'/')
}

#[inline]
fn is_parent_collapsed(path: &str, collapsed_dirs: &HashSet<String>) -> bool {
    if collapsed_dirs.is_empty() {
        return false;
    }
    for (pos, ch) in path.char_indices() {
        if ch == '/'
            && let Some(parent) = path.get(..pos)
            && collapsed_dirs.contains(parent)
        {
            return true;
        }
    }
    false
}

#[inline]
fn is_under_collapsed_dir(file_path: &str, collapsed_dirs: &HashSet<String>) -> bool {
    is_parent_collapsed(file_path, collapsed_dirs)
}

#[inline]
fn extract_name_and_depth(path: &str) -> (&str, u8) {
    let mut slashes = 0_u8;
    let mut last_slash = None;
    for (idx, b) in path.bytes().enumerate() {
        if b == b'/' {
            slashes = slashes.saturating_add(1);
            last_slash = Some(idx);
        }
    }
    let name = match last_slash {
        Some(pos) => path.get(pos + 1..).unwrap_or(path),
        None => path,
    };
    (name, slashes)
}

pub type TreeNodeActionHandler = Arc<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>;
pub type TreePathHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type TreeBulkActionHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Default)]
struct TreeCallbacks {
    toggle_check: Option<TreeNodeActionHandler>,
    toggle_expand: Option<TreePathHandler>,
    toggle_secret_unlock: Option<TreePathHandler>,
    master_toggle_all: Option<TreeBulkActionHandler>,
    toggle_expand_collapse_all: Option<TreeBulkActionHandler>,
    focus_search: Option<TreeBulkActionHandler>,
    clear_search: Option<TreeBulkActionHandler>,
    open_project: Option<TreeBulkActionHandler>,
}

#[derive(IntoElement)]
pub struct FileTreeComponent {
    projection: FlatTreeProjection,
    search_query: String,
    search_focused: bool,
    all_collapsed: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    scroll_handle: Option<UniformListScrollHandle>,
    callbacks: TreeCallbacks,
}

impl fmt::Debug for FileTreeComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileTreeComponent")
            .field("projection", &self.projection)
            .field("search_query", &self.search_query)
            .finish_non_exhaustive()
    }
}

impl FileTreeComponent {
    #[inline]
    pub fn new(projection: FlatTreeProjection, labels: &'static UiLabels) -> Self {
        Self {
            projection,
            search_query: String::new(),
            search_focused: false,
            all_collapsed: false,
            labels,
            theme: active_theme(),
            scroll_handle: None,
            callbacks: TreeCallbacks::default(),
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
    pub fn with_search_state(mut self, query: String, focused: bool, all_collapsed: bool) -> Self {
        self.search_query = query;
        self.search_focused = focused;
        self.all_collapsed = all_collapsed;
        self
    }

    #[must_use]
    #[inline]
    pub fn track_scroll(mut self, handle: UniformListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_check(
        mut self,
        handler: impl Fn(&str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_check = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_expand(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_expand = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_toggle_secret_unlock(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.toggle_secret_unlock = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn with_compact_header_actions(
        mut self,
        on_master_toggle_all: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_toggle_expand_collapse: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_focus_search: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_clear_search: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.master_toggle_all = Some(Arc::new(on_master_toggle_all));
        self.callbacks.toggle_expand_collapse_all = Some(Arc::new(on_toggle_expand_collapse));
        self.callbacks.focus_search = Some(Arc::new(on_focus_search));
        self.callbacks.clear_search = Some(Arc::new(on_clear_search));
        self
    }

    #[must_use]
    #[inline]
    pub fn on_open_project(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callbacks.open_project = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for FileTreeComponent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme;
        let labels = self.labels;
        let nodes = self.projection.visible_nodes_arc();
        let item_count = nodes.len();

        let header = render_compact_tree_toolbar(
            &self.projection,
            &self.search_query,
            self.search_focused,
            self.all_collapsed,
            labels,
            theme,
            &self.callbacks,
        );

        if item_count == 0 {
            let mut open_btn =
                UiButton::new("btn-empty-tree-open-project", labels.btn_open_folder_prompt)
                    .icon(UiIcon::FolderOpen)
                    .variant(ButtonVariant::Primary)
                    .with_theme(theme);
            if let Some(ref cb) = self.callbacks.open_project {
                let cb_clone = Arc::clone(cb);
                open_btn = open_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
            }

            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(theme.colors.bg_panel)
                .child(header)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(theme.spacing.md))
                        .p(px(theme.spacing.xxl))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.colors.text_secondary)
                                .child(labels.empty_project_hint),
                        )
                        .child(open_btn),
                )
                .into_any_element();
        }

        let callbacks = self.callbacks;
        let list = uniform_list(
            "context-packer-file-tree",
            item_count,
            move |range: Range<usize>, _window: &mut Window, _cx: &mut App| {
                let mut rows = Vec::with_capacity(range.len());
                for row_idx in range {
                    if let Some(node) = nodes.get(row_idx) {
                        rows.push(render_tree_row(row_idx, node, labels, theme, &callbacks));
                    }
                }
                rows
            },
        )
        .size_full();

        let tracked =
            if let Some(handle) = self.scroll_handle { list.track_scroll(handle) } else { list };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_panel)
            .overflow_hidden()
            .child(header)
            .child(div().flex_1().overflow_hidden().child(tracked))
            .into_any_element()
    }
}

fn render_compact_tree_toolbar(
    projection: &FlatTreeProjection,
    search_query: &str,
    search_focused: bool,
    all_collapsed: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    callbacks: &TreeCallbacks,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let master_state = projection.master_check_state();
    let (check_background, check_border, check_foreground) = match master_state {
        TriState::Checked => (colors.badge_accent.bg, colors.cyan, colors.cyan),
        TriState::Indeterminate => (colors.badge_warning.bg, colors.warning, colors.warning),
        TriState::Unchecked => (colors.bg_elevated, colors.border_active, colors.text_muted),
    };

    let mut master_check = render_vector_checkbox(
        master_state == TriState::Checked,
        master_state == TriState::Indeterminate,
        16.0,
        check_background,
        check_border,
        check_foreground,
        theme.radius.sm,
    )
    .id("tree-master-checkbox")
    .cursor_pointer();

    if let Some(ref cb) = callbacks.master_toggle_all {
        let cb_clone = Arc::clone(cb);
        master_check = master_check.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    let search_box = render_search_box(search_query, search_focused, labels, theme, callbacks);

    let counter_label = SharedString::from(format!(
        "{}/{}",
        projection.selected_files(),
        projection.total_indexed_files()
    ));

    let fold_icon = if all_collapsed { UiIcon::ChevronsUpDown } else { UiIcon::ChevronsDownUp };
    let mut fold_btn = UiButton::icon_only("tree-fold-toggle-btn", fold_icon)
        .variant(ButtonVariant::Ghost)
        .with_theme(theme)
        .active(all_collapsed);
    if let Some(ref cb) = callbacks.toggle_expand_collapse_all {
        let cb_clone = Arc::clone(cb);
        fold_btn = fold_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    div()
        .w_full()
        .h(px(theme.layout.pane_header_height))
        .px(px(theme.spacing.lg))
        .border_b_1()
        .border_color(colors.border)
        .bg(colors.bg_panel)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(theme.spacing.sm))
        .child(master_check)
        .child(search_box)
        .child(
            div()
                .flex_shrink_0()
                .font_family(crate::theme::FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_secondary)
                .child(counter_label),
        )
        .child(fold_btn)
}

fn render_search_box(
    search_query: &str,
    search_focused: bool,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    callbacks: &TreeCallbacks,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let has_query = !search_query.is_empty();
    let search_display = if has_query {
        SharedString::from(format!("{search_query}▏"))
    } else if search_focused {
        SharedString::from(format!("▏{}", labels.tree_filter_placeholder))
    } else {
        SharedString::from(labels.tree_filter_placeholder)
    };

    let right_search_control = if has_query {
        let mut clear_btn = div()
            .id("tree-search-clear-btn")
            .p(px(2.0))
            .rounded(px(theme.radius.sm))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .child(render_icon(UiIcon::X, 12.0, colors.text_secondary));
        if let Some(ref cb) = callbacks.clear_search {
            let cb_clone = Arc::clone(cb);
            clear_btn = clear_btn.on_click(move |ev, w, c| cb_clone(ev, w, c));
        }
        clear_btn.into_any_element()
    } else {
        div().into_any_element()
    };

    let mut search_box = div()
        .id("tree-interactive-search-box")
        .flex_1()
        .min_w(px(0.0))
        .h(px(theme.layout.control_height))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .border_1()
        .border_color(if search_focused || has_query {
            colors.border_active
        } else {
            colors.border
        })
        .bg(colors.bg_elevated)
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.xs))
        .child(render_icon(UiIcon::Search, 13.0, colors.text_muted))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .font_family(crate::theme::FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if has_query { colors.text_primary } else { colors.text_muted })
                .child(search_display),
        )
        .child(right_search_control);

    if let Some(ref cb) = callbacks.focus_search {
        let cb_clone = Arc::clone(cb);
        search_box = search_box.on_click(move |ev, w, c| cb_clone(ev, w, c));
    }

    search_box
}

fn render_tree_row(
    row_idx: usize,
    node: &FlatTreeNode,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    callbacks: &TreeCallbacks,
) -> impl IntoElement + use<> {
    let colors = &theme.colors;
    let indent_px =
        f32::from(node.depth) * theme.layout.tree_indent_step + theme.layout.tree_indent_base;
    let (chevron_el, check_el, name_el) =
        build_tree_row_interactive_cells(row_idx, node, theme, callbacks);
    let (leading_icon, count_chip, size_label, secret_unlock_btn) =
        render_node_metadata(row_idx, node, labels, theme, callbacks.toggle_secret_unlock.clone());
    let hover_bg = colors.bg_hover;

    div()
        .w_full()
        .h(px(theme.layout.tree_row_height))
        .pl(px(indent_px))
        .pr(px(theme.spacing.lg))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.sm))
        .overflow_hidden()
        .whitespace_nowrap()
        .hover(move |s| s.bg(hover_bg))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.xs))
                .overflow_hidden()
                .child(chevron_el)
                .child(check_el)
                .child(leading_icon)
                .child(name_el)
                .child(count_chip),
        )
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(theme.spacing.xs))
                .child(secret_unlock_btn)
                .child(
                    div()
                        .font_family(crate::theme::FONT_FAMILY_MONO)
                        .text_xs()
                        .text_color(colors.text_muted)
                        .child(size_label),
                ),
        )
}

fn build_tree_row_interactive_cells(
    row_idx: usize,
    node: &FlatTreeNode,
    theme: &'static ThemeTokens,
    callbacks: &TreeCallbacks,
) -> (gpui::Stateful<gpui::Div>, gpui::Stateful<gpui::Div>, gpui::Stateful<gpui::Div>) {
    let colors = &theme.colors;
    let is_dir = node.is_dir();
    let path_for_check = node.path.clone();
    let path_for_expand = node.path.clone();
    let path_for_label = node.path.clone();

    let (check_background, check_border, check_foreground) = match node.check_state {
        TriState::Checked => (colors.badge_accent.bg, colors.cyan, colors.cyan),
        TriState::Indeterminate => (colors.badge_warning.bg, colors.warning, colors.warning),
        TriState::Unchecked => (colors.bg_elevated, colors.border_active, colors.text_muted),
    };

    let mut chevron_el = div()
        .id(("tree-chevron", row_idx))
        .flex_shrink_0()
        .w(px(theme.layout.tree_chevron_width))
        .h(px(theme.layout.tree_chevron_width))
        .flex()
        .items_center()
        .justify_center();
    if is_dir {
        let chev_icon = if node.is_expanded { UiIcon::ChevronDown } else { UiIcon::ChevronRight };
        chevron_el = chevron_el.child(render_icon(chev_icon, 12.0, colors.text_muted));
        if let Some(ref expand_fn) = callbacks.toggle_expand {
            let cb = Arc::clone(expand_fn);
            chevron_el = chevron_el.cursor_pointer().on_click(move |_ev, win, cx| {
                cb(&path_for_expand, win, cx);
            });
        }
    }

    let mut check_el = render_vector_checkbox(
        node.check_state == TriState::Checked,
        node.check_state == TriState::Indeterminate,
        theme.layout.tree_checkbox_size,
        check_background,
        check_border,
        check_foreground,
        theme.radius.sm,
    )
    .id(("tree-check", row_idx))
    .cursor_pointer();
    if let Some(ref check_fn) = callbacks.toggle_check {
        let cb = Arc::clone(check_fn);
        check_el = check_el.on_click(move |_ev, win, cx| {
            cb(&path_for_check, is_dir, win, cx);
        });
    }

    let mut name_el = div()
        .id(("tree-label", row_idx))
        .overflow_hidden()
        .whitespace_nowrap()
        .cursor_pointer()
        .font_family(if is_dir {
            crate::theme::FONT_FAMILY_UI
        } else {
            crate::theme::FONT_FAMILY_MONO
        })
        .text_xs()
        .text_color(if is_dir { colors.text_primary } else { colors.text_secondary })
        .child(SharedString::from(node.display_name.clone()));

    if is_dir {
        if let Some(ref expand_fn) = callbacks.toggle_expand {
            let cb = Arc::clone(expand_fn);
            name_el = name_el.on_click(move |_ev, win, cx| {
                cb(&path_for_label, win, cx);
            });
        }
    } else if let Some(ref check_fn) = callbacks.toggle_check {
        let cb = Arc::clone(check_fn);
        name_el = name_el.on_click(move |_ev, win, cx| {
            cb(&path_for_label, false, win, cx);
        });
    }

    (chevron_el, check_el, name_el)
}

fn render_node_metadata(
    row_idx: usize,
    node: &FlatTreeNode,
    labels: &'static UiLabels,
    theme: &'static ThemeTokens,
    on_secret: Option<TreePathHandler>,
) -> (gpui::AnyElement, gpui::AnyElement, SharedString, gpui::AnyElement) {
    let colors = &theme.colors;
    match &node.kind {
        TreeNodeKind::Directory { file_count, selected_count, size_bytes } => {
            let folder_icon = if node.is_expanded { UiIcon::FolderOpen } else { UiIcon::Folder };
            let folder_glyph =
                render_icon(folder_icon, 14.0, colors.text_secondary).into_any_element();
            let count_chip = div()
                .flex_shrink_0()
                .font_family(crate::theme::FONT_FAMILY_MONO)
                .text_xs()
                .text_color(colors.text_muted)
                .child(format!("{selected_count}/{file_count}"))
                .into_any_element();
            let size_str = SharedString::from(format_bytes_human(*size_bytes));
            (folder_glyph, count_chip, size_str, div().into_any_element())
        }
        TreeNodeKind::File { kind, size_bytes, is_secret_unlocked, .. } => {
            let dot_color = colors.color_for_file_kind(*kind);
            let file_icon = match kind {
                FileKind::Secret => {
                    if *is_secret_unlocked {
                        UiIcon::Unlock
                    } else {
                        UiIcon::Lock
                    }
                }
                FileKind::Documentation | FileKind::Configuration => UiIcon::FileText,
                FileKind::SourceCode | FileKind::Binary | FileKind::SymlinkSkipped => {
                    UiIcon::FileCode
                }
            };
            let file_dot = render_icon(file_icon, 13.0, dot_color).into_any_element();
            let tag_chip = match kind {
                FileKind::Secret => Badge::new(TAG_SECRET, BadgeVariant::Danger)
                    .with_theme(theme)
                    .into_any_element(),
                FileKind::Binary => Badge::new(TAG_BINARY, BadgeVariant::Warning)
                    .with_theme(theme)
                    .into_any_element(),
                FileKind::SymlinkSkipped => Badge::new(TAG_SYMLINK, BadgeVariant::Neutral)
                    .with_theme(theme)
                    .into_any_element(),
                FileKind::SourceCode | FileKind::Configuration | FileKind::Documentation => {
                    div().into_any_element()
                }
            };
            let size_str = SharedString::from(format_bytes_human(*size_bytes));
            let secret_btn = if *kind == FileKind::Secret
                && let Some(secret_fn) = on_secret
            {
                let path_clone = node.path.clone();
                let label = if *is_secret_unlocked {
                    labels.secret_unlocked_session
                } else {
                    labels.secret_locked
                };
                UiButton::new(("secret-unlock", row_idx), label)
                    .variant(if *is_secret_unlocked {
                        ButtonVariant::Danger
                    } else {
                        ButtonVariant::Secondary
                    })
                    .with_theme(theme)
                    .on_click(move |_ev, win, cx| secret_fn(&path_clone, win, cx))
                    .into_any_element()
            } else {
                div().into_any_element()
            };
            (file_dot, tag_chip, size_str, secret_btn)
        }
    }
}
