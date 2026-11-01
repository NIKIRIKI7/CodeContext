use std::path::{Path, PathBuf};

use core::applying::aggregates::backup_manifest::BackupManifest;
use di::{AppContainer, ParsedResponseFile};

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
    last_clipboard_revision: Option<u64>,
    response_buffer: VirtualTextBuffer,
    parsed_files: Vec<ParsedResponseFile>,
    selected_file_index: usize,
    selected_file_buffer: VirtualTextBuffer,
    autofix_flags: AutofixFlags,
    backups_enabled: bool,
    options_open: bool,
    last_backup_manifest: Option<BackupManifest>,
    status_notice: Option<ApplierStatusNotice>,
}

impl ChangeApplierState {
    pub fn new(container: AppContainer, project_root: Option<PathBuf>) -> Self {
        let backups_enabled =
            container.preset_service().load_global_config().create_backups_by_default;
        let resolved_root = project_root.or_else(|| std::env::current_dir().ok());

        Self {
            container,
            project_root: resolved_root,
            raw_response: String::new(),
            last_clipboard_revision: None,
            response_buffer: VirtualTextBuffer::default(),
            parsed_files: Vec::new(),
            selected_file_index: 0,
            selected_file_buffer: VirtualTextBuffer::default(),
            autofix_flags: AutofixFlags::default(),
            backups_enabled,
            options_open: false,
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
        self.autofix_flags = AutofixFlags::default();
        self.status_notice = None;
        self.apply_text_and_parse(text);
    }

    pub fn load_response_from_clipboard(&mut self) {
        let _ = self.load_response_from_clipboard_if_changed();
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

        if clip_text.is_empty() || clip_text == self.raw_response {
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

    pub fn apply_changes_to_disk(&mut self) -> Result<usize, String> {
        let root = self
            .project_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Project root is not set".to_owned())?;

        match self.container.execute_plan_for_project(
            &root,
            &self.raw_response,
            self.backups_enabled,
        ) {
            Ok(manifest) => {
                let count = manifest.applied_files_count;
                self.last_backup_manifest = Some(manifest);
                self.status_notice = Some(ApplierStatusNotice::Applied { count });
                Ok(count)
            }
            Err(err) => {
                let reason = err.to_string();
                self.status_notice = Some(ApplierStatusNotice::Error { reason: reason.clone() });
                Err(reason)
            }
        }
    }

    pub fn undo_last_apply(&mut self) -> Result<usize, String> {
        let root = self
            .project_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Project root is not set".to_owned())?;

        match self.container.undo_last_for_project(&root) {
            Ok(count) => {
                self.last_backup_manifest = None;
                self.status_notice = Some(ApplierStatusNotice::Undone { count });
                Ok(count)
            }
            Err(err) => {
                let reason = err.to_string();
                self.status_notice = Some(ApplierStatusNotice::Error { reason: reason.clone() });
                Err(reason)
            }
        }
    }

    pub fn select_parsed_file(&mut self, index: usize) {
        if let Some(file) = self.parsed_files.get(index) {
            self.selected_file_index = index;
            self.selected_file_buffer = VirtualTextBuffer::new(file.diff_preview.as_str());
        }
    }

    fn apply_text_and_parse(&mut self, text: String) {
        if self.project_root.is_none() {
            self.project_root = std::env::current_dir().ok();
        }
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

        self.response_buffer = VirtualTextBuffer::new(effective_text.as_str());
        self.parsed_files = parsed;
        self.raw_response = effective_text;
        self.selected_file_index = 0;
        self.selected_file_buffer =
            self.parsed_files.first().map_or_else(VirtualTextBuffer::default, |f| {
                VirtualTextBuffer::new(f.diff_preview.as_str())
            });
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
