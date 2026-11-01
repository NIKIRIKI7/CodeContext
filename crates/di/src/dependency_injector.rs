use std::path::{Path, PathBuf};
use std::sync::Arc;

use apply::{AutofixResult, ParsedFile, ResponseApplier};
use config::LayeredConfigLoader;
use core::applying::aggregates::backup_manifest::BackupManifest;
use core::applying::ports::applier::ApplierPortError;
use core::applying::services::apply_service::ApplyService;
use core::applying::values::patch_status::{PatchOperationKind, PatchResolutionStatus};
use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};
use core::packing::services::pack_service::PackService;
use core::preset::models::preset::Preset;
use core::preset::services::preset_service::PresetService;
use core::shared_kernel::fs::line_ending::LineEnding;
use gpui::Global;
use platform::{
    InMemoryClipboard, SystemClipboard, SystemGlobalHotkey, SystemShellIntegration,
    SystemSingleInstanceIpc,
};
use scanner::IgnoreFileScanner;

#[derive(Debug, Clone)]
pub enum ClipboardAdapter {
    System(SystemClipboard),
    InMemory(InMemoryClipboard),
}

impl Default for ClipboardAdapter {
    fn default() -> Self {
        Self::System(SystemClipboard::new())
    }
}

impl ClipboardAdapter {
    pub fn read_text(&self) -> Result<String, ClipboardPortError> {
        match self {
            Self::System(sys) => sys.read_text(),
            Self::InMemory(mem) => Ok(mem.last_text()),
        }
    }

    #[inline]
    pub fn revision(&self) -> Option<u64> {
        match self {
            Self::System(_) => None,
            Self::InMemory(mem) => Some(mem.revision()),
        }
    }
}

impl ClipboardPort for ClipboardAdapter {
    #[inline]
    fn copy_text(&self, text: &str) -> Result<(), ClipboardPortError> {
        match self {
            Self::System(sys) => sys.copy_text(text),
            Self::InMemory(mem) => mem.copy_text(text),
        }
    }

    #[inline]
    fn read_text(&self) -> Result<String, ClipboardPortError> {
        Self::read_text(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResponseFile {
    pub rel_path: String,
    pub content: String,
    pub diff_preview: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub line_ending: LineEnding,
    pub operation_kind: PatchOperationKind,
    pub is_truncated: bool,
    pub is_sandbox_safe: bool,
    pub has_conflict: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AppContainerBuilder {
    global_config_path: Option<PathBuf>,
    clipboard: Option<ClipboardAdapter>,
    shell_integration: Option<SystemShellIntegration>,
    hotkey: Option<SystemGlobalHotkey>,
    ipc: Option<SystemSingleInstanceIpc>,
}

impl AppContainerBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[inline]
    pub fn with_global_config_path(mut self, path: PathBuf) -> Self {
        self.global_config_path = Some(path);
        self
    }

    #[must_use]
    #[inline]
    pub fn with_clipboard(mut self, clipboard: ClipboardAdapter) -> Self {
        self.clipboard = Some(clipboard);
        self
    }

    #[must_use]
    #[inline]
    pub fn with_shell_integration(mut self, shell: SystemShellIntegration) -> Self {
        self.shell_integration = Some(shell);
        self
    }

    #[must_use]
    #[inline]
    pub fn with_hotkey(mut self, hotkey: SystemGlobalHotkey) -> Self {
        self.hotkey = Some(hotkey);
        self
    }

    #[must_use]
    #[inline]
    pub fn with_ipc(mut self, ipc: SystemSingleInstanceIpc) -> Self {
        self.ipc = Some(ipc);
        self
    }

    pub fn build(self) -> AppContainer {
        let clipboard = self.clipboard.unwrap_or_default();
        let config_loader = self
            .global_config_path
            .map_or_else(LayeredConfigLoader::new, LayeredConfigLoader::with_global_config_path);
        let shell_integration = self.shell_integration.unwrap_or_default();
        let hotkey = self.hotkey.unwrap_or_default();
        let ipc = self.ipc.unwrap_or_default();
        let scanner = IgnoreFileScanner::new();
        let applier = ResponseApplier::new();

        AppContainer {
            pack_service: Arc::new(PackService::new(scanner, clipboard.clone())),
            preset_service: Arc::new(PresetService::new(config_loader)),
            apply_service: Arc::new(ApplyService::new(applier)),
            clipboard,
            shell_integration,
            hotkey,
            ipc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppContainer {
    pack_service: Arc<PackService<IgnoreFileScanner, ClipboardAdapter>>,
    preset_service: Arc<PresetService<LayeredConfigLoader>>,
    apply_service: Arc<ApplyService<ResponseApplier>>,
    clipboard: ClipboardAdapter,
    shell_integration: SystemShellIntegration,
    hotkey: SystemGlobalHotkey,
    ipc: SystemSingleInstanceIpc,
}

impl Global for AppContainer {}

impl Default for AppContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContainer {
    #[inline]
    pub fn new() -> Self {
        AppContainerBuilder::new().build()
    }

    #[inline]
    pub fn builder() -> AppContainerBuilder {
        AppContainerBuilder::new()
    }

    #[inline]
    pub fn for_testing(global_config_path: PathBuf) -> Self {
        AppContainerBuilder::new()
            .with_global_config_path(global_config_path)
            .with_clipboard(ClipboardAdapter::InMemory(InMemoryClipboard::new()))
            .with_shell_integration(SystemShellIntegration::new_in_memory())
            .with_hotkey(SystemGlobalHotkey::new_in_memory())
            .with_ipc(SystemSingleInstanceIpc::new_in_memory())
            .build()
    }

    #[inline]
    pub fn pack_service(&self) -> &PackService<IgnoreFileScanner, ClipboardAdapter> {
        &self.pack_service
    }

    #[inline]
    pub fn pack_service_arc(&self) -> Arc<PackService<IgnoreFileScanner, ClipboardAdapter>> {
        Arc::clone(&self.pack_service)
    }

    #[inline]
    pub fn preset_service(&self) -> &PresetService<LayeredConfigLoader> {
        &self.preset_service
    }

    #[inline]
    pub fn apply_service(&self) -> &ApplyService<ResponseApplier> {
        &self.apply_service
    }

    #[inline]
    pub const fn clipboard(&self) -> &ClipboardAdapter {
        &self.clipboard
    }

    #[inline]
    pub const fn shell_integration(&self) -> &SystemShellIntegration {
        &self.shell_integration
    }

    #[inline]
    pub const fn hotkey(&self) -> &SystemGlobalHotkey {
        &self.hotkey
    }

    #[inline]
    pub const fn ipc(&self) -> &SystemSingleInstanceIpc {
        &self.ipc
    }

    #[inline]
    pub fn autofix_response_for_project(
        &self,
        raw_response: &str,
        project_root: Option<&Path>,
    ) -> (String, bool, bool) {
        let resolved = self.preset_service.resolve_project_config(project_root);
        self.apply_service.autofix_response(raw_response, &resolved.effective_preset)
    }

    pub fn parse_and_validate_for_project(
        &self,
        raw_response: &str,
        project_root: Option<&Path>,
    ) -> Vec<ParsedResponseFile> {
        let resolved = self.preset_service.resolve_project_config(project_root);
        Self::plan_to_parsed_files(
            &self.apply_service,
            raw_response,
            project_root,
            &resolved.effective_preset,
        )
    }

    pub fn execute_plan_for_project(
        &self,
        project_root: &Path,
        raw_response: &str,
        create_backup: bool,
    ) -> Result<BackupManifest, ApplierPortError> {
        let resolved = self.preset_service.resolve_project_config(Some(project_root));
        let plan = self.apply_service.build_plan(
            Some(project_root),
            raw_response,
            &resolved.effective_preset,
        );
        self.apply_service.execute_plan(project_root, &plan, create_backup)
    }

    #[inline]
    pub fn undo_last_for_project(&self, project_root: &Path) -> Result<usize, ApplierPortError> {
        self.apply_service.undo_last(project_root)
    }

    #[inline]
    pub fn autofix_response(raw_response: &str) -> (String, bool, bool) {
        let AutofixResult { fixed_text, preamble_stripped, closed_truncated_block } =
            ResponseApplier::autofix(raw_response);
        (fixed_text, preamble_stripped, closed_truncated_block)
    }

    pub fn parse_and_validate_response(
        raw_response: &str,
        project_root: Option<&Path>,
    ) -> Vec<ParsedResponseFile> {
        let applier = ApplyService::new(ResponseApplier::new());
        Self::plan_to_parsed_files(&applier, raw_response, project_root, &Preset::default())
    }

    fn plan_to_parsed_files(
        service: &ApplyService<ResponseApplier>,
        raw_response: &str,
        project_root: Option<&Path>,
        preset: &Preset,
    ) -> Vec<ParsedResponseFile> {
        let plan = service.build_plan(project_root, raw_response, preset);
        plan.items
            .into_iter()
            .map(|item| {
                let has_conflict =
                    matches!(item.status, PatchResolutionStatus::SearchBlockConflict { .. });
                ParsedResponseFile {
                    rel_path: item.rel_path,
                    content: item.resolved_content,
                    diff_preview: item.diff_preview,
                    added_lines: item.added_lines,
                    removed_lines: item.removed_lines,
                    line_ending: item.line_ending,
                    operation_kind: item.operation.kind(),
                    is_truncated: item.is_truncated,
                    is_sandbox_safe: item.is_sandbox_safe,
                    has_conflict,
                }
            })
            .collect()
    }

    pub fn apply_response_files(
        project_root: &Path,
        files: &[ParsedResponseFile],
        create_backup: bool,
    ) -> Result<usize, String> {
        let files_to_write: Vec<ParsedFile> = files
            .iter()
            .filter(|f| !f.has_conflict && f.operation_kind != PatchOperationKind::DeleteFile)
            .map(|f| ParsedFile {
                rel_path: f.rel_path.clone(),
                content: f.content.clone(),
                is_truncated: f.is_truncated,
            })
            .collect();

        let files_to_delete: Vec<String> = files
            .iter()
            .filter(|f| !f.has_conflict && f.operation_kind == PatchOperationKind::DeleteFile)
            .map(|f| f.rel_path.clone())
            .collect();

        let report = ResponseApplier::apply_files_and_deletions(
            project_root,
            &files_to_write,
            &files_to_delete,
            create_backup,
        )
        .map_err(|e| e.to_string())?;

        Ok(report.applied_files_count)
    }

    #[inline]
    pub fn undo_last_backup(project_root: &Path) -> Result<usize, String> {
        let service = ApplyService::new(ResponseApplier::new());
        service.undo_last(project_root).map_err(|e| e.to_string())
    }
}
