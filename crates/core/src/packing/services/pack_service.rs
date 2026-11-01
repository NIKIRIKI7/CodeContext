use std::path::Path;

use thiserror::Error;

use crate::packing::aggregates::file_entry::FileEntry;
use crate::packing::aggregates::pack_result::PackResult;
use crate::packing::aggregates::project::Project;
use crate::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};
use crate::packing::ports::scanner::{FileScannerPort, HydrateReport, ScannerPortError};
use crate::packing::services::chunker::Chunker;
use crate::packing::services::secret_guard::SecretGuard;
use crate::preset::models::preset::Preset;
use crate::shared_kernel::execution::anomaly::Anomaly;
use crate::shared_kernel::execution::cancel_token::CancelToken;
use crate::shared_kernel::fs::file_kind::FileKind;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PackServiceError {
    #[error(transparent)]
    Scanner(#[from] ScannerPortError),
    #[error(transparent)]
    Clipboard(#[from] ClipboardPortError),
}

#[derive(Debug, Clone)]
pub struct PackService<S, C> {
    scanner: S,
    clipboard: C,
}

impl<S, C> PackService<S, C>
where
    S: FileScannerPort,
    C: ClipboardPort,
{
    #[inline]
    pub const fn new(scanner: S, clipboard: C) -> Self {
        Self { scanner, clipboard }
    }

    pub fn index_project(
        &self,
        project_root: &Path,
        preset: Preset,
        from_untrusted_project_config: bool,
        cancel_token: &CancelToken,
    ) -> Result<Project, PackServiceError> {
        let effective_preset = if from_untrusted_project_config {
            preset.sanitized_from_untrusted_project_config()
        } else {
            preset
        };

        let walk_report = self.scanner.scan_index(project_root, &effective_preset, cancel_token)?;
        let mut entries = Vec::with_capacity(walk_report.entries.len());
        let mut anomalies = walk_report.anomalies;

        for raw in walk_report.entries {
            let (kind, secret_reason, secret_anomaly) =
                SecretGuard::classify_entry(&raw.rel_path, &effective_preset);

            if let Some(anomaly) = secret_anomaly {
                anomalies.push(anomaly);
            }

            if raw.size_bytes > effective_preset.large_file_threshold_bytes
                && kind.is_packable_by_default()
            {
                anomalies.push(Anomaly::LargeFileWarning {
                    rel_path: raw.rel_path.clone(),
                    size_bytes: raw.size_bytes,
                    threshold_bytes: effective_preset.large_file_threshold_bytes,
                });
            }

            let entry = if let Some(reason) = secret_reason {
                let mut e = FileEntry::new_blocked_secret(
                    raw.rel_path,
                    raw.size_bytes,
                    reason.as_str().to_owned(),
                );
                if effective_preset.is_secret_explicitly_allowed(&e.rel_path) {
                    e.set_selected(true, true);
                }
                e
            } else {
                FileEntry::new(raw.rel_path, raw.size_bytes, kind)
            };

            entries.push(entry);
        }

        Ok(Project::new(project_root.to_path_buf(), entries, effective_preset, anomalies))
    }

    pub fn hydrate_selectable_files(
        &self,
        project: &Project,
        cancel_token: &CancelToken,
    ) -> Result<Vec<(String, String)>, PackServiceError> {
        let preset = project.preset();
        let selectable_paths: Vec<String> = project
            .entries()
            .iter()
            .filter(|e| e.can_select(preset.is_secret_explicitly_allowed(&e.rel_path)))
            .map(|e| e.rel_path.clone())
            .collect();

        let hydrate_report =
            self.scanner.read_selected_files(project.root(), &selectable_paths, cancel_token)?;

        let mut pairs: Vec<(String, String)> =
            hydrate_report.files.into_iter().map(|f| (f.rel_path, f.content)).collect();
        pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Ok(pairs)
    }

    #[allow(clippy::unused_self)]
    pub fn pack_from_hydrated_cache(
        &self,
        project: &Project,
        preset: &Preset,
        hydrated_cache: &[(String, String)],
        extra_skipped_binaries: usize,
        extra_anomalies: &[Anomaly],
    ) -> PackResult {
        let mut anomalies = project.anomalies().to_vec();
        anomalies.extend_from_slice(extra_anomalies);

        let mut blocked_secrets_count = project
            .entries()
            .iter()
            .filter(|e| e.kind == FileKind::Secret && !e.is_selected)
            .count();

        let skipped_binary_count =
            project.entries().iter().filter(|e| e.kind == FileKind::Binary).count()
                + extra_skipped_binaries;

        let mut selected_slices: Vec<(&str, &str)> =
            Vec::with_capacity(project.selected_files_count());

        for (rel_path, content) in hydrated_cache {
            let is_selected = project
                .entries()
                .binary_search_by(|e| e.rel_path.as_str().cmp(rel_path.as_str()))
                .ok()
                .and_then(|idx| project.entries().get(idx))
                .is_some_and(|e| e.is_selected);

            if !is_selected {
                continue;
            }

            if !preset.is_secret_explicitly_allowed(rel_path)
                && let Some(reason) = SecretGuard::inspect_content_prefix(content)
            {
                blocked_secrets_count += 1;
                anomalies.push(Anomaly::SecretBlocked {
                    rel_path: rel_path.clone(),
                    reason: reason.as_str().to_owned(),
                });
                continue;
            }

            selected_slices.push((rel_path.as_str(), content.as_str()));
        }

        let packed_files_count = selected_slices.len();
        let chunks = Chunker::pack_into_chunks(
            &selected_slices,
            &preset.template,
            preset.minify_code,
            preset.normalize_line_endings,
        );

        PackResult::new(
            chunks,
            packed_files_count,
            blocked_secrets_count,
            skipped_binary_count,
            anomalies,
        )
    }

    pub fn pack_project(
        &self,
        project: &Project,
        cancel_token: &CancelToken,
    ) -> Result<PackResult, PackServiceError> {
        let selected_paths = project.selected_rel_paths();
        let HydrateReport { files, skipped_binaries, anomalies: hydrate_anomalies } =
            self.scanner.read_selected_files(project.root(), &selected_paths, cancel_token)?;

        let hydrated_pairs: Vec<(String, String)> =
            files.into_iter().map(|f| (f.rel_path, f.content)).collect();

        Ok(self.pack_from_hydrated_cache(
            project,
            project.preset(),
            &hydrated_pairs,
            skipped_binaries,
            &hydrate_anomalies,
        ))
    }

    pub fn pack_and_copy_to_clipboard(
        &self,
        project: &Project,
        cancel_token: &CancelToken,
    ) -> Result<PackResult, PackServiceError> {
        let result = self.pack_project(project, cancel_token)?;
        self.clipboard.copy_text(result.primary_text())?;
        Ok(result)
    }

    pub fn pack_headless(
        &self,
        project_root: &Path,
        preset: Preset,
        from_untrusted_project_config: bool,
        cancel_token: &CancelToken,
    ) -> Result<PackResult, PackServiceError> {
        let project =
            self.index_project(project_root, preset, from_untrusted_project_config, cancel_token)?;
        self.pack_and_copy_to_clipboard(&project, cancel_token)
    }
}
