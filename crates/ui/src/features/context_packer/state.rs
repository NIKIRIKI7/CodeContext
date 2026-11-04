use std::cell::Cell;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use core::packing::aggregates::pack_result::PackResult;
use core::packing::aggregates::project::{Project, compare_paths_folders_first};
use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};
use core::packing::services::pack_service::PackServiceError;
use core::preset::models::preset::Preset;
use core::preset::models::resolution::ResolvedConfig;
use core::preset::ports::config_repo::ConfigPortError;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::shared_kernel::fs::file_kind::split_file_name_and_ext;
use di::AppContainer;

use crate::features::context_packer::components::file_tree::{
    FlatTreeProjection, QuickFilterChip, QuickFilterChipKind, TreeSortMode, TriState,
};
use core::preset::models::template::{
    PromptBlockItem, PromptCategory, SystemPromptMode, default_prompt_blocks,
};

use crate::features::context_packer::components::filter_modal::{
    ExtensionFilterMode, ExtensionStatItem, FilterModalProps, FolderRuleItem, RegexTargetScope,
    SystemPromptModalProps,
};
use crate::features::context_packer::components::rules_editor::RuleToggleKind;
use crate::locale::{AppLocale, UiLabels};
use crate::shared::containers::virtual_text::VirtualTextBuffer;
use crate::shared::primitives::warning_banner::WarningBannerItem;
use crate::theme::ThemeTokens;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PackerUiStatus {
    pub copied_recently: bool,
    pub warnings_dismissed: bool,
    pub search_focused: bool,
    pub regex_filter_open: bool,
    pub filters_popover_open: bool,
    pub meta_modal_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ModalInputFocus {
    #[default]
    None,
    FolderName,
    RegexPattern,
    CustomExtension,
    PromptDraft,
}

#[derive(Debug, Clone)]
pub struct ContextPackerState {
    container: AppContainer,
    project_root: Option<PathBuf>,
    resolved_config: ResolvedConfig,
    project: Option<Project>,
    discovered_ext_counts: BTreeMap<String, usize>,
    discovered_top_folders: Vec<String>,
    hydrated_cache: Vec<(String, String)>,
    pack_result: Option<PackResult>,
    tree_projection: FlatTreeProjection,
    focused_tree_index: Option<usize>,
    sort_mode: TreeSortMode,
    ext_filter_mode: ExtensionFilterMode,
    extension_input_draft: String,
    regex_scope: RegexTargetScope,
    folder_input_draft: String,
    prompt_input_draft: String,
    regex_draft: String,
    regex_is_valid: bool,
    modal_input_focus: ModalInputFocus,
    prompt_modal_open: bool,
    prompt_category_filter: Option<PromptCategory>,
    pending_tree_scroll: Cell<Option<usize>>,
    pending_preview_scroll: Cell<Option<usize>>,
    collapsed_dirs: HashSet<String>,
    search_query: String,
    preview_buffer: VirtualTextBuffer,
    active_chunk_index: usize,
    ui_status: PackerUiStatus,
    cancel_token: CancelToken,
    scan_generation: u64,
}

impl ContextPackerState {
    pub fn new(container: AppContainer, initial_project_root: Option<PathBuf>) -> Self {
        let resolved_config =
            container.preset_service().resolve_project_config(initial_project_root.as_deref());
        let mut state = Self {
            container,
            project_root: None,
            resolved_config,
            project: None,
            discovered_ext_counts: BTreeMap::new(),
            discovered_top_folders: Vec::new(),
            hydrated_cache: Vec::new(),
            pack_result: None,
            tree_projection: FlatTreeProjection::default(),
            focused_tree_index: None,
            sort_mode: TreeSortMode::default(),
            ext_filter_mode: ExtensionFilterMode::default(),
            extension_input_draft: String::new(),
            regex_scope: RegexTargetScope::default(),
            folder_input_draft: String::new(),
            prompt_input_draft: String::new(),
            regex_draft: String::new(),
            regex_is_valid: true,
            modal_input_focus: ModalInputFocus::None,
            prompt_modal_open: false,
            prompt_category_filter: None,
            pending_tree_scroll: Cell::new(None),
            pending_preview_scroll: Cell::new(None),
            collapsed_dirs: HashSet::new(),
            search_query: String::new(),
            preview_buffer: VirtualTextBuffer::default(),
            active_chunk_index: 0,
            ui_status: PackerUiStatus::default(),
            cancel_token: CancelToken::new(),
            scan_generation: 0,
        };
        if let Some(root) = initial_project_root {
            let _ = state.open_project(root);
        }
        state
    }

    pub fn open_project(&mut self, project_root: PathBuf) -> Result<(), PackServiceError> {
        self.cancel_token.cancel();
        self.cancel_token = CancelToken::new();
        self.scan_generation = self.scan_generation.wrapping_add(1);
        self.ui_status = PackerUiStatus::default();
        self.focused_tree_index = None;
        self.discovered_ext_counts.clear();
        self.discovered_top_folders.clear();
        self.folder_input_draft.clear();
        self.extension_input_draft.clear();
        self.prompt_input_draft.clear();
        self.modal_input_focus = ModalInputFocus::None;
        self.prompt_modal_open = false;
        self.pending_tree_scroll.set(Some(0));
        self.pending_preview_scroll.set(Some(0));
        self.search_query.clear();
        self.collapsed_dirs.clear();
        self.resolved_config =
            self.container.preset_service().resolve_project_config(Some(&project_root));
        self.sync_filter_controls_from_preset();
        self.project_root = Some(project_root);
        self.rescan_current_project()
    }

    pub fn rescan_current_project(&mut self) -> Result<(), PackServiceError> {
        let Some(ref root) = self.project_root else {
            return Ok(());
        };
        self.cancel_token.cancel();
        let token = CancelToken::new();
        self.cancel_token = token.clone();
        self.scan_generation = self.scan_generation.wrapping_add(1);
        self.ui_status.copied_recently = false;

        let previous_deselected: HashSet<String> = self
            .project
            .as_ref()
            .filter(|p| p.root() == root.as_path())
            .map(|p| {
                p.entries().iter().filter(|e| !e.is_selected).map(|e| e.rel_path.clone()).collect()
            })
            .unwrap_or_default();

        let preset = self.resolved_config.effective_preset.clone();
        let from_untrusted = self.resolved_config.from_untrusted_project_config();
        let mut indexed_project =
            self.container.pack_service().index_project(root, preset, from_untrusted, &token)?;

        if self.discovered_ext_counts.is_empty()
            || (self.resolved_config.effective_preset.filters.include_extensions.is_empty()
                && self.resolved_config.effective_preset.filters.exclude_extensions.is_empty())
        {
            self.discovered_ext_counts.clear();
            for entry in indexed_project.entries() {
                if let (_, Some(ext)) = split_file_name_and_ext(&entry.rel_path) {
                    *self.discovered_ext_counts.entry(ext.to_ascii_lowercase()).or_insert(0) += 1;
                }
            }
        }
        for entry in indexed_project.entries() {
            if let Some((top_dir, _)) = entry.rel_path.split_once('/')
                && !top_dir.is_empty()
                && !self.discovered_top_folders.iter().any(|d| d == top_dir)
            {
                self.discovered_top_folders.push(top_dir.to_owned());
            }
        }

        for rel_path in &previous_deselected {
            let _ = indexed_project.set_file_selected(rel_path, false);
        }

        self.populate_hydrated_cache_from_project(&indexed_project, &token);
        let packed = self.container.pack_service().pack_from_hydrated_cache(
            &indexed_project,
            &indexed_project.preset().clone(),
            &self.hydrated_cache,
            0,
            &[],
        );
        self.project = Some(indexed_project);
        self.apply_pack_result(packed);
        self.rebuild_tree_projection();
        Ok(())
    }

    pub fn sync_with_disk_if_changed(&mut self) -> bool {
        let Some(ref root) = self.project_root.clone() else {
            return false;
        };
        let token = CancelToken::new();
        let preset = self.resolved_config.effective_preset.clone();
        let from_untrusted = self.resolved_config.from_untrusted_project_config();
        let Ok(fresh_project) =
            self.container.pack_service().index_project(root, preset, from_untrusted, &token)
        else {
            return false;
        };

        let is_unchanged = self.project.as_ref().is_some_and(|curr| {
            curr.entries().len() == fresh_project.entries().len()
                && curr.entries().iter().zip(fresh_project.entries().iter()).all(|(a, b)| {
                    a.rel_path == b.rel_path && a.size_bytes == b.size_bytes && a.kind == b.kind
                })
        });

        if is_unchanged {
            return false;
        }

        let _ = self.rescan_current_project();
        true
    }

    pub fn apply_indexed_and_packed(
        &mut self,
        project: Project,
        pack_result: PackResult,
        hydrated_files: Vec<(String, String)>,
    ) {
        self.hydrated_cache = hydrated_files;
        self.hydrated_cache.sort_unstable_by(|a, b| compare_paths_folders_first(&a.0, &b.0));
        self.project = Some(project);
        self.apply_pack_result(pack_result);
        self.rebuild_tree_projection();
    }

    pub fn toggle_tree_node_checkbox(&mut self, path: &str, is_dir: bool) -> usize {
        self.ui_status.copied_recently = false;
        let target_select = self
            .tree_projection
            .visible_nodes()
            .iter()
            .find(|n| n.path == path && n.is_dir() == is_dir)
            .is_none_or(|n| n.check_state.next_target_selection());

        if is_dir {
            self.set_directory_selected(path, target_select)
        } else {
            usize::from(self.set_file_selected(path, target_select))
        }
    }

    pub fn toggle_select_all_files(&mut self) -> usize {
        let target_select = self.tree_projection.master_check_state() != TriState::Checked;
        self.set_directory_selected("", target_select)
    }

    pub fn set_directory_selected(&mut self, dir_prefix: &str, selected: bool) -> usize {
        let Some(ref mut project) = self.project else {
            return 0;
        };
        let changed = project.set_directory_selected(dir_prefix, selected);
        if changed > 0 {
            self.ui_status.copied_recently = false;
            self.rebuild_tree_projection();
            self.repack_from_memory_cache();
        }
        changed
    }

    pub fn set_file_selected(&mut self, rel_path: &str, selected: bool) -> bool {
        let Some(ref mut project) = self.project else {
            return false;
        };
        let changed = project.set_file_selected(rel_path, selected);
        if changed {
            self.ui_status.copied_recently = false;
            self.rebuild_tree_projection();
            self.repack_from_memory_cache();
        }
        changed
    }

    pub fn toggle_directory_expanded(&mut self, dir_path: &str) {
        if !self.collapsed_dirs.remove(dir_path) {
            self.collapsed_dirs.insert(dir_path.to_owned());
        }
        self.rebuild_tree_projection();
    }

    #[inline]
    pub fn are_all_dirs_collapsed(&self) -> bool {
        !self.collapsed_dirs.is_empty()
    }

    pub fn toggle_expand_collapse_all(&mut self) {
        if self.collapsed_dirs.is_empty() {
            self.collapse_all_directories();
        } else {
            self.expand_all_directories();
        }
    }

    pub fn expand_all_directories(&mut self) {
        if !self.collapsed_dirs.is_empty() {
            self.collapsed_dirs.clear();
            self.rebuild_tree_projection();
        }
    }

    pub fn collapse_all_directories(&mut self) {
        let Some(ref project) = self.project else {
            return;
        };
        for entry in project.entries() {
            for (pos, ch) in entry.rel_path.char_indices() {
                if ch == '/'
                    && pos > 0
                    && let Some(dir) = entry.rel_path.get(..pos)
                {
                    self.collapsed_dirs.insert(dir.to_owned());
                }
            }
        }
        self.rebuild_tree_projection();
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.rebuild_tree_projection();
    }

    pub fn append_search_char(&mut self, ch: char) {
        self.ui_status.search_focused = true;
        self.search_query.push(ch);
        self.rebuild_tree_projection();
    }

    pub fn pop_search_char(&mut self) -> bool {
        if self.search_query.pop().is_some() {
            self.rebuild_tree_projection();
            return true;
        }
        false
    }

    pub fn clear_search_query(&mut self) {
        self.ui_status.search_focused = false;
        if !self.search_query.is_empty() {
            self.search_query.clear();
            self.rebuild_tree_projection();
        }
    }

    #[inline]
    pub const fn set_search_focused(&mut self, focused: bool) {
        self.ui_status.search_focused = focused;
    }

    #[inline]
    pub const fn toggle_filters_popover(&mut self) {
        self.ui_status.filters_popover_open = !self.ui_status.filters_popover_open;
    }

    #[inline]
    pub const fn request_open_meta_modal(&mut self) {
        self.ui_status.meta_modal_requested = true;
    }

    #[inline]
    pub const fn take_meta_modal_request(&mut self) -> bool {
        let requested = self.ui_status.meta_modal_requested;
        self.ui_status.meta_modal_requested = false;
        requested
    }

    #[inline]
    pub const fn is_prompt_modal_open(&self) -> bool {
        self.prompt_modal_open
    }

    #[inline]
    pub const fn set_prompt_modal_open(&mut self, open: bool) {
        self.prompt_modal_open = open;
        if open {
            self.ui_status.regex_filter_open = false;
            self.modal_input_focus = ModalInputFocus::None;
        }
    }

    #[inline]
    pub const fn prompt_category_filter(&self) -> Option<PromptCategory> {
        self.prompt_category_filter
    }

    #[inline]
    pub const fn set_prompt_category_filter(&mut self, category: Option<PromptCategory>) {
        self.prompt_category_filter = category;
    }

    pub fn select_system_prompt_mode(&mut self, mode: SystemPromptMode) {
        self.ui_status.copied_recently = false;
        let tpl = &mut self.resolved_config.effective_preset.template;
        for b in &mut tpl.prompt_blocks {
            b.enabled = false;
        }
        match mode {
            SystemPromptMode::None => {
                tpl.preamble.clear();
                tpl.system_prompt_preset = SystemPromptMode::None;
            }
            SystemPromptMode::SearchReplacePatch => {
                if let Some(b) = tpl.prompt_blocks.iter_mut().find(|b| b.id == "tech_patcher_full")
                {
                    b.enabled = true;
                }
                tpl.sync_preamble_from_blocks();
                tpl.system_prompt_preset = SystemPromptMode::SearchReplacePatch;
            }
            SystemPromptMode::CanonicalFullFiles => {
                if let Some(b) =
                    tpl.prompt_blocks.iter_mut().find(|b| b.id == "tech_canonical_full")
                {
                    b.enabled = true;
                }
                tpl.sync_preamble_from_blocks();
                tpl.system_prompt_preset = SystemPromptMode::CanonicalFullFiles;
            }
            SystemPromptMode::UnifiedDiffPatch => {
                if let Some(b) = tpl.prompt_blocks.iter_mut().find(|b| b.id == "tech_unified_diff")
                {
                    b.enabled = true;
                }
                tpl.sync_preamble_from_blocks();
                tpl.system_prompt_preset = SystemPromptMode::UnifiedDiffPatch;
            }
            SystemPromptMode::Custom => {
                tpl.system_prompt_preset = SystemPromptMode::Custom;
            }
        }
        self.repack_from_memory_cache();
    }

    pub fn toggle_prompt_block(&mut self, index: usize) {
        let tpl = &mut self.resolved_config.effective_preset.template;
        if let Some(block) = tpl.prompt_blocks.get_mut(index) {
            block.enabled = !block.enabled;
            tpl.sync_preamble_from_blocks();
            self.ui_status.copied_recently = false;
            self.repack_from_memory_cache();
        }
    }

    pub fn move_prompt_block_up(&mut self, index: usize) {
        let tpl = &mut self.resolved_config.effective_preset.template;
        if index > 0 && index < tpl.prompt_blocks.len() {
            tpl.prompt_blocks.swap(index - 1, index);
            tpl.sync_preamble_from_blocks();
            self.ui_status.copied_recently = false;
            self.repack_from_memory_cache();
        }
    }

    pub fn move_prompt_block_down(&mut self, index: usize) {
        let tpl = &mut self.resolved_config.effective_preset.template;
        if index + 1 < tpl.prompt_blocks.len() {
            tpl.prompt_blocks.swap(index, index + 1);
            tpl.sync_preamble_from_blocks();
            self.ui_status.copied_recently = false;
            self.repack_from_memory_cache();
        }
    }

    pub fn add_custom_prompt_block(&mut self, category: PromptCategory, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        let tpl = &mut self.resolved_config.effective_preset.template;
        let cat_count = tpl.prompt_blocks.iter().filter(|b| b.category == category).count() + 1;
        let first_line = trimmed.lines().next().unwrap_or("Custom").trim();
        let short_title = if first_line.chars().count() > 42 {
            let prefix: String = first_line.chars().take(42).collect();
            format!("{prefix}…")
        } else {
            first_line.to_owned()
        };

        let cat_tag = match category {
            PromptCategory::Patcher => "patcher",
            PromptCategory::Architecture => "arch",
            PromptCategory::Behavior => "behavior",
            PromptCategory::Custom => "custom",
        };
        let unique_id = format!("custom_{cat_tag}_{cat_count}");

        tpl.prompt_blocks.push(PromptBlockItem {
            id: unique_id,
            title: short_title,
            category,
            content: trimmed.to_owned(),
            enabled: true,
        });

        tpl.sync_preamble_from_blocks();
        self.ui_status.copied_recently = false;
        self.repack_from_memory_cache();
    }

    pub fn set_custom_system_prompt_text(&mut self, text: &str) {
        let category = self.prompt_category_filter.unwrap_or(PromptCategory::Custom);
        self.add_custom_prompt_block(category, text);
    }

    pub fn delete_prompt_block(&mut self, index: usize) {
        let tpl = &mut self.resolved_config.effective_preset.template;
        if index < tpl.prompt_blocks.len() {
            tpl.prompt_blocks.remove(index);
            tpl.sync_preamble_from_blocks();
            self.ui_status.copied_recently = false;
            self.repack_from_memory_cache();
        }
    }

    #[inline]
    pub const fn set_prompt_input_focused(&mut self, focused: bool) {
        self.modal_input_focus =
            if focused { ModalInputFocus::PromptDraft } else { ModalInputFocus::None };
    }

    #[inline]
    pub const fn is_prompt_input_focused(&self) -> bool {
        matches!(self.modal_input_focus, ModalInputFocus::PromptDraft)
    }

    pub fn append_prompt_input_char(&mut self, ch: char) {
        self.modal_input_focus = ModalInputFocus::PromptDraft;
        self.prompt_input_draft.push(ch);
    }

    pub fn pop_prompt_input_char(&mut self) -> bool {
        self.prompt_input_draft.pop().is_some()
    }

    pub fn submit_manual_prompt_draft(&mut self) {
        let draft = std::mem::take(&mut self.prompt_input_draft);
        let trimmed = draft.trim();
        if trimmed.is_empty() {
            return;
        }
        let category = self.prompt_category_filter.unwrap_or(PromptCategory::Custom);
        self.add_custom_prompt_block(category, trimmed);
    }

    pub fn reset_system_prompt_to_builtin(&mut self) {
        self.ui_status.copied_recently = false;
        let tpl = &mut self.resolved_config.effective_preset.template;
        tpl.prompt_blocks = default_prompt_blocks();
        tpl.preamble.clear();
        tpl.system_prompt_preset = SystemPromptMode::None;
        self.repack_from_memory_cache();
    }

    pub fn build_system_prompt_modal_props(
        &self,
        labels: &'static UiLabels,
        theme: &'static ThemeTokens,
    ) -> SystemPromptModalProps {
        let tpl = &self.resolved_config.effective_preset.template;
        SystemPromptModalProps {
            active_category: self.prompt_category_filter,
            prompt_input_draft: self.prompt_input_draft.clone(),
            prompt_input_focused: self.is_prompt_input_focused(),
            blocks: tpl.prompt_blocks.clone(),
            effective_prompt_text: tpl.effective_preamble().to_owned(),
            labels,
            theme,
        }
    }

    #[inline]
    pub const fn toggle_regex_filter_panel(&mut self) {
        self.ui_status.regex_filter_open = !self.ui_status.regex_filter_open;
        if self.ui_status.regex_filter_open {
            self.prompt_modal_open = false;
        } else {
            self.modal_input_focus = ModalInputFocus::None;
        }
    }

    #[inline]
    pub const fn set_regex_filter_open(&mut self, open: bool) {
        self.ui_status.regex_filter_open = open;
        if open {
            self.prompt_modal_open = false;
        } else {
            self.modal_input_focus = ModalInputFocus::None;
        }
    }

    #[inline]
    pub const fn set_regex_input_focused(&mut self, focused: bool) {
        self.modal_input_focus =
            if focused { ModalInputFocus::RegexPattern } else { ModalInputFocus::None };
    }

    #[inline]
    pub const fn is_regex_input_focused(&self) -> bool {
        matches!(self.modal_input_focus, ModalInputFocus::RegexPattern)
    }

    #[inline]
    pub const fn set_folder_input_focused(&mut self, focused: bool) {
        self.modal_input_focus =
            if focused { ModalInputFocus::FolderName } else { ModalInputFocus::None };
    }

    #[inline]
    pub const fn is_folder_input_focused(&self) -> bool {
        matches!(self.modal_input_focus, ModalInputFocus::FolderName)
    }

    pub fn append_folder_input_char(&mut self, ch: char) {
        self.modal_input_focus = ModalInputFocus::FolderName;
        self.folder_input_draft.push(ch);
    }

    pub fn pop_folder_input_char(&mut self) -> bool {
        self.folder_input_draft.pop().is_some()
    }

    #[inline]
    pub const fn set_extension_input_focused(&mut self, focused: bool) {
        self.modal_input_focus =
            if focused { ModalInputFocus::CustomExtension } else { ModalInputFocus::None };
    }

    #[inline]
    pub const fn is_extension_input_focused(&self) -> bool {
        matches!(self.modal_input_focus, ModalInputFocus::CustomExtension)
    }

    pub fn append_extension_input_char(&mut self, ch: char) {
        self.modal_input_focus = ModalInputFocus::CustomExtension;
        self.extension_input_draft.push(ch);
    }

    pub fn pop_extension_input_char(&mut self) -> bool {
        self.extension_input_draft.pop().is_some()
    }

    pub fn submit_manual_extension(&mut self) {
        let raw = std::mem::take(&mut self.extension_input_draft);
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return;
        }

        for part in trimmed.split([',', ';', ' ']) {
            let clean = part.trim().trim_start_matches('.').to_ascii_lowercase();
            if clean.is_empty() {
                continue;
            }
            self.discovered_ext_counts.entry(clean.clone()).or_insert(0);

            if self.ext_filter_mode == ExtensionFilterMode::All {
                self.ext_filter_mode = ExtensionFilterMode::IncludeOnly;
            }

            let filters = &mut self.resolved_config.effective_preset.filters;
            match self.ext_filter_mode {
                ExtensionFilterMode::ExcludeSelected => {
                    if !filters.exclude_extensions.iter().any(|e| e.eq_ignore_ascii_case(&clean)) {
                        filters.exclude_extensions.push(clean);
                    }
                }
                ExtensionFilterMode::All | ExtensionFilterMode::IncludeOnly => {
                    if !filters.include_extensions.iter().any(|e| e.eq_ignore_ascii_case(&clean)) {
                        filters.include_extensions.push(clean);
                    }
                }
            }
        }
        let _ = self.rescan_current_project();
    }

    pub fn submit_manual_folder_exclusion(&mut self) {
        let clean = self.folder_input_draft.trim().trim_matches(['/', '\\']).to_owned();
        self.folder_input_draft.clear();
        if clean.is_empty() || clean == ".git" || clean == ".ctxpack" {
            return;
        }
        if clean.contains('/') || clean.contains('*') {
            let globs = &mut self.resolved_config.effective_preset.folder_filters.exclude_globs;
            if !globs.iter().any(|g| g == &clean) {
                globs.push(clean);
            }
        } else {
            let names = &mut self.resolved_config.effective_preset.folder_filters.exclude_names;
            if !names.iter().any(|n| n.eq_ignore_ascii_case(&clean)) {
                names.push(clean);
            }
        }
        let _ = self.rescan_current_project();
    }

    pub fn set_extension_filter_mode(&mut self, mode: ExtensionFilterMode) {
        if self.ext_filter_mode == mode {
            return;
        }
        let filters = &mut self.resolved_config.effective_preset.filters;
        match mode {
            ExtensionFilterMode::All => {
                filters.include_extensions.clear();
                filters.exclude_extensions.clear();
            }
            ExtensionFilterMode::IncludeOnly => {
                if filters.include_extensions.is_empty() && !filters.exclude_extensions.is_empty() {
                    filters.include_extensions = std::mem::take(&mut filters.exclude_extensions);
                } else {
                    filters.exclude_extensions.clear();
                }
            }
            ExtensionFilterMode::ExcludeSelected => {
                if filters.exclude_extensions.is_empty() && !filters.include_extensions.is_empty() {
                    filters.exclude_extensions = std::mem::take(&mut filters.include_extensions);
                } else {
                    filters.include_extensions.clear();
                }
            }
        }
        self.ext_filter_mode = mode;
        let _ = self.rescan_current_project();
    }

    pub fn toggle_extension_in_modal(&mut self, ext: &str) {
        let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        if clean.is_empty() {
            return;
        }
        if self.ext_filter_mode == ExtensionFilterMode::All {
            self.ext_filter_mode = ExtensionFilterMode::IncludeOnly;
        }
        let filters = &mut self.resolved_config.effective_preset.filters;
        let target_vec = match self.ext_filter_mode {
            ExtensionFilterMode::ExcludeSelected => &mut filters.exclude_extensions,
            ExtensionFilterMode::All | ExtensionFilterMode::IncludeOnly => {
                &mut filters.include_extensions
            }
        };
        if let Some(pos) = target_vec.iter().position(|e| e.eq_ignore_ascii_case(&clean)) {
            target_vec.remove(pos);
        } else {
            target_vec.push(clean);
        }
        if filters.include_extensions.is_empty() && filters.exclude_extensions.is_empty() {
            self.ext_filter_mode = ExtensionFilterMode::All;
        }
        let _ = self.rescan_current_project();
    }

    pub fn select_regex_scope(&mut self, scope: RegexTargetScope) {
        self.regex_scope = scope;
        self.apply_regex_draft_to_preset();
    }

    pub fn toggle_regex_case_sensitive(&mut self) {
        let preset = &mut self.resolved_config.effective_preset;
        match self.regex_scope {
            RegexTargetScope::FilesExclude | RegexTargetScope::FilesInclude => {
                preset.filters.regex_case_sensitive = !preset.filters.regex_case_sensitive;
            }
            RegexTargetScope::FoldersExclude | RegexTargetScope::FoldersInclude => {
                preset.folder_filters.regex_case_sensitive =
                    !preset.folder_filters.regex_case_sensitive;
            }
        }
        let _ = self.rescan_current_project();
    }

    pub fn append_regex_char(&mut self, ch: char) {
        self.modal_input_focus = ModalInputFocus::RegexPattern;
        self.regex_draft.push(ch);
        self.apply_regex_draft_to_preset();
    }

    pub fn pop_regex_char(&mut self) -> bool {
        if self.regex_draft.pop().is_some() {
            self.apply_regex_draft_to_preset();
            return true;
        }
        false
    }

    pub fn clear_regex_draft(&mut self) {
        if !self.regex_draft.is_empty() {
            self.regex_draft.clear();
            self.apply_regex_draft_to_preset();
        }
    }

    pub fn apply_regex_preset_pattern(&mut self, pattern: &str) {
        if self.regex_draft == pattern {
            self.regex_draft.clear();
        } else {
            pattern.clone_into(&mut self.regex_draft);
        }
        self.apply_regex_draft_to_preset();
    }

    pub fn reset_all_filters(&mut self) {
        let preset = &mut self.resolved_config.effective_preset;
        preset.filters = core::preset::models::preset::FileFilters::default();
        preset.folder_filters = core::preset::models::preset::FolderFilters::default();
        self.ext_filter_mode = ExtensionFilterMode::All;
        self.regex_scope = RegexTargetScope::FilesExclude;
        self.regex_draft.clear();
        self.regex_is_valid = true;
        let _ = self.rescan_current_project();
    }

    pub fn active_custom_filter_rules_count(&self) -> usize {
        let preset = &self.resolved_config.effective_preset;
        let ext_rules =
            preset.filters.include_extensions.len() + preset.filters.exclude_extensions.len();
        let regex_rules = usize::from(preset.filters.include_regex.is_some())
            + usize::from(preset.filters.exclude_regex.is_some())
            + usize::from(preset.folder_filters.include_regex_folders.is_some())
            + usize::from(preset.folder_filters.exclude_regex_folders.is_some());
        let folder_rules = preset.folder_filters.only_folders.len()
            + preset.folder_filters.exclude_globs.len()
            + preset.folder_filters.folder_overrides.len();
        ext_rules + regex_rules + folder_rules
    }

    pub fn build_filter_modal_props(
        &self,
        labels: &'static UiLabels,
        theme: &'static ThemeTokens,
    ) -> FilterModalProps {
        let preset = &self.resolved_config.effective_preset;
        let mut ext_map = self.discovered_ext_counts.clone();
        for ext in
            preset.filters.include_extensions.iter().chain(&preset.filters.exclude_extensions)
        {
            ext_map.entry(ext.clone()).or_insert(0);
        }
        let mut sorted_exts: Vec<(String, usize)> = ext_map.into_iter().collect();
        sorted_exts.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let extensions = sorted_exts
            .into_iter()
            .map(|(ext, count)| {
                let selected = match self.ext_filter_mode {
                    ExtensionFilterMode::All => false,
                    ExtensionFilterMode::IncludeOnly => preset
                        .filters
                        .include_extensions
                        .iter()
                        .any(|e| e.eq_ignore_ascii_case(&ext)),
                    ExtensionFilterMode::ExcludeSelected => preset
                        .filters
                        .exclude_extensions
                        .iter()
                        .any(|e| e.eq_ignore_ascii_case(&ext)),
                };
                ExtensionStatItem { ext, count, selected }
            })
            .collect();

        let mut all_folder_names: Vec<String> = vec![
            "target".to_owned(),
            "node_modules".to_owned(),
            "dist".to_owned(),
            "build".to_owned(),
            "tests".to_owned(),
            "benches".to_owned(),
            "docs".to_owned(),
            "assets".to_owned(),
            ".venv".to_owned(),
            "vendor".to_owned(),
        ];
        for top_dir in &self.discovered_top_folders {
            if top_dir != ".git"
                && top_dir != ".ctxpack"
                && !all_folder_names.iter().any(|n| n.eq_ignore_ascii_case(top_dir))
            {
                all_folder_names.push(top_dir.clone());
            }
        }
        for exc_name in &preset.folder_filters.exclude_names {
            if exc_name != ".git"
                && exc_name != ".ctxpack"
                && !all_folder_names.iter().any(|n| n.eq_ignore_ascii_case(exc_name))
            {
                all_folder_names.push(exc_name.clone());
            }
        }
        for exc_glob in &preset.folder_filters.exclude_globs {
            if !all_folder_names.iter().any(|n| n == exc_glob) {
                all_folder_names.push(exc_glob.clone());
            }
        }

        let folders = all_folder_names
            .into_iter()
            .map(|name| {
                let is_excluded = preset
                    .folder_filters
                    .exclude_names
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(&name))
                    || preset.folder_filters.exclude_globs.iter().any(|g| g == &name);
                FolderRuleItem { name, is_excluded, is_project_folder: true }
            })
            .collect();

        let regex_case_sensitive = match self.regex_scope {
            RegexTargetScope::FilesExclude | RegexTargetScope::FilesInclude => {
                preset.filters.regex_case_sensitive
            }
            RegexTargetScope::FoldersExclude | RegexTargetScope::FoldersInclude => {
                preset.folder_filters.regex_case_sensitive
            }
        };

        FilterModalProps {
            ext_mode: self.ext_filter_mode,
            extensions,
            extension_input_draft: self.extension_input_draft.clone(),
            extension_input_focused: self.is_extension_input_focused(),
            folders,
            folder_input_draft: self.folder_input_draft.clone(),
            folder_input_focused: self.is_folder_input_focused(),
            regex_scope: self.regex_scope,
            regex_pattern: self.regex_draft.clone(),
            regex_focused: self.is_regex_input_focused(),
            regex_case_sensitive,
            regex_is_valid: self.regex_is_valid,
            active_rules_count: self.active_custom_filter_rules_count(),
            selected_files_count: self.tree_projection.selected_files(),
            total_files_count: self.tree_projection.total_indexed_files(),
            labels,
            theme,
        }
    }

    fn sync_filter_controls_from_preset(&mut self) {
        let preset = &self.resolved_config.effective_preset;
        self.ext_filter_mode = if !preset.filters.include_extensions.is_empty() {
            ExtensionFilterMode::IncludeOnly
        } else if !preset.filters.exclude_extensions.is_empty() {
            ExtensionFilterMode::ExcludeSelected
        } else {
            ExtensionFilterMode::All
        };

        if let Some(ref pat) = preset.filters.exclude_regex {
            self.regex_scope = RegexTargetScope::FilesExclude;
            self.regex_draft.clone_from(pat);
        } else if let Some(ref pat) = preset.filters.include_regex {
            self.regex_scope = RegexTargetScope::FilesInclude;
            self.regex_draft.clone_from(pat);
        } else if let Some(ref pat) = preset.folder_filters.exclude_regex_folders {
            self.regex_scope = RegexTargetScope::FoldersExclude;
            self.regex_draft.clone_from(pat);
        } else if let Some(ref pat) = preset.folder_filters.include_regex_folders {
            self.regex_scope = RegexTargetScope::FoldersInclude;
            self.regex_draft.clone_from(pat);
        } else {
            self.regex_draft.clear();
        }
    }

    fn apply_regex_draft_to_preset(&mut self) {
        let preset = &mut self.resolved_config.effective_preset;
        let case_sensitive = match self.regex_scope {
            RegexTargetScope::FilesExclude | RegexTargetScope::FilesInclude => {
                preset.filters.regex_case_sensitive
            }
            RegexTargetScope::FoldersExclude | RegexTargetScope::FoldersInclude => {
                preset.folder_filters.regex_case_sensitive
            }
        };
        let trimmed = self.regex_draft.trim();
        self.regex_is_valid = AppContainer::is_valid_regex(trimmed, case_sensitive);
        if !self.regex_is_valid {
            return;
        }
        preset.filters.include_regex = None;
        preset.filters.exclude_regex = None;
        preset.folder_filters.include_regex_folders = None;
        preset.folder_filters.exclude_regex_folders = None;
        if !trimmed.is_empty() {
            let owned = Some(trimmed.to_owned());
            match self.regex_scope {
                RegexTargetScope::FilesExclude => preset.filters.exclude_regex = owned,
                RegexTargetScope::FilesInclude => preset.filters.include_regex = owned,
                RegexTargetScope::FoldersExclude => {
                    preset.folder_filters.exclude_regex_folders = owned;
                }
                RegexTargetScope::FoldersInclude => {
                    preset.folder_filters.include_regex_folders = owned;
                }
            }
        }
        let _ = self.rescan_current_project();
    }

    pub fn toggle_quick_filter_chip(&mut self, kind: &QuickFilterChipKind) {
        match kind {
            QuickFilterChipKind::Extension(ext) => {
                self.resolved_config.effective_preset.toggle_extension_filter(ext);
            }
            QuickFilterChipKind::ExcludedFolder(folder) => {
                let globs = &mut self.resolved_config.effective_preset.folder_filters.exclude_globs;
                if let Some(pos) = globs.iter().position(|g| g == folder) {
                    globs.remove(pos);
                } else {
                    self.resolved_config.effective_preset.toggle_excluded_folder_name(folder);
                }
            }
        }
        let _ = self.rescan_current_project();
    }

    pub fn build_quick_filter_chips(&self) -> Vec<QuickFilterChip> {
        let mut ext_counts: BTreeMap<String, usize> = BTreeMap::new();
        if let Some(ref proj) = self.project {
            for entry in proj.entries() {
                if let (_, Some(ext)) = split_file_name_and_ext(&entry.rel_path) {
                    let lower = ext.to_ascii_lowercase();
                    *ext_counts.entry(lower).or_insert(0) += 1;
                }
            }
        }
        let preset = &self.resolved_config.effective_preset;
        for active_ext in &preset.filters.include_extensions {
            ext_counts.entry(active_ext.clone()).or_insert(0);
        }

        let mut sorted_exts: Vec<(String, usize)> = ext_counts.into_iter().collect();
        sorted_exts.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut chips = Vec::with_capacity(sorted_exts.len() + 3);
        for (ext, count) in sorted_exts {
            let active =
                preset.filters.include_extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext));
            let label = if count > 0 { format!(".{ext} {count}") } else { format!(".{ext}") };
            chips.push(QuickFilterChip {
                label,
                active,
                kind: QuickFilterChipKind::Extension(ext),
            });
        }

        for folder_name in ["target", "node_modules", "dist"] {
            let is_excluded = preset
                .folder_filters
                .exclude_names
                .iter()
                .any(|n| n.eq_ignore_ascii_case(folder_name));
            chips.push(QuickFilterChip {
                label: format!("✕ {folder_name}"),
                active: is_excluded,
                kind: QuickFilterChipKind::ExcludedFolder(folder_name.to_owned()),
            });
        }
        chips
    }

    #[inline]
    pub const fn focused_tree_index(&self) -> Option<usize> {
        self.focused_tree_index
    }

    #[inline]
    pub const fn sort_mode(&self) -> TreeSortMode {
        self.sort_mode
    }

    #[inline]
    pub fn take_pending_tree_scroll(&self) -> Option<usize> {
        self.pending_tree_scroll.take()
    }

    #[inline]
    pub fn take_pending_preview_scroll(&self) -> Option<usize> {
        self.pending_preview_scroll.take()
    }

    pub fn toggle_tree_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.toggle();
        self.focused_tree_index = None;
        self.pending_tree_scroll.set(Some(0));
        self.rebuild_tree_projection();
    }

    pub fn select_file_in_tree_and_jump(&mut self, row_idx: usize, rel_path: &str) {
        self.ui_status.search_focused = false;
        self.focused_tree_index = Some(row_idx);
        let _ = self.jump_to_file_in_preview(rel_path);
    }

    pub fn jump_to_file_in_preview(&mut self, rel_path: &str) -> Option<usize> {
        self.ui_status.search_focused = false;
        let prefix = self.resolved_config.effective_preset.template.file_header_prefix.clone();
        let suffix = self.resolved_config.effective_preset.template.file_header_suffix.clone();

        if let Some(line_idx) =
            self.preview_buffer.find_file_header_line(rel_path, &prefix, &suffix)
        {
            self.pending_preview_scroll.set(Some(line_idx));
            return Some(line_idx);
        }

        if let Some(ref pack_res) = self.pack_result
            && pack_res.chunks.len() > 1
        {
            let target_chunk = pack_res.chunks.iter().enumerate().find_map(|(c_idx, chunk)| {
                if c_idx == self.active_chunk_index {
                    return None;
                }
                let buf = VirtualTextBuffer::new(chunk.content.as_str());
                buf.find_file_header_line(rel_path, &prefix, &suffix).map(|line| (c_idx, buf, line))
            });
            if let Some((c_idx, buf, line_idx)) = target_chunk {
                self.active_chunk_index = c_idx;
                self.ui_status.copied_recently = false;
                self.preview_buffer = buf;
                self.pending_preview_scroll.set(Some(line_idx));
                return Some(line_idx);
            }
        }
        None
    }

    pub fn move_tree_focus_down(&mut self) -> bool {
        let len = self.tree_projection.visible_nodes().len();
        if len == 0 {
            return false;
        }
        let next_idx =
            self.focused_tree_index.map_or(0, |curr| curr.saturating_add(1).min(len - 1));
        if self.focused_tree_index == Some(next_idx) {
            return false;
        }
        self.focused_tree_index = Some(next_idx);
        self.pending_tree_scroll.set(Some(next_idx));
        true
    }

    pub fn move_tree_focus_up(&mut self) -> bool {
        let len = self.tree_projection.visible_nodes().len();
        if len == 0 {
            return false;
        }
        let prev_idx = self.focused_tree_index.map_or(0, |curr| curr.saturating_sub(1));
        if self.focused_tree_index == Some(prev_idx) {
            return false;
        }
        self.focused_tree_index = Some(prev_idx);
        self.pending_tree_scroll.set(Some(prev_idx));
        true
    }

    pub fn toggle_focused_tree_node_checkbox(&mut self) -> usize {
        let Some(idx) = self.focused_tree_index else {
            return 0;
        };
        let Some(node) = self.tree_projection.visible_nodes().get(idx) else {
            return 0;
        };
        let path = node.path.clone();
        let is_dir = node.is_dir();
        self.toggle_tree_node_checkbox(&path, is_dir)
    }

    pub fn expand_or_step_into_focused_tree_node(&mut self) -> bool {
        let Some(idx) = self.focused_tree_index else {
            return false;
        };
        let Some(node) = self.tree_projection.visible_nodes().get(idx) else {
            return false;
        };
        if node.is_dir() {
            if !node.is_expanded {
                let dir_path = node.path.clone();
                self.toggle_directory_expanded(&dir_path);
                return true;
            }
            return self.move_tree_focus_down();
        }
        let file_path = node.path.clone();
        self.select_file_in_tree_and_jump(idx, &file_path);
        true
    }

    pub fn collapse_or_step_out_focused_tree_node(&mut self) -> bool {
        let Some(idx) = self.focused_tree_index else {
            return false;
        };
        let Some(node) = self.tree_projection.visible_nodes().get(idx) else {
            return false;
        };
        if node.is_dir() && node.is_expanded {
            let dir_path = node.path.clone();
            self.toggle_directory_expanded(&dir_path);
            return true;
        }
        let Some((parent_dir, _)) = node.path.rsplit_once('/') else {
            return false;
        };
        if let Some(parent_idx) = self
            .tree_projection
            .visible_nodes()
            .iter()
            .position(|n| n.is_dir() && n.path == parent_dir)
        {
            self.focused_tree_index = Some(parent_idx);
            self.pending_tree_scroll.set(Some(parent_idx));
            return true;
        }
        false
    }

    pub fn activate_focused_tree_node(&mut self) -> bool {
        let Some(idx) = self.focused_tree_index else {
            return false;
        };
        let Some(node) = self.tree_projection.visible_nodes().get(idx) else {
            return false;
        };
        if node.is_dir() {
            let dir_path = node.path.clone();
            self.toggle_directory_expanded(&dir_path);
        } else {
            let file_path = node.path.clone();
            self.select_file_in_tree_and_jump(idx, &file_path);
        }
        true
    }

    pub fn toggle_secret_unlock_in_session(&mut self, rel_path: &str) {
        let is_currently_unlocked =
            self.resolved_config.effective_preset.is_secret_explicitly_allowed(rel_path);
        if is_currently_unlocked {
            self.resolved_config
                .effective_preset
                .explicitly_allowed_secrets
                .retain(|p| p != rel_path);
        } else {
            self.resolved_config
                .effective_preset
                .explicitly_allowed_secrets
                .push(rel_path.to_owned());
        }

        if let Some(ref root) = self.project_root.clone() {
            let token = CancelToken::new();
            let preset = self.resolved_config.effective_preset.clone();
            if let Ok(mut indexed) =
                self.container.pack_service().index_project(root, preset, false, &token)
            {
                if !is_currently_unlocked {
                    let _ = indexed.set_file_selected(rel_path, true);
                }
                self.populate_hydrated_cache_from_project(&indexed, &token);
                self.project = Some(indexed);
                self.rebuild_tree_projection();
                self.repack_from_memory_cache();
            }
        }
    }

    pub fn toggle_rule(&mut self, kind: RuleToggleKind) {
        self.ui_status.copied_recently = false;
        let preset = &mut self.resolved_config.effective_preset;
        let requires_disk_walk = match kind {
            RuleToggleKind::RespectGitignore => {
                preset.respect_gitignore = !preset.respect_gitignore;
                true
            }
            RuleToggleKind::FollowSymlinks => {
                preset.follow_symlinks = !preset.follow_symlinks;
                true
            }
            RuleToggleKind::MinifyCode => {
                preset.minify_code = !preset.minify_code;
                false
            }
            RuleToggleKind::NormalizeLineEndings => {
                preset.normalize_line_endings = !preset.normalize_line_endings;
                false
            }
            RuleToggleKind::IncludeFileTree => {
                preset.template.include_file_tree = !preset.template.include_file_tree;
                false
            }
        };

        if requires_disk_walk {
            let _ = self.rescan_current_project();
        } else {
            self.repack_from_memory_cache();
        }
    }

    pub fn set_max_chunk_bytes(&mut self, max_chunk_bytes: Option<usize>) {
        self.ui_status.copied_recently = false;
        self.resolved_config.effective_preset.template.max_chunk_bytes = max_chunk_bytes;
        self.active_chunk_index = 0;
        self.repack_from_memory_cache();
    }

    pub fn select_preset(&mut self, preset_name: &str) -> Result<(), ConfigPortError> {
        let updated_global = self.container.preset_service().switch_active_preset(preset_name)?;
        self.resolved_config.effective_preset = updated_global.active_preset();
        self.resolved_config.global_config = updated_global;
        let _ = self.rescan_current_project();
        Ok(())
    }

    pub fn cycle_next_preset(&mut self) -> Result<(), ConfigPortError> {
        let presets = &self.resolved_config.global_config.presets;
        if presets.len() <= 1 {
            return self.save_current_preset_globally();
        }
        let current_name = &self.resolved_config.effective_preset.name;
        let curr_idx =
            presets.iter().position(|p| p.name.eq_ignore_ascii_case(current_name)).unwrap_or(0);
        let next_idx = (curr_idx + 1) % presets.len();
        if let Some(next_preset) = presets.get(next_idx) {
            let target_name = next_preset.name.clone();
            return self.select_preset(&target_name);
        }
        Ok(())
    }

    pub fn save_current_preset_globally(&mut self) -> Result<(), ConfigPortError> {
        let preset = self.resolved_config.effective_preset.clone();
        let updated_global = self.container.preset_service().save_preset_globally(preset, true)?;
        self.resolved_config.global_config = updated_global;
        if self.resolved_config.origin
            == core::preset::models::resolution::ConfigOrigin::BuiltInDefault
        {
            self.resolved_config.origin =
                core::preset::models::resolution::ConfigOrigin::GlobalUserConfig;
        }
        Ok(())
    }

    pub fn save_current_preset_to_project(&mut self) -> Result<(), ConfigPortError> {
        let Some(ref root) = self.project_root else {
            return Ok(());
        };
        self.container
            .preset_service()
            .save_preset_to_project(root, &self.resolved_config.effective_preset)?;
        self.resolved_config.origin =
            core::preset::models::resolution::ConfigOrigin::LocalProjectOverride;
        Ok(())
    }

    pub fn select_chunk(&mut self, chunk_index: usize) {
        let Some(ref pack_res) = self.pack_result else {
            return;
        };
        if let Some(chunk) = pack_res.chunks.get(chunk_index) {
            self.active_chunk_index = chunk_index;
            self.ui_status.copied_recently = false;
            self.preview_buffer = VirtualTextBuffer::new(chunk.content.as_str());
        }
    }

    pub fn copy_active_chunk_to_clipboard(&mut self) -> Result<usize, ClipboardPortError> {
        let text_to_copy = self
            .pack_result
            .as_ref()
            .and_then(|res| res.chunks.get(self.active_chunk_index))
            .map_or_else(|| self.preview_buffer.as_str(), |c| c.content.as_str());
        let bytes_len = text_to_copy.len();
        self.container.clipboard().copy_text(text_to_copy)?;
        self.ui_status.copied_recently = true;
        Ok(bytes_len)
    }

    #[inline]
    pub const fn dismiss_warnings(&mut self) {
        self.ui_status.warnings_dismissed = true;
    }

    #[inline]
    pub fn active_banner_items(&self) -> Vec<WarningBannerItem> {
        self.active_banner_items_localized(AppLocale::Ru)
    }

    pub fn active_banner_items_localized(&self, locale: AppLocale) -> Vec<WarningBannerItem> {
        if self.ui_status.warnings_dismissed {
            return Vec::new();
        }
        let mut items: Vec<WarningBannerItem> = self
            .resolved_config
            .warnings
            .iter()
            .map(|w| WarningBannerItem::from_config_warning_localized(w, locale))
            .collect();

        if let Some(ref pack_res) = self.pack_result {
            for anomaly in &pack_res.anomalies {
                items.push(WarningBannerItem::from_anomaly_localized(anomaly, locale));
            }
        } else if let Some(ref proj) = self.project {
            for anomaly in proj.anomalies() {
                items.push(WarningBannerItem::from_anomaly_localized(anomaly, locale));
            }
        }
        items
    }

    pub fn repack_from_memory_cache(&mut self) {
        let Some(ref project) = self.project else {
            return;
        };
        let preset = &self.resolved_config.effective_preset;
        let result = self.container.pack_service().pack_from_hydrated_cache(
            project,
            preset,
            &self.hydrated_cache,
            0,
            &[],
        );
        self.apply_pack_result(result);
    }

    fn populate_hydrated_cache_from_project(&mut self, project: &Project, token: &CancelToken) {
        if let Ok(pairs) = self.container.pack_service().hydrate_selectable_files(project, token) {
            self.hydrated_cache = pairs;
        }
    }

    fn apply_pack_result(&mut self, result: PackResult) {
        let chunk_idx = self.active_chunk_index.min(result.chunks.len().saturating_sub(1));
        self.active_chunk_index = chunk_idx;
        let active_text = result.chunks.get(chunk_idx).map_or("", |c| c.content.as_str());
        self.preview_buffer = VirtualTextBuffer::new(active_text);
        self.pack_result = Some(result);
    }

    fn rebuild_tree_projection(&mut self) {
        if let Some(ref proj) = self.project {
            self.tree_projection = FlatTreeProjection::build_with_mode(
                proj,
                &self.collapsed_dirs,
                &self.search_query,
                self.sort_mode,
            );
        } else {
            self.tree_projection = FlatTreeProjection::default();
        }
        let len = self.tree_projection.visible_nodes().len();
        if len == 0 {
            self.focused_tree_index = None;
        } else if let Some(idx) = self.focused_tree_index
            && idx >= len
        {
            self.focused_tree_index = Some(len - 1);
        }
    }

    #[inline]
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    #[inline]
    pub const fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    #[inline]
    pub const fn resolved_config(&self) -> &ResolvedConfig {
        &self.resolved_config
    }

    #[inline]
    pub const fn effective_preset(&self) -> &Preset {
        &self.resolved_config.effective_preset
    }

    #[inline]
    pub const fn pack_result(&self) -> Option<&PackResult> {
        self.pack_result.as_ref()
    }

    #[inline]
    pub const fn tree_projection(&self) -> &FlatTreeProjection {
        &self.tree_projection
    }

    #[inline]
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    #[inline]
    pub const fn preview_buffer(&self) -> &VirtualTextBuffer {
        &self.preview_buffer
    }

    #[inline]
    pub const fn active_chunk_index(&self) -> usize {
        self.active_chunk_index
    }

    #[inline]
    pub const fn ui_status(&self) -> PackerUiStatus {
        self.ui_status
    }
}
