use std::path::{Path, PathBuf};

use core::applying::aggregates::backup_manifest::BackupManifest;
use di::{AppContainer, BackupSnapshotInfo, ParsedResponseFile};

use crate::locale::UiLabels;
use crate::shared::containers::virtual_text::VirtualTextBuffer;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutofixFlags {
    pub preamble_stripped: bool,
    pub closed_truncated_block: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplierStatusNotice {
    Applied { count: usize },
    Undone { count: usize },
    Error { reason: String },
}

impl ApplierStatusNotice {
    #[inline]
    pub fn format_with_labels(&self, labels: &UiLabels) -> String {
        match self {
            Self::Applied { count } => format!("{} {count}", labels.applier_status_applied_prefix),
            Self::Undone { count } => format!("{} {count}", labels.applier_status_undone_prefix),
            Self::Error { reason } => format!("{} {reason}", labels.applier_status_error_prefix),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangeApplierState {
    container: AppContainer,
    project_root: Option<PathBuf>,
    raw_response: String,
    last_raw_clipboard_text: String,
    last_clipboard_revision: Option<u64>,
    response_buffer: VirtualTextBuffer,
    parsed_files: Vec<ParsedResponseFile>,
    suggested_commands: Vec<String>,
    selected_file_index: usize,
    selected_file_buffer: VirtualTextBuffer,
    editing_diff_line: Option<(usize, String)>,
    autofix_flags: AutofixFlags,
    backups_enabled: bool,
    options_open: bool,
    raw_code_view_mode: bool,
    split_ratio: f32,
    resizing_split: bool,
    last_backup_manifest: Option<BackupManifest>,
    status_notice: Option<ApplierStatusNotice>,
}

impl ChangeApplierState {
    pub fn new(container: AppContainer, project_root: Option<PathBuf>) -> Self {
        let backups_enabled =
            container.preset_service().load_global_config().create_backups_by_default;

        Self {
            container,
            project_root,
            raw_response: String::new(),
            last_raw_clipboard_text: String::new(),
            last_clipboard_revision: None,
            response_buffer: VirtualTextBuffer::default(),
            parsed_files: Vec::new(),
            suggested_commands: Vec::new(),
            selected_file_index: 0,
            selected_file_buffer: VirtualTextBuffer::default(),
            editing_diff_line: None,
            autofix_flags: AutofixFlags::default(),
            backups_enabled,
            options_open: false,
            raw_code_view_mode: false,
            split_ratio: crate::theme::DEFAULT_LAYOUT.applier_split_ratio,
            resizing_split: false,
            last_backup_manifest: None,
            status_notice: None,
        }
    }

    pub fn set_project_root(&mut self, project_root: Option<PathBuf>) {
        self.project_root = project_root;
        if !self.raw_response.is_empty() {
            let existing = std::mem::take(&mut self.raw_response);
            self.apply_text_and_parse(existing);
        }
    }

    #[inline]
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    pub fn set_response_text(&mut self, text: String) {
        self.last_raw_clipboard_text.clone_from(&text);
        self.autofix_flags = AutofixFlags::default();
        self.status_notice = None;
        self.apply_text_and_parse(text);
    }
    pub fn load_response_from_clipboard(&mut self) {
        if let Ok(clip_text) = self.container.clipboard().read_text()
            && !clip_text.is_empty()
        {
            self.last_clipboard_revision = self.container.clipboard().revision();
            self.set_response_text(clip_text);
        }
    }
    pub fn load_response_from_clipboard_if_changed(&mut self) -> bool {
        let current_rev = self.container.clipboard().revision();
        if let (Some(curr), Some(prev)) = (current_rev, self.last_clipboard_revision)
            && curr == prev
            && !self.raw_response.is_empty()
        {
            return false;
        }
        let Ok(clip_text) = self.container.clipboard().read_text() else {
            return false;
        };
        self.last_clipboard_revision = current_rev;
        if clip_text.is_empty()
            || clip_text == self.last_raw_clipboard_text
            || clip_text == self.raw_response
            || clip_text.contains(concat!("=== ", "PROJECT TREE ==="))
        {
            return false;
        }
        self.set_response_text(clip_text);
        true
    }

    pub fn run_autofix(&mut self) {
        let (fixed, preamble_stripped, closed_truncated_block) = self
            .container
            .autofix_response_for_project(&self.raw_response, self.project_root.as_deref());
        self.autofix_flags = AutofixFlags { preamble_stripped, closed_truncated_block };
        self.status_notice = None;
        self.apply_text_and_parse(fixed);
    }

    #[inline]
    pub const fn toggle_backups_enabled(&mut self) {
        self.backups_enabled = !self.backups_enabled;
    }

    #[inline]
    pub const fn set_backups_enabled(&mut self, enabled: bool) {
        self.backups_enabled = enabled;
    }

    #[inline]
    pub const fn toggle_options_open(&mut self) {
        self.options_open = !self.options_open;
    }

    #[inline]
    pub const fn is_options_open(&self) -> bool {
        self.options_open
    }

    #[inline]
    pub const fn is_raw_code_view_mode(&self) -> bool {
        self.raw_code_view_mode
    }

    pub fn toggle_raw_code_view_mode(&mut self) {
        self.raw_code_view_mode = !self.raw_code_view_mode;
        self.editing_diff_line = None;
        self.refresh_selected_file_buffer();
    }

    #[inline]
    pub fn editing_diff_line(&self) -> Option<(usize, &str)> {
        self.editing_diff_line.as_ref().map(|(idx, s)| (*idx, s.as_str()))
    }

    pub fn start_editing_diff_line(&mut self, line_idx: usize) {
        if !self.raw_code_view_mode {
            self.raw_code_view_mode = true;
            self.refresh_selected_file_buffer();
        }
        let raw = self.selected_file_buffer.raw_line(line_idx).to_owned();
        self.editing_diff_line = Some((line_idx, raw));
    }

    pub fn cancel_editing_diff_line(&mut self) {
        self.editing_diff_line = None;
    }

    pub fn append_diff_line_char(&mut self, ch: char) {
        if let Some((_, ref mut draft)) = self.editing_diff_line {
            draft.push(ch);
        }
    }

    pub fn pop_diff_line_char(&mut self) -> bool {
        if let Some((_, ref mut draft)) = self.editing_diff_line {
            return draft.pop().is_some();
        }
        false
    }

    pub fn commit_diff_line_edit(&mut self) {
        let Some((line_idx, new_line)) = self.editing_diff_line.take() else {
            return;
        };
        if let Some(file) = self.parsed_files.get_mut(self.selected_file_index) {
            let mut lines: Vec<&str> = file.content.lines().collect();
            if let Some(slot) = lines.get_mut(line_idx) {
                *slot = &new_line;
                let updated = lines.join(file.line_ending.as_str());
                file.content = updated;
                file.has_conflict = false;
                file.is_selected = file.is_sandbox_safe;
            }
        }
        self.refresh_selected_file_buffer();
    }

    pub fn move_diff_line_edit(&mut self, delta: isize) {
        let Some((curr_idx, _)) = self.editing_diff_line else {
            return;
        };
        self.commit_diff_line_edit();
        let total = self.selected_file_buffer.line_count();
        if total == 0 {
            return;
        }
        let next_idx = if delta < 0 {
            curr_idx.saturating_sub(delta.unsigned_abs())
        } else {
            (curr_idx + delta.unsigned_abs()).min(total - 1)
        };
        self.start_editing_diff_line(next_idx);
    }

    #[inline]
    pub fn suggested_commands(&self) -> &[String] {
        &self.suggested_commands
    }

    pub fn dismiss_suggested_commands(&mut self) {
        self.suggested_commands.clear();
    }

    pub fn backup_snapshots(&self) -> Vec<BackupSnapshotInfo> {
        self.project_root.as_deref().map(AppContainer::list_backups).unwrap_or_default()
    }

    pub fn restore_snapshot_by_id(&mut self, snapshot_id: &str) -> Result<usize, String> {
        let root = self
            .project_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Project root is not set".to_owned())?;
        match AppContainer::restore_backup_snapshot(&root, snapshot_id) {
            Ok(count) => {
                for file in &mut self.parsed_files {
                    if file.is_applied {
                        file.is_applied = false;
                        file.is_selected = file.is_sandbox_safe && !file.has_conflict;
                    }
                }
                self.status_notice = Some(ApplierStatusNotice::Undone { count });
                Ok(count)
            }
            Err(err) => {
                self.status_notice = Some(ApplierStatusNotice::Error { reason: err.clone() });
                Err(err)
            }
        }
    }

    pub fn apply_single_file_to_disk(&mut self, index: usize) -> Result<usize, String> {
        let root = self
            .project_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Project root is not set".to_owned())?;
        let Some(file) = self.parsed_files.get(index).cloned() else {
            return Err("File index out of bounds".to_owned());
        };
        if !file.is_sandbox_safe || file.has_conflict || file.is_applied {
            return Err("File cannot be applied".to_owned());
        }
        let mut single = file;
        single.is_selected = true;
        match AppContainer::apply_response_files(&root, &[single], self.backups_enabled) {
            Ok(count) => {
                if let Some(target) = self.parsed_files.get_mut(index) {
                    target.is_applied = true;
                    target.is_selected = false;
                }
                self.status_notice = Some(ApplierStatusNotice::Applied { count });
                Ok(count)
            }
            Err(err) => {
                self.status_notice = Some(ApplierStatusNotice::Error { reason: err.clone() });
                Err(err)
            }
        }
    }

    pub fn force_resolve_selected_conflict(&mut self) {
        if let Some(file) = self.parsed_files.get_mut(self.selected_file_index)
            && file.has_conflict
            && file.is_sandbox_safe
        {
            file.has_conflict = false;
            file.is_selected = true;
            self.raw_code_view_mode = true;
            self.refresh_selected_file_buffer();
        }
    }

    #[inline]
    pub const fn split_ratio(&self) -> f32 {
        self.split_ratio
    }

    #[inline]
    pub const fn set_split_ratio(&mut self, ratio: f32) {
        let min = crate::theme::DEFAULT_LAYOUT.split_ratio_min;
        let max = crate::theme::DEFAULT_LAYOUT.split_ratio_max;
        self.split_ratio = if ratio < min {
            min
        } else if ratio > max {
            max
        } else {
            ratio
        };
    }

    #[inline]
    pub const fn is_resizing_split(&self) -> bool {
        self.resizing_split
    }

    #[inline]
    pub const fn set_resizing_split(&mut self, resizing: bool) {
        self.resizing_split = resizing;
    }

    pub fn apply_changes_to_disk(&mut self) -> Result<usize, String> {
        let root = self
            .project_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Project root is not set".to_owned())?;

        let applicable_indices: Vec<usize> = self
            .parsed_files
            .iter()
            .enumerate()
            .filter(|(_, f)| f.is_selected && !f.is_applied && f.is_sandbox_safe && !f.has_conflict)
            .map(|(i, _)| i)
            .collect();

        if applicable_indices.is_empty() {
            return Err("Все изменения уже применены или не выбраны".to_owned());
        }

        match AppContainer::apply_response_files(&root, &self.parsed_files, self.backups_enabled) {
            Ok(count) => {
                for idx in applicable_indices {
                    if let Some(file) = self.parsed_files.get_mut(idx) {
                        file.is_applied = true;
                        file.is_selected = false;
                    }
                }
                if let Some(curr) = self.parsed_files.get(self.selected_file_index) {
                    self.selected_file_buffer = VirtualTextBuffer::new(curr.diff_preview.as_str());
                }
                self.status_notice = Some(ApplierStatusNotice::Applied { count });
                Ok(count)
            }
            Err(err) => {
                self.status_notice = Some(ApplierStatusNotice::Error { reason: err.clone() });
                Err(err)
            }
        }
    }

    pub fn undo_last_apply(&mut self) -> Result<usize, String> {
        let root = self
            .project_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Project root is not set".to_owned())?;

        match AppContainer::undo_last_backup(&root) {
            Ok(count) => {
                for file in &mut self.parsed_files {
                    if file.is_applied {
                        file.is_applied = false;
                        file.is_selected = file.is_sandbox_safe && !file.has_conflict;
                    }
                }
                if let Some(curr) = self.parsed_files.get(self.selected_file_index) {
                    self.selected_file_buffer = VirtualTextBuffer::new(curr.diff_preview.as_str());
                }
                self.last_backup_manifest = None;
                self.status_notice = Some(ApplierStatusNotice::Undone { count });
                Ok(count)
            }
            Err(err) => {
                self.status_notice = Some(ApplierStatusNotice::Error { reason: err.clone() });
                Err(err)
            }
        }
    }

    pub fn select_parsed_file(&mut self, index: usize) {
        if index < self.parsed_files.len() {
            self.selected_file_index = index;
            self.editing_diff_line = None;
            self.refresh_selected_file_buffer();
        }
    }

    fn refresh_selected_file_buffer(&mut self) {
        if let Some(file) = self.parsed_files.get(self.selected_file_index) {
            let text = if self.raw_code_view_mode {
                file.content.as_str()
            } else {
                file.diff_preview.as_str()
            };
            self.selected_file_buffer = VirtualTextBuffer::new(text);
        }
    }

    pub fn toggle_file_selection(&mut self, index: usize) {
        if let Some(file) = self.parsed_files.get_mut(index)
            && !file.is_applied
            && file.is_sandbox_safe
            && !file.has_conflict
        {
            file.is_selected = !file.is_selected;
        }
    }

    fn apply_text_and_parse(&mut self, text: String) {
        let mut effective_text = text;
        if !effective_text.trim().is_empty() {
            let (fixed, preamble_stripped, closed_truncated_block) = self
                .container
                .autofix_response_for_project(&effective_text, self.project_root.as_deref());
            if preamble_stripped || closed_truncated_block {
                self.autofix_flags = AutofixFlags { preamble_stripped, closed_truncated_block };
                effective_text = fixed;
            }
        }

        let parsed = self
            .container
            .parse_and_validate_for_project(&effective_text, self.project_root.as_deref());
        self.suggested_commands = AppContainer::extract_suggested_commands(&effective_text);

        self.response_buffer = VirtualTextBuffer::new(effective_text.as_str());
        self.parsed_files = parsed;
        self.raw_response = effective_text;
        self.selected_file_index = 0;
        self.raw_code_view_mode = false;
        self.refresh_selected_file_buffer();
        if self.parsed_files.is_empty() {
            self.selected_file_buffer = VirtualTextBuffer::default();
        }
    }

    #[inline]
    pub const fn response_buffer(&self) -> &VirtualTextBuffer {
        &self.response_buffer
    }

    #[inline]
    pub fn parsed_files(&self) -> &[ParsedResponseFile] {
        &self.parsed_files
    }

    #[inline]
    pub const fn selected_file_index(&self) -> usize {
        self.selected_file_index
    }

    #[inline]
    pub const fn selected_file_buffer(&self) -> &VirtualTextBuffer {
        &self.selected_file_buffer
    }

    #[inline]
    pub const fn autofix_flags(&self) -> AutofixFlags {
        self.autofix_flags
    }

    #[inline]
    pub const fn backups_enabled(&self) -> bool {
        self.backups_enabled
    }

    #[inline]
    pub const fn last_backup_manifest(&self) -> Option<&BackupManifest> {
        self.last_backup_manifest.as_ref()
    }

    #[inline]
    pub const fn status_notice(&self) -> Option<&ApplierStatusNotice> {
        self.status_notice.as_ref()
    }

    #[inline]
    pub fn clear_status_notice(&mut self) {
        self.status_notice = None;
    }

    #[inline]
    pub fn formatted_status_message(&self, labels: &UiLabels) -> Option<String> {
        self.status_notice.as_ref().map(|n| n.format_with_labels(labels))
    }
}
