use std::collections::HashSet;
use std::path::{Path, PathBuf};

use core::packing::aggregates::pack_result::PackResult;
use core::packing::aggregates::project::Project;
use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};
use core::packing::services::pack_service::PackServiceError;
use core::preset::models::preset::Preset;
use core::preset::models::resolution::ResolvedConfig;
use core::preset::ports::config_repo::ConfigPortError;
use core::shared_kernel::execution::cancel_token::CancelToken;
use di::AppContainer;

use crate::features::context_packer::components::file_tree::{FlatTreeProjection, TriState};
use crate::features::context_packer::components::rules_editor::RuleToggleKind;
use crate::locale::AppLocale;
use crate::shared::containers::virtual_text::VirtualTextBuffer;
use crate::shared::primitives::warning_banner::WarningBannerItem;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PackerUiStatus {
    pub copied_recently: bool,
    pub warnings_dismissed: bool,
    pub search_focused: bool,
    pub filters_popover_open: bool,
    pub meta_modal_requested: bool,
}

#[derive(Debug, Clone)]
pub struct ContextPackerState {
    container: AppContainer,
    project_root: Option<PathBuf>,
    resolved_config: ResolvedConfig,
    project: Option<Project>,
    hydrated_cache: Vec<(String, String)>,
    pack_result: Option<PackResult>,
    tree_projection: FlatTreeProjection,
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
            hydrated_cache: Vec::new(),
            pack_result: None,
            tree_projection: FlatTreeProjection::default(),
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
        self.search_query.clear();
        self.collapsed_dirs.clear();
        self.resolved_config =
            self.container.preset_service().resolve_project_config(Some(&project_root));
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

        for rel_path in &previous_deselected {
            let _ = indexed_project.set_file_selected(rel_path, false);
        }

        let packed = self.container.pack_service().pack_project(&indexed_project, &token)?;
        self.populate_hydrated_cache_from_project(&indexed_project, &token);
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
        self.hydrated_cache.sort_unstable_by(|a, b| a.0.cmp(&b.0));
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
            self.tree_projection =
                FlatTreeProjection::build(proj, &self.collapsed_dirs, &self.search_query);
        } else {
            self.tree_projection = FlatTreeProjection::default();
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
