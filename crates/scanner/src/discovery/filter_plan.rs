use core::preset::models::preset::{FolderOverrideMode, Preset};
use core::shared_kernel::fs::file_kind::split_file_name_and_ext;
use globset::{Glob, GlobMatcher, GlobSet, GlobSetBuilder};
use regex::{Regex, RegexBuilder};

use super::parallel_walker::ScannerError;

const MAX_STACK_PATH_BYTES: usize = 1024;
const REGEX_DFA_SIZE_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderWalkVerdict {
    /// Папка разрешена: обходчик заходит внутрь и передаёт файлы в файловую воронку.
    IncludeSubtree,
    /// Прозрачный транзит: папка исключена, но является предком для более глубокого `include`.
    /// Файлы, лежащие непосредственно в этой папке, отбрасываются.
    TraverseOnly,
    /// Полная обрезка поддерева: обходчик не заходит внутрь папки (0 системных вызовов `readdir`).
    PruneSubtree,
}

impl FolderWalkVerdict {
    #[inline]
    pub const fn allows_descent(self) -> bool {
        !matches!(self, Self::PruneSubtree)
    }

    #[inline]
    pub const fn allows_direct_files(self) -> bool {
        matches!(self, Self::IncludeSubtree)
    }
}

struct StackNormalizedPath<'a> {
    borrowed: Option<&'a str>,
    buf: [u8; MAX_STACK_PATH_BYTES],
    len: usize,
    heap_fallback: Option<String>,
}

impl<'a> StackNormalizedPath<'a> {
    #[inline]
    fn new(raw: &'a str) -> Self {
        let trimmed =
            raw.trim().trim_start_matches("./").trim_start_matches(".\\").trim_matches(['/', '\\']);
        if !trimmed.contains('\\') {
            return Self {
                borrowed: Some(trimmed),
                buf: [0_u8; MAX_STACK_PATH_BYTES],
                len: 0,
                heap_fallback: None,
            };
        }
        let bytes = trimmed.as_bytes();
        if bytes.len() <= MAX_STACK_PATH_BYTES {
            let mut buf = [0_u8; MAX_STACK_PATH_BYTES];
            for (idx, &b) in bytes.iter().enumerate() {
                if let Some(slot) = buf.get_mut(idx) {
                    *slot = if b == b'\\' { b'/' } else { b };
                }
            }
            Self { borrowed: None, buf, len: bytes.len(), heap_fallback: None }
        } else {
            Self {
                borrowed: None,
                buf: [0_u8; MAX_STACK_PATH_BYTES],
                len: 0,
                heap_fallback: Some(trimmed.replace('\\', "/")),
            }
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        if let Some(b) = self.borrowed {
            return b;
        }
        if let Some(ref h) = self.heap_fallback {
            return h.as_str();
        }
        std::str::from_utf8(self.buf.get(..self.len).unwrap_or(&[])).unwrap_or("")
    }
}

#[derive(Debug, Clone)]
struct GlobOverrideRule {
    matcher: GlobMatcher,
    root_prefix: Option<Box<str>>,
    specificity_segments: usize,
    specificity_chars: usize,
    mode: FolderOverrideMode,
}

#[derive(Debug, Clone)]
struct LiteralOverrideRule {
    path: Box<str>,
    segments: usize,
    mode: FolderOverrideMode,
}

/// Предскомпилированный план двухуровневой фильтрации (Папки + Файлы),
/// выполняющий проверки в горячем цикле обхода с 0 аллокаций кучи.
#[derive(Debug, Clone)]
pub struct CompiledFilterPlan {
    exclude_folder_names: Vec<Box<str>>,
    folder_exclude_globs: GlobSet,
    folder_exclude_roots: Vec<Box<str>>,
    only_folders: Vec<Box<str>>,
    ancestor_prefixes: Vec<Box<str>>,
    has_glob_include_override: bool,
    literal_overrides: Vec<LiteralOverrideRule>,
    glob_overrides: Vec<GlobOverrideRule>,
    folder_include_regex: Option<Regex>,
    folder_exclude_regex: Option<Regex>,
    include_extensions: Vec<Box<str>>,
    exclude_extensions: Vec<Box<str>>,
    file_include_globs: GlobSet,
    file_exclude_globs: GlobSet,
    has_file_include_globs: bool,
    file_include_regex: Option<Regex>,
    file_exclude_regex: Option<Regex>,
}

impl CompiledFilterPlan {
    pub fn compile(preset: &Preset) -> Result<Self, ScannerError> {
        let ff = &preset.folder_filters;
        let f = &preset.filters;

        let exclude_folder_names = ff
            .exclude_names
            .iter()
            .map(|s| normalize_segment(s).into_boxed_str())
            .filter(|s| !s.is_empty())
            .collect();

        let (folder_exclude_globs, folder_exclude_roots) =
            compile_folder_exclude_globs(&ff.exclude_globs)?;

        let only_folders: Vec<Box<str>> = ff
            .only_folders
            .iter()
            .map(|s| normalize_clean_path(s).into_boxed_str())
            .filter(|s| !s.is_empty())
            .collect();

        let mut ancestor_prefixes: Vec<Box<str>> = Vec::new();
        for folder in &only_folders {
            collect_strict_ancestors(folder, &mut ancestor_prefixes);
        }

        let (literal_overrides, glob_overrides, has_glob_include_override) =
            compile_folder_overrides(&ff.folder_overrides, &mut ancestor_prefixes)?;

        let folder_include_regex =
            compile_optional_regex(ff.include_regex_folders.as_deref(), ff.regex_case_sensitive)?;
        let folder_exclude_regex =
            compile_optional_regex(ff.exclude_regex_folders.as_deref(), ff.regex_case_sensitive)?;

        let include_extensions = f
            .include_extensions
            .iter()
            .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase().into_boxed_str())
            .filter(|e| !e.is_empty())
            .collect();
        let exclude_extensions = f
            .exclude_extensions
            .iter()
            .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase().into_boxed_str())
            .filter(|e| !e.is_empty())
            .collect();

        let (file_include_globs, has_file_include_globs) =
            compile_combined_globs(&preset.include_globs, &f.include_globs)?;
        for pat in preset.include_globs.iter().chain(&f.include_globs) {
            if let Some(static_prefix) = extract_static_glob_prefix(&normalize_clean_path(pat)) {
                collect_strict_ancestors(&static_prefix, &mut ancestor_prefixes);
                push_unique_boxed(&mut ancestor_prefixes, &static_prefix);
            }
        }
        let (file_exclude_globs, _) =
            compile_combined_globs(&preset.exclude_globs, &f.exclude_globs)?;

        let file_include_regex =
            compile_optional_regex(f.include_regex.as_deref(), f.regex_case_sensitive)?;
        let file_exclude_regex =
            compile_optional_regex(f.exclude_regex.as_deref(), f.regex_case_sensitive)?;

        Ok(Self {
            exclude_folder_names,
            folder_exclude_globs,
            folder_exclude_roots,
            only_folders,
            ancestor_prefixes,
            has_glob_include_override,
            literal_overrides,
            glob_overrides,
            folder_include_regex,
            folder_exclude_regex,
            include_extensions,
            exclude_extensions,
            file_include_globs,
            file_exclude_globs,
            has_file_include_globs,
            file_include_regex,
            file_exclude_regex,
        })
    }

    /// Проверяет папку на входе (ДО чтения её содержимого) с 0 аллокаций кучи.
    #[inline]
    pub fn evaluate_folder(&self, raw_rel_path: &str, is_gitignored: bool) -> FolderWalkVerdict {
        let stack_norm = StackNormalizedPath::new(raw_rel_path);
        let rel_path = stack_norm.as_str();
        if rel_path.is_empty() {
            return if self.only_folders.is_empty() {
                FolderWalkVerdict::IncludeSubtree
            } else {
                FolderWalkVerdict::TraverseOnly
            };
        }

        let dir_name = match rel_path.rfind('/') {
            Some(pos) => rel_path.get(pos + 1..).unwrap_or(rel_path),
            None => rel_path,
        };

        if dir_name == ".git" || dir_name == ".ctxpack" {
            return FolderWalkVerdict::PruneSubtree;
        }

        let has_included_descendant = self.is_ancestor_of_included(rel_path);
        let fallback_if_excluded = if has_included_descendant {
            FolderWalkVerdict::TraverseOnly
        } else {
            FolderWalkVerdict::PruneSubtree
        };

        let (override_mode, is_exact_override) = self.resolve_folder_override(rel_path);
        if override_mode == Some(FolderOverrideMode::Exclude) {
            return fallback_if_excluded;
        }
        let revived_by_override = override_mode == Some(FolderOverrideMode::Include);
        if revived_by_override && is_exact_override {
            return FolderWalkVerdict::IncludeSubtree;
        }

        if is_gitignored && !revived_by_override {
            return fallback_if_excluded;
        }

        if !revived_by_override
            && self.is_in_excluded_folder_names(dir_name)
            && !self.is_in_only_folders(rel_path)
        {
            return fallback_if_excluded;
        }

        if !revived_by_override && self.matches_folder_exclude_glob(rel_path) {
            return fallback_if_excluded;
        }

        if !self.only_folders.is_empty()
            && !revived_by_override
            && !self.is_in_only_folders(rel_path)
        {
            return fallback_if_excluded;
        }

        if !revived_by_override {
            if let Some(ref exc_re) = self.folder_exclude_regex
                && exc_re.is_match(rel_path)
            {
                return fallback_if_excluded;
            }
            if let Some(ref inc_re) = self.folder_include_regex
                && !inc_re.is_match(rel_path)
            {
                return fallback_if_excluded;
            }
        }

        FolderWalkVerdict::IncludeSubtree
    }

    /// Проверяет файл внутри уже проверенной родительской папки с 0 аллокаций кучи.
    #[inline]
    pub fn evaluate_file(
        &self,
        raw_rel_path: &str,
        parent_verdict: FolderWalkVerdict,
        is_gitignored: bool,
    ) -> bool {
        if !parent_verdict.allows_direct_files() || is_gitignored {
            return false;
        }

        let stack_norm = StackNormalizedPath::new(raw_rel_path);
        let rel_path = stack_norm.as_str();
        if rel_path.is_empty() {
            return false;
        }

        let (_, ext_opt) = split_file_name_and_ext(rel_path);

        if !self.exclude_extensions.is_empty()
            && let Some(ext) = ext_opt
            && self.exclude_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
        {
            return false;
        }

        if !self.include_extensions.is_empty() {
            let Some(ext) = ext_opt else {
                return false;
            };
            if !self.include_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                return false;
            }
        }

        let matches_inc_glob =
            self.has_file_include_globs && self.file_include_globs.is_match(rel_path);
        if self.file_exclude_globs.is_match(rel_path) && !matches_inc_glob {
            return false;
        }
        if self.has_file_include_globs && !matches_inc_glob {
            return false;
        }

        if let Some(ref exc_re) = self.file_exclude_regex
            && exc_re.is_match(rel_path)
        {
            return false;
        }
        if let Some(ref inc_re) = self.file_include_regex
            && !inc_re.is_match(rel_path)
        {
            return false;
        }

        true
    }

    #[inline]
    pub fn has_custom_override_includes(&self) -> bool {
        self.literal_overrides.iter().any(|r| r.mode == FolderOverrideMode::Include)
            || self.has_glob_include_override
    }

    #[inline]
    fn is_ancestor_of_included(&self, rel_path: &str) -> bool {
        self.ancestor_prefixes.iter().any(|anc| anc.as_ref() == rel_path)
    }

    #[inline]
    fn is_in_excluded_folder_names(&self, dir_name: &str) -> bool {
        self.exclude_folder_names.iter().any(|n| n.eq_ignore_ascii_case(dir_name))
    }

    #[inline]
    fn is_in_only_folders(&self, rel_path: &str) -> bool {
        self.only_folders.iter().any(|allowed| is_same_or_subpath(rel_path, allowed))
    }

    #[inline]
    fn matches_folder_exclude_glob(&self, rel_path: &str) -> bool {
        if self.folder_exclude_globs.is_match(rel_path) {
            return true;
        }
        self.folder_exclude_roots.iter().any(|root| is_same_or_subpath(rel_path, root))
    }

    /// Разрешает точечные и glob-правила `folder_overrides` по принципу максимальной специфичности.
    fn resolve_folder_override(&self, rel_path: &str) -> (Option<FolderOverrideMode>, bool) {
        let mut best_score: Option<((usize, u8, usize), FolderOverrideMode, bool)> = None;

        for rule in &self.literal_overrides {
            if is_same_or_subpath(rel_path, &rule.path) {
                let is_exact = rel_path.len() == rule.path.len();
                let score = (rule.segments, 2_u8, rule.path.len());
                if best_score.is_none_or(|(prev, _, _)| score > prev) {
                    best_score = Some((score, rule.mode, is_exact));
                }
            }
        }

        for rule in &self.glob_overrides {
            let direct_match = rule.matcher.is_match(rel_path)
                || rule
                    .root_prefix
                    .as_deref()
                    .is_some_and(|prefix| is_same_or_subpath(rel_path, prefix));

            let matched_depth = if direct_match {
                Some((count_segments(rel_path), true))
            } else {
                find_matching_ancestor_depth(rel_path, &rule.matcher).map(|d| (d, false))
            };

            if let Some((depth, is_exact)) = matched_depth {
                let effective_segments = rule.specificity_segments.max(depth);
                let score = (effective_segments, 1_u8, rule.specificity_chars);
                if best_score.is_none_or(|(prev, _, _)| score > prev) {
                    best_score = Some((score, rule.mode, is_exact));
                }
            }
        }

        match best_score {
            Some((_, mode, is_exact)) => (Some(mode), is_exact),
            None => (None, false),
        }
    }
}

type CompiledOverridesTuple = (Vec<LiteralOverrideRule>, Vec<GlobOverrideRule>, bool);

fn compile_folder_overrides(
    overrides: &std::collections::BTreeMap<String, FolderOverrideMode>,
    ancestor_prefixes: &mut Vec<Box<str>>,
) -> Result<CompiledOverridesTuple, ScannerError> {
    let mut literal_overrides = Vec::new();
    let mut glob_overrides = Vec::new();
    let mut has_glob_include_override = false;

    for (raw_pat, &mode) in overrides {
        let clean = normalize_clean_path(raw_pat);
        if clean.is_empty() {
            continue;
        }
        if is_glob_syntax(&clean) {
            if mode == FolderOverrideMode::Include {
                has_glob_include_override = true;
                if let Some(static_prefix) = extract_static_glob_prefix(&clean) {
                    collect_strict_ancestors(&static_prefix, ancestor_prefixes);
                    push_unique_boxed(ancestor_prefixes, &static_prefix);
                }
            }
            let glob = Glob::new(&clean)
                .map_err(|source| ScannerError::InvalidGlob { pattern: clean.clone(), source })?;
            let root_prefix = clean
                .strip_suffix("/**")
                .or_else(|| clean.strip_suffix("/*"))
                .filter(|s| !is_glob_syntax(s))
                .map(|s| s.to_owned().into_boxed_str());
            let specificity_segments = count_segments(&clean);
            let specificity_chars =
                clean.bytes().filter(|&b| b != b'*' && b != b'?' && b != b'/').count();
            glob_overrides.push(GlobOverrideRule {
                matcher: glob.compile_matcher(),
                root_prefix,
                specificity_segments,
                specificity_chars,
                mode,
            });
        } else {
            if mode == FolderOverrideMode::Include {
                collect_strict_ancestors(&clean, ancestor_prefixes);
            }
            let segments = count_segments(&clean);
            literal_overrides.push(LiteralOverrideRule {
                path: clean.into_boxed_str(),
                segments,
                mode,
            });
        }
    }

    literal_overrides.sort_unstable_by_key(|a| std::cmp::Reverse(a.path.len()));
    Ok((literal_overrides, glob_overrides, has_glob_include_override))
}

#[inline]
fn is_same_or_subpath(candidate: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    candidate.starts_with(prefix)
        && (candidate.len() == prefix.len()
            || candidate.as_bytes().get(prefix.len()).copied() == Some(b'/'))
}

fn find_matching_ancestor_depth(rel_path: &str, matcher: &GlobMatcher) -> Option<usize> {
    let mut best_depth = None;
    for (idx, b) in rel_path.bytes().enumerate() {
        if b == b'/'
            && idx > 0
            && let Some(anc) = rel_path.get(..idx)
            && matcher.is_match(anc)
        {
            best_depth = Some(count_segments(anc));
        }
    }
    best_depth
}

#[inline]
fn count_segments(path: &str) -> usize {
    if path.is_empty() {
        return 0;
    }
    path.bytes().filter(|&b| b == b'/').count() + 1
}

#[inline]
fn is_glob_syntax(s: &str) -> bool {
    s.contains(['*', '?', '[', '{'])
}

fn extract_static_glob_prefix(pattern: &str) -> Option<String> {
    let mut last_slash = None;
    for (idx, b) in pattern.bytes().enumerate() {
        if matches!(b, b'*' | b'?' | b'[' | b'{') {
            break;
        }
        if b == b'/' {
            last_slash = Some(idx);
        }
    }
    if !is_glob_syntax(pattern) {
        return (!pattern.is_empty()).then(|| pattern.to_owned());
    }
    last_slash.and_then(|pos| pattern.get(..pos)).filter(|s| !s.is_empty()).map(str::to_owned)
}

fn collect_strict_ancestors(path: &str, out: &mut Vec<Box<str>>) {
    for (idx, b) in path.bytes().enumerate() {
        if b == b'/'
            && idx > 0
            && let Some(anc) = path.get(..idx)
        {
            push_unique_boxed(out, anc);
        }
    }
}

fn push_unique_boxed(out: &mut Vec<Box<str>>, val: &str) {
    if !val.is_empty() && !out.iter().any(|existing| existing.as_ref() == val) {
        out.push(val.to_owned().into_boxed_str());
    }
}

fn normalize_segment(raw: &str) -> String {
    raw.trim().trim_matches(['/', '\\']).to_owned()
}

fn normalize_clean_path(raw: &str) -> String {
    let replaced = raw.trim().replace('\\', "/");
    let mut s = replaced.as_str();
    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped;
    }
    s.trim_matches('/').to_owned()
}

fn compile_folder_exclude_globs(
    patterns: &[String],
) -> Result<(GlobSet, Vec<Box<str>>), ScannerError> {
    let mut builder = GlobSetBuilder::new();
    let mut roots = Vec::new();
    for raw in patterns {
        let clean = normalize_clean_path(raw);
        if clean.is_empty() {
            continue;
        }
        if let Some(prefix) = clean.strip_suffix("/**").or_else(|| clean.strip_suffix("/*"))
            && !is_glob_syntax(prefix)
            && !prefix.is_empty()
        {
            push_unique_boxed(&mut roots, prefix);
        } else if !is_glob_syntax(&clean) {
            push_unique_boxed(&mut roots, &clean);
        }
        let glob = Glob::new(&clean)
            .map_err(|source| ScannerError::InvalidGlob { pattern: clean, source })?;
        builder.add(glob);
    }
    let set = builder
        .build()
        .map_err(|source| ScannerError::InvalidGlob { pattern: "*".to_owned(), source })?;
    Ok((set, roots))
}

fn compile_combined_globs(
    legacy: &[String],
    v2: &[String],
) -> Result<(GlobSet, bool), ScannerError> {
    let mut builder = GlobSetBuilder::new();
    let mut count = 0_usize;
    for raw in legacy.iter().chain(v2.iter()) {
        let clean = normalize_clean_path(raw);
        if clean.is_empty() {
            continue;
        }
        let glob = Glob::new(&clean)
            .map_err(|source| ScannerError::InvalidGlob { pattern: clean, source })?;
        builder.add(glob);
        count += 1;
    }
    let set = builder
        .build()
        .map_err(|source| ScannerError::InvalidGlob { pattern: "*".to_owned(), source })?;
    Ok((set, count > 0))
}

fn compile_optional_regex(
    pattern_opt: Option<&str>,
    case_sensitive: bool,
) -> Result<Option<Regex>, ScannerError> {
    let Some(raw) = pattern_opt.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let regex = RegexBuilder::new(raw)
        .case_insensitive(!case_sensitive)
        .dfa_size_limit(REGEX_DFA_SIZE_LIMIT)
        .build()
        .map_err(|err| ScannerError::InvalidRegex {
            pattern: raw.to_owned(),
            reason: err.to_string(),
        })?;
    Ok(Some(regex))
}
