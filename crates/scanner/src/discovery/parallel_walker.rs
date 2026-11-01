use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use core::packing::ports::scanner::{
    FileScannerPort, HydrateReport, LoadedTextFile, RawScannedEntry, ScanWalkReport,
    ScannerPortError,
};
use core::preset::models::preset::Preset;
use core::preset::models::template::{
    DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX, DEFAULT_FILE_HEADER_SUFFIX,
};
use core::shared_kernel::execution::anomaly::Anomaly;
use core::shared_kernel::execution::cancel_token::CancelToken;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{DirEntry, Error as IgnoreError, WalkBuilder, WalkState};
use rayon::prelude::*;
use thiserror::Error;

use crate::content::pipeline::append_transformed_content;
use crate::content::stream_reader::read_text_file;
use crate::discovery::binary_detector::{is_binary_extension, is_binary_probe, is_binary_slice};

const FIXED_MARKER_BYTES_PER_FILE: usize =
    DEFAULT_FILE_HEADER_PREFIX.len() + DEFAULT_FILE_HEADER_SUFFIX.len() + DEFAULT_FILE_FOOTER.len();

#[derive(Debug, Error)]
pub(crate) enum ScannerError {
    #[error("Некорректный glob-паттерн `{pattern}`: {source}")]
    InvalidGlob {
        pattern: String,
        #[source]
        source: globset::Error,
    },
}

struct ThreadBatchSender {
    batch: Vec<ScannedFile>,
    tx: std::sync::mpsc::Sender<Vec<ScannedFile>>,
}

impl Drop for ThreadBatchSender {
    fn drop(&mut self) {
        if !self.batch.is_empty() {
            let finished = std::mem::take(&mut self.batch);
            let _ = self.tx.send(finished);
        }
    }
}

struct WalkBatchPayload {
    entries: Vec<RawScannedEntry>,
    anomalies: Vec<Anomaly>,
}

struct WorkerBatch {
    entries: Vec<RawScannedEntry>,
    anomalies: Vec<Anomaly>,
    tx: std::sync::mpsc::Sender<WalkBatchPayload>,
}

impl Drop for WorkerBatch {
    fn drop(&mut self) {
        if !self.entries.is_empty() || !self.anomalies.is_empty() {
            let payload = WalkBatchPayload {
                entries: std::mem::take(&mut self.entries),
                anomalies: std::mem::take(&mut self.anomalies),
            };
            let _ = self.tx.send(payload);
        }
    }
}

struct WalkFilterContext {
    root: PathBuf,
    exclude_matcher: GlobSet,
    include_matcher: GlobSet,
    has_includes: bool,
    max_depth: usize,
    max_files: usize,
    follow_symlinks: bool,
}

enum ReadOutcome {
    Text(LoadedTextFile),
    BinarySkipped,
    Unreadable(Anomaly),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub abs_path: PathBuf,
    pub rel_path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IgnoreFileScanner;

impl IgnoreFileScanner {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    #[inline]
    pub fn is_binary(bytes: &[u8]) -> bool {
        is_binary_probe(bytes)
    }

    #[inline]
    pub fn is_binary_full_slice(bytes: &[u8]) -> bool {
        is_binary_slice(bytes)
    }

    #[inline]
    pub fn is_known_binary_path(rel_path: &str) -> bool {
        is_binary_extension(rel_path)
    }

    #[inline]
    pub fn calculate_exact_capacity<P, C>(files: &[(P, C)]) -> usize
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        files
            .iter()
            .map(|(path, content)| {
                path.as_ref().len() + content.as_ref().len() + FIXED_MARKER_BYTES_PER_FILE
            })
            .sum()
    }

    #[inline]
    pub fn pack_slices_preallocated<P, C>(files: &[(P, C)], minify: bool) -> String
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        Self::pack_slices_with_options(files, minify, false)
    }

    #[inline]
    pub fn pack_slices_with_options<P, C>(
        files: &[(P, C)],
        minify: bool,
        normalize_lf: bool,
    ) -> String
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        let capacity = Self::calculate_exact_capacity(files);
        let mut output = String::with_capacity(capacity);
        for (path, content) in files {
            let path_str = path.as_ref();
            output.push_str(DEFAULT_FILE_HEADER_PREFIX);
            output.push_str(path_str);
            output.push_str(DEFAULT_FILE_HEADER_SUFFIX);
            append_transformed_content(
                path_str,
                content.as_ref(),
                &mut output,
                minify,
                normalize_lf,
            );
            output.push_str(DEFAULT_FILE_FOOTER);
        }
        output
    }

    pub fn scan_project_index(
        &self,
        project_root: &Path,
        respect_gitignore: bool,
        max_depth: usize,
        cancel_token: &CancelToken,
    ) -> Vec<ScannedFile> {
        let walker = WalkBuilder::new(project_root)
            .hidden(false)
            .git_ignore(respect_gitignore)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .ignore(respect_gitignore)
            .parents(false)
            .max_depth(Some(max_depth))
            .build_parallel();

        let (tx, rx) = std::sync::mpsc::channel::<Vec<ScannedFile>>();
        walker.run(|| {
            let token = cancel_token.clone();
            let root = project_root.to_path_buf();
            let mut sender = ThreadBatchSender { batch: Vec::with_capacity(256), tx: tx.clone() };
            Box::new(move |entry| {
                if token.is_cancelled() {
                    return WalkState::Quit;
                }
                if let Ok(dir_entry) = entry
                    && dir_entry.file_type().is_some_and(|ft| ft.is_file())
                {
                    let abs_path = dir_entry.into_path();
                    if abs_path.file_name().is_some_and(|n| n == ".gitignore" || n == ".ignore") {
                        return WalkState::Continue;
                    }
                    let size_bytes = abs_path.metadata().map_or(0, |m| m.len());
                    let rel_path = normalize_rel_path(&abs_path, &root);
                    sender.batch.push(ScannedFile { abs_path, rel_path, size_bytes });
                }
                WalkState::Continue
            })
        });
        drop(tx);

        let batches: Vec<Vec<ScannedFile>> = rx.into_iter().collect();
        let total_files: usize = batches.iter().map(Vec::len).sum();
        let mut all_files = Vec::with_capacity(total_files);
        for batch in batches {
            all_files.extend(batch);
        }
        all_files
    }

    pub fn pack_project_end_to_end(
        &self,
        project_root: &Path,
        exclude_globs: &[&str],
        minify: bool,
        cancel_token: &CancelToken,
    ) -> Result<String> {
        let index = self.scan_project_index(project_root, true, 32, cancel_token);
        let exclude_set = build_glob_set(exclude_globs).context("Ошибка сборки glob-правил")?;
        let loaded_files: Vec<(String, String)> = index
            .into_par_iter()
            .filter(|item| !exclude_set.is_match(&item.rel_path))
            .filter_map(|item| {
                let content = read_text_file(&item.abs_path).ok()??;
                Some((item.rel_path, content))
            })
            .collect();
        Ok(Self::pack_slices_preallocated(&loaded_files, minify))
    }
}

impl FileScannerPort for IgnoreFileScanner {
    fn scan_index(
        &self,
        project_root: &Path,
        preset: &Preset,
        cancel_token: &CancelToken,
    ) -> std::result::Result<ScanWalkReport, ScannerPortError> {
        if !project_root.exists() || !project_root.is_dir() {
            return Err(ScannerPortError::InvalidProjectRoot(project_root.display().to_string()));
        }

        let (exclude_set, include_set) = compile_preset_globs(preset)?;
        let filter_ctx = WalkFilterContext {
            root: project_root.to_path_buf(),
            exclude_matcher: exclude_set,
            include_matcher: include_set,
            has_includes: !preset.include_globs.is_empty(),
            max_depth: preset.max_depth,
            max_files: preset.max_files,
            follow_symlinks: preset.follow_symlinks,
        };

        let preset_for_filter = preset.clone();
        let walker = WalkBuilder::new(project_root)
            .hidden(false)
            .git_ignore(preset.respect_gitignore)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .ignore(preset.respect_gitignore)
            .parents(false)
            .follow_links(preset.follow_symlinks)
            .max_depth(Some(preset.max_depth.saturating_add(1)))
            .filter_entry(move |entry| {
                if entry.depth() == 0 {
                    return true;
                }
                if entry.file_type().is_some_and(|ft| ft.is_dir())
                    && let Some(dir_name) = entry.file_name().to_str()
                {
                    return !preset_for_filter.should_prune_heavy_directory(dir_name);
                }
                true
            })
            .build_parallel();

        let (tx, rx) = std::sync::mpsc::channel::<WalkBatchPayload>();
        let discovered_counter = AtomicUsize::new(0);

        walker.run(|| {
            let token = cancel_token.clone();
            let ctx_ref = &filter_ctx;
            let counter_ref = &discovered_counter;
            let mut batch = WorkerBatch {
                entries: Vec::with_capacity(256),
                anomalies: Vec::new(),
                tx: tx.clone(),
            };
            Box::new(move |entry_res| {
                if token.is_cancelled() {
                    return WalkState::Quit;
                }
                process_walk_entry(entry_res, ctx_ref, counter_ref, &mut batch)
            })
        });
        drop(tx);

        if cancel_token.is_cancelled() {
            return Err(ScannerPortError::Cancelled);
        }

        let mut all_entries = Vec::new();
        let mut all_anomalies = Vec::new();
        for payload in rx {
            all_entries.extend(payload.entries);
            all_anomalies.extend(payload.anomalies);
        }
        all_entries.sort_unstable_by(|a, b| a.rel_path.cmp(&b.rel_path));

        Ok(ScanWalkReport { entries: all_entries, anomalies: all_anomalies })
    }

    fn read_selected_files(
        &self,
        project_root: &Path,
        selected_rel_paths: &[String],
        cancel_token: &CancelToken,
    ) -> std::result::Result<HydrateReport, ScannerPortError> {
        if cancel_token.is_cancelled() {
            return Err(ScannerPortError::Cancelled);
        }

        let outcomes: Vec<ReadOutcome> = selected_rel_paths
            .par_iter()
            .map(|rel_path| {
                if cancel_token.is_cancelled() {
                    return ReadOutcome::Cancelled;
                }
                if is_binary_extension(rel_path) {
                    return ReadOutcome::BinarySkipped;
                }
                let full_path = project_root.join(rel_path);
                match read_text_file(&full_path) {
                    Ok(Some(content)) => {
                        ReadOutcome::Text(LoadedTextFile { rel_path: rel_path.clone(), content })
                    }
                    Ok(None) => ReadOutcome::BinarySkipped,
                    Err(err) => ReadOutcome::Unreadable(Anomaly::UnreadableOrInvalidUtf8 {
                        rel_path: rel_path.clone(),
                        reason: err.to_string(),
                    }),
                }
            })
            .collect();

        if cancel_token.is_cancelled() {
            return Err(ScannerPortError::Cancelled);
        }

        let mut files = Vec::with_capacity(outcomes.len());
        let mut skipped_binaries = 0;
        let mut anomalies = Vec::new();
        for outcome in outcomes {
            match outcome {
                ReadOutcome::Text(file) => files.push(file),
                ReadOutcome::BinarySkipped => skipped_binaries += 1,
                ReadOutcome::Unreadable(anomaly) => anomalies.push(anomaly),
                ReadOutcome::Cancelled => return Err(ScannerPortError::Cancelled),
            }
        }

        Ok(HydrateReport { files, skipped_binaries, anomalies })
    }
}

fn process_walk_entry(
    entry_res: std::result::Result<DirEntry, IgnoreError>,
    ctx: &WalkFilterContext,
    counter: &AtomicUsize,
    batch: &mut WorkerBatch,
) -> WalkState {
    let dir_entry = match entry_res {
        Ok(e) => e,
        Err(err) => {
            if is_symlink_loop_error(&err) {
                batch.anomalies.push(Anomaly::CyclicOrSkippedSymlink { rel_path: err.to_string() });
            }
            return WalkState::Continue;
        }
    };

    let depth = dir_entry.depth();
    if depth == 0 {
        return WalkState::Continue;
    }

    let abs_path = dir_entry.path();
    let rel_path = normalize_rel_path(abs_path, &ctx.root);

    if depth > ctx.max_depth {
        batch.anomalies.push(Anomaly::MaxDepthExceeded {
            rel_path,
            depth,
            max_depth: ctx.max_depth,
        });
        return WalkState::Skip;
    }

    if dir_entry.path_is_symlink() && !ctx.follow_symlinks {
        batch.anomalies.push(Anomaly::CyclicOrSkippedSymlink { rel_path });
        return WalkState::Skip;
    }

    if !dir_entry.file_type().is_some_and(|ft| ft.is_file()) {
        return WalkState::Continue;
    }

    if abs_path.file_name().is_some_and(|n| n == ".gitignore" || n == ".ignore") {
        return WalkState::Continue;
    }

    if ctx.exclude_matcher.is_match(&rel_path) && !ctx.include_matcher.is_match(&rel_path) {
        return WalkState::Continue;
    }

    if ctx.has_includes && !ctx.include_matcher.is_match(&rel_path) {
        return WalkState::Continue;
    }

    let current_count = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if current_count > ctx.max_files {
        if current_count == ctx.max_files + 1 {
            batch.anomalies.push(Anomaly::MaxFilesExceeded {
                discovered_files: current_count,
                max_files: ctx.max_files,
            });
        }
        return WalkState::Quit;
    }

    let size_bytes = dir_entry.metadata().map_or(0, |m| m.len());
    batch.entries.push(RawScannedEntry { rel_path, size_bytes, depth });
    WalkState::Continue
}

fn is_symlink_loop_error(err: &IgnoreError) -> bool {
    match err {
        IgnoreError::Loop { .. } => true,
        IgnoreError::WithPath { err: inner, .. }
        | IgnoreError::WithDepth { err: inner, .. }
        | IgnoreError::WithLineNumber { err: inner, .. } => is_symlink_loop_error(inner),
        IgnoreError::Partial(errs) => errs.iter().any(is_symlink_loop_error),
        _ => false,
    }
}

fn compile_preset_globs(
    preset: &Preset,
) -> std::result::Result<(GlobSet, GlobSet), ScannerPortError> {
    let exclude_refs: Vec<&str> = preset.exclude_globs.iter().map(String::as_str).collect();
    let include_refs: Vec<&str> = preset.include_globs.iter().map(String::as_str).collect();
    let exclude_set = build_glob_set(&exclude_refs).map_err(|err| match err {
        ScannerError::InvalidGlob { pattern, source } => {
            ScannerPortError::InvalidGlobPattern { pattern, reason: source.to_string() }
        }
    })?;
    let include_set = build_glob_set(&include_refs).map_err(|err| match err {
        ScannerError::InvalidGlob { pattern, source } => {
            ScannerPortError::InvalidGlobPattern { pattern, reason: source.to_string() }
        }
    })?;
    Ok((exclude_set, include_set))
}

#[inline]
fn normalize_rel_path(abs_path: &Path, root: &Path) -> String {
    let raw = abs_path.strip_prefix(root).unwrap_or(abs_path).to_string_lossy();
    if raw.contains('\\') { raw.replace('\\', "/") } else { raw.into_owned() }
}

fn build_glob_set(patterns: &[&str]) -> std::result::Result<GlobSet, ScannerError> {
    let mut builder = GlobSetBuilder::new();
    for &pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|source| ScannerError::InvalidGlob { pattern: pattern.to_owned(), source })?;
        builder.add(glob);
    }
    builder.build().map_err(|source| ScannerError::InvalidGlob { pattern: "*".to_owned(), source })
}
