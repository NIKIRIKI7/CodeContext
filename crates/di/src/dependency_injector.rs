use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use apply::{
    AutofixResult, BackupSnapshotInfo, ParsedFile, ResponseApplier, compute_dir_size_bytes,
    list_backup_snapshots, purge_all_backups, restore_backup_by_id,
};
use config::LayeredConfigLoader;
use core::applying::aggregates::backup_manifest::BackupManifest;
use core::applying::ports::applier::ApplierPortError;
use core::applying::services::apply_service::ApplyService;
use core::applying::values::patch_status::{PatchOperationKind, PatchResolutionStatus};
use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};
use core::packing::services::pack_service::PackService;
use core::preset::models::preset::Preset;
use core::preset::models::resolution::{ConfigWarning, EditionProfile, GlobalConfig};
use core::preset::services::preset_service::PresetService;
use core::shared_kernel::execution::telemetry::TelemetryLog;
use core::shared_kernel::fs::line_ending::LineEnding;
use core::system::plugins::protocol::{PluginDescriptor, PluginHostPort as _};
use gpui::Global;
use platform::{
    InMemoryClipboard, SystemClipboard, SystemGlobalHotkey, SystemPluginHost,
    SystemShellIntegration, SystemSingleInstanceIpc, SystemUpdater,
};
use scanner::{IgnoreFileScanner, is_valid_regex_pattern};

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HousekeepingStats {
    pub backups_bytes: u64,
    pub backups_count: usize,
    pub temp_files_bytes: u64,
    pub temp_files_count: usize,
    pub ipc_socket_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResponseFile {
    pub rel_path: String,
    pub renamed_from: Option<String>,
    pub content: String,
    pub diff_preview: String,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub line_ending: LineEnding,
    pub operation_kind: PatchOperationKind,
    pub is_truncated: bool,
    pub is_sandbox_safe: bool,
    pub has_conflict: bool,
    pub is_selected: bool,
    pub is_applied: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AppContainerBuilder {
    global_config_path: Option<PathBuf>,
    clipboard: Option<ClipboardAdapter>,
    shell_integration: Option<SystemShellIntegration>,
    hotkey: Option<SystemGlobalHotkey>,
    ipc: Option<SystemSingleInstanceIpc>,
    updater: Option<SystemUpdater>,
    plugin_host: Option<SystemPluginHost>,
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

    #[must_use]
    #[inline]
    pub fn with_updater(mut self, updater: SystemUpdater) -> Self {
        self.updater = Some(updater);
        self
    }

    #[must_use]
    #[inline]
    pub fn with_plugin_host(mut self, plugin_host: SystemPluginHost) -> Self {
        self.plugin_host = Some(plugin_host);
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
        let updater = self.updater.unwrap_or_default();
        let plugin_host = self.plugin_host.unwrap_or_default();
        let scanner = IgnoreFileScanner::new();
        let applier = ResponseApplier::new();
        let telemetry = Arc::new(TelemetryLog::new());
        telemetry.record_info("system", "init_container", "DI container initialized");

        AppContainer {
            pack_service: Arc::new(PackService::new(scanner, clipboard.clone())),
            preset_service: Arc::new(PresetService::new(config_loader)),
            apply_service: Arc::new(ApplyService::new(applier)),
            telemetry,
            clipboard,
            shell_integration,
            hotkey,
            ipc,
            updater,
            plugin_host,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppContainer {
    pack_service: Arc<PackService<IgnoreFileScanner, ClipboardAdapter>>,
    preset_service: Arc<PresetService<LayeredConfigLoader>>,
    apply_service: Arc<ApplyService<ResponseApplier>>,
    telemetry: Arc<TelemetryLog>,
    clipboard: ClipboardAdapter,
    shell_integration: SystemShellIntegration,
    hotkey: SystemGlobalHotkey,
    ipc: SystemSingleInstanceIpc,
    updater: SystemUpdater,
    plugin_host: SystemPluginHost,
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
            .with_updater(SystemUpdater::new_in_memory())
            .with_plugin_host(SystemPluginHost::new_in_memory())
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
    pub const fn updater(&self) -> &SystemUpdater {
        &self.updater
    }

    #[inline]
    pub const fn plugin_host(&self) -> &SystemPluginHost {
        &self.plugin_host
    }

    #[inline]
    pub fn list_plugins(&self) -> Vec<PluginDescriptor> {
        self.plugin_host.list_plugins()
    }

    pub fn export_global_settings_json(&self) -> String {
        let cfg = self.preset_service.load_global_config();
        LayeredConfigLoader::export_global_json_string(&cfg).unwrap_or_default()
    }

    pub fn apply_global_settings_json(
        &self,
        raw_json: &str,
    ) -> Result<(GlobalConfig, Vec<ConfigWarning>), String> {
        self.preset_service
            .repo()
            .parse_and_save_global_json_str(raw_json)
            .map_err(|e| e.to_string())
    }

    pub fn create_project_file(project_root: &Path, rel_path: &str) -> Result<(), String> {
        let abs_target =
            ResponseApplier::validate_path(project_root, rel_path).map_err(|e| e.to_string())?;
        if abs_target.exists() {
            return Ok(());
        }
        ResponseApplier::apply_files(
            project_root,
            &[ParsedFile {
                rel_path: rel_path.to_owned(),
                content: String::new(),
                is_truncated: false,
            }],
            false,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
    }

    pub fn create_project_folder(project_root: &Path, rel_path: &str) -> Result<(), String> {
        let abs_target =
            ResponseApplier::validate_path(project_root, rel_path).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(abs_target).map_err(|e| e.to_string())
    }

    pub fn delete_project_entry_with_backup(
        project_root: &Path,
        rel_path: &str,
    ) -> Result<(), String> {
        let abs_target =
            ResponseApplier::validate_path(project_root, rel_path).map_err(|e| e.to_string())?;
        if abs_target.is_dir() {
            let mut files_in_dir = Vec::new();
            collect_rel_files_in_dir(project_root, &abs_target, &mut files_in_dir);
            if !files_in_dir.is_empty() {
                let _ = ResponseApplier::apply_files_and_deletions(
                    project_root,
                    &[],
                    &files_in_dir,
                    true,
                );
            }
            std::fs::remove_dir_all(&abs_target).map_err(|e| e.to_string())
        } else if abs_target.is_file() {
            ResponseApplier::apply_files_and_deletions(
                project_root,
                &[],
                &[rel_path.to_owned()],
                true,
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        } else {
            Ok(())
        }
    }

    pub fn perform_modular_setup(
        &self,
        edition: EditionProfile,
        custom_dir: Option<PathBuf>,
        register_shell: bool,
    ) -> Result<PathBuf, String> {
        use core::system::ports::shell::{ShellIntegrationPort as _, ShellMenuConfig};
        use core::system::ports::updater::{InstallRequestOptions, UpdaterPort as _};

        let _ = self.preset_service.apply_edition_profile(edition);
        let probe = self.updater.probe_system();
        let target_dir = custom_dir.unwrap_or(probe.default_install_dir);
        let opts = InstallRequestOptions {
            target_dir,
            edition: edition.as_str().to_owned(),
            register_context_menu: register_shell,
            add_to_user_path: true,
            enable_change_applier: edition != EditionProfile::Lite,
            enable_terminal_runner: edition != EditionProfile::Lite,
        };
        let receipt = self.updater.execute_modular_install(&opts).map_err(|e| e.to_string())?;
        if register_shell {
            let cfg = ShellMenuConfig::with_executable(receipt.binary_path.clone());
            let _ = self.shell_integration.register_with_config(&cfg);
        }
        Ok(receipt.binary_path)
    }

    #[inline]
    pub const fn telemetry(&self) -> &Arc<TelemetryLog> {
        &self.telemetry
    }

    #[inline]
    pub fn autofix_response_for_project(
        &self,
        raw_response: &str,
        project_root: Option<&Path>,
    ) -> (String, bool, bool) {
        let t0 = Instant::now();
        let resolved = self.preset_service.resolve_project_config(project_root);
        let res = self.apply_service.autofix_response(raw_response, &resolved.effective_preset);
        self.telemetry.record_timed(
            "apply",
            "autofix_stream",
            t0.elapsed(),
            format!(
                "{} B -> {} B · preamble={} · closed={}",
                raw_response.len(),
                res.0.len(),
                res.1,
                res.2
            ),
        );
        res
    }

    pub fn parse_and_validate_for_project(
        &self,
        raw_response: &str,
        project_root: Option<&Path>,
    ) -> Vec<ParsedResponseFile> {
        let t0 = Instant::now();
        let resolved = self.preset_service.resolve_project_config(project_root);
        let files = Self::plan_to_parsed_files(
            &self.apply_service,
            raw_response,
            project_root,
            &resolved.effective_preset,
        );
        let conflicts = files.iter().filter(|f| f.has_conflict || !f.is_sandbox_safe).count();
        if conflicts > 0 {
            self.telemetry.record_warn(
                "apply",
                "parse_and_diff",
                Some(t0.elapsed()),
                format!("{} files parsed · {} conflicts/blocked", files.len(), conflicts),
            );
        } else {
            self.telemetry.record_timed(
                "apply",
                "parse_and_diff",
                t0.elapsed(),
                format!("{} files parsed · 0 conflicts", files.len()),
            );
        }
        files
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
    pub fn is_valid_regex(pattern: &str, case_sensitive: bool) -> bool {
        is_valid_regex_pattern(pattern, case_sensitive)
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
                let exists_on_disk =
                    project_root.is_some_and(|root| root.join(&item.rel_path).exists());
                let has_effective_changes = !exists_on_disk
                    || item.operation.kind() == PatchOperationKind::DeleteFile
                    || item.added_lines > 0
                    || item.removed_lines > 0;
                let is_selected = item.is_sandbox_safe && !has_conflict && has_effective_changes;
                let renamed_from = match &item.operation {
                    core::applying::aggregates::apply_plan::FileOperation::RenameFile {
                        from_rel_path,
                    } => Some(from_rel_path.clone()),
                    _ => None,
                };
                ParsedResponseFile {
                    rel_path: item.rel_path,
                    renamed_from,
                    content: item.resolved_content,
                    diff_preview: item.diff_preview,
                    added_lines: item.added_lines,
                    removed_lines: item.removed_lines,
                    line_ending: item.line_ending,
                    operation_kind: item.operation.kind(),
                    is_truncated: item.is_truncated,
                    is_sandbox_safe: item.is_sandbox_safe,
                    has_conflict,
                    is_selected,
                    is_applied: false,
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
            .filter(|f| {
                f.is_selected
                    && !f.is_applied
                    && f.is_sandbox_safe
                    && !f.has_conflict
                    && f.operation_kind != PatchOperationKind::DeleteFile
            })
            .map(|f| ParsedFile {
                rel_path: f.rel_path.clone(),
                content: f.content.clone(),
                is_truncated: f.is_truncated,
            })
            .collect();

        let mut files_to_delete: Vec<String> = files
            .iter()
            .filter(|f| {
                f.is_selected
                    && !f.is_applied
                    && f.is_sandbox_safe
                    && !f.has_conflict
                    && f.operation_kind == PatchOperationKind::DeleteFile
            })
            .map(|f| f.rel_path.clone())
            .collect();
        for f in files {
            if f.is_selected
                && !f.is_applied
                && f.is_sandbox_safe
                && !f.has_conflict
                && let Some(ref old_p) = f.renamed_from
                && !files_to_delete.contains(old_p)
            {
                files_to_delete.push(old_p.clone());
            }
        }

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

    #[inline]
    pub fn list_backups(project_root: &Path) -> Vec<BackupSnapshotInfo> {
        list_backup_snapshots(project_root)
    }

    #[inline]
    pub fn restore_backup_snapshot(
        project_root: &Path,
        snapshot_id: &str,
    ) -> Result<usize, String> {
        restore_backup_by_id(project_root, snapshot_id)
    }

    #[inline]
    pub fn extract_suggested_commands(raw_response: &str) -> Vec<String> {
        ResponseApplier::extract_run_commands(raw_response)
    }

    pub fn rename_project_entry(
        project_root: &Path,
        old_rel_path: &str,
        new_rel_path: &str,
    ) -> Result<(), String> {
        let abs_old = ResponseApplier::validate_path(project_root, old_rel_path)
            .map_err(|e| e.to_string())?;
        let abs_new = ResponseApplier::validate_path(project_root, new_rel_path)
            .map_err(|e| e.to_string())?;
        if let Some(parent) = abs_new.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::rename(abs_old, abs_new).map_err(|e| e.to_string())
    }

    pub fn inspect_housekeeping(&self, project_root: Option<&Path>) -> HousekeepingStats {
        let mut stats = HousekeepingStats::default();
        if let Some(root) = project_root {
            let snaps = list_backup_snapshots(root);
            stats.backups_count = snaps.len();
            stats.backups_bytes = compute_dir_size_bytes(&root.join(".ctxpack/backups"));
            let (tmp_count, tmp_bytes) = scan_temp_files_in_dir(root, false);
            stats.temp_files_count = tmp_count;
            stats.temp_files_bytes = tmp_bytes;
        }
        stats.ipc_socket_exists = self.ipc.endpoint_config().socket_file_path.exists();
        stats
    }

    pub fn clean_housekeeping(
        &self,
        project_root: Option<&Path>,
        clean_backups: bool,
        clean_temp: bool,
        clean_ipc: bool,
    ) -> u64 {
        let mut freed = 0_u64;
        if let Some(root) = project_root {
            if clean_backups {
                freed = freed.saturating_add(purge_all_backups(root));
            }
            if clean_temp {
                let (_, bytes) = scan_temp_files_in_dir(root, true);
                freed = freed.saturating_add(bytes);
            }
        }
        if clean_ipc {
            let sock = &self.ipc.endpoint_config().socket_file_path;
            if sock.exists() {
                if let Ok(meta) = std::fs::metadata(sock) {
                    freed = freed.saturating_add(meta.len());
                }
                let _ = std::fs::remove_file(sock);
            }
        }
        freed
    }
}

fn collect_rel_files_in_dir(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_rel_files_in_dir(root, &p, out);
        } else if p.is_file()
            && let Ok(rel) = p.strip_prefix(root)
        {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn scan_temp_files_in_dir(dir: &Path, remove: bool) -> (usize, u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (0, 0);
    };
    let mut count = 0_usize;
    let mut bytes = 0_u64;
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if p.is_dir() {
            if name_str != ".git" && name_str != "target" && name_str != "node_modules" {
                let (c, b) = scan_temp_files_in_dir(&p, remove);
                count += c;
                bytes = bytes.saturating_add(b);
            }
        } else if name_str.starts_with('.') && name_str.ends_with(".tmp") {
            if let Ok(meta) = entry.metadata() {
                bytes = bytes.saturating_add(meta.len());
            }
            count += 1;
            if remove {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    (count, bytes)
}
