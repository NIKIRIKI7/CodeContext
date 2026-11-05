use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::template::{OutputTemplate, ParserConfig};

pub const PROJECT_CONFIG_FILE_NAME: &str = ".ctxpack.json";
pub const GLOBAL_CONFIG_FILE_NAME: &str = "config.json";
pub const MAX_CONFIG_FILE_BYTES: u64 = 1_048_576;

pub const DEFAULT_HEAVY_EXCLUDED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".ctxpack",
    ".next",
    ".nuxt",
    ".venv",
    "venv",
    "__pycache__",
    "vendor",
];

pub const DEFAULT_MAX_DEPTH: usize = 32;
pub const MIN_ALLOWED_MAX_DEPTH: usize = 1;
pub const MAX_ALLOWED_MAX_DEPTH: usize = 128;

pub const DEFAULT_MAX_FILES: usize = 50_000;
pub const MIN_ALLOWED_MAX_FILES: usize = 1;
pub const MAX_ALLOWED_MAX_FILES: usize = 500_000;

pub const DEFAULT_LARGE_FILE_THRESHOLD_BYTES: u64 = 1_048_576;
pub const MIN_LARGE_FILE_THRESHOLD_BYTES: u64 = 1_024;
pub const MAX_LARGE_FILE_THRESHOLD_BYTES: u64 = 104_857_600;

pub const MIN_ALLOWED_CHUNK_BYTES: usize = 4_096;

const fn default_true() -> bool {
    true
}

pub fn default_excluded_folder_names() -> Vec<String> {
    DEFAULT_HEAVY_EXCLUDED_DIRS.iter().map(|&s| s.to_owned()).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FolderOverrideMode {
    Include,
    Exclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegexTargetScope {
    #[default]
    FilesExclude,
    FilesInclude,
    FoldersExclude,
    FoldersInclude,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegexTemplateItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    pub pattern: String,
    #[serde(default)]
    pub scope: RegexTargetScope,
    #[serde(default)]
    pub is_custom: bool,
}

impl RegexTemplateItem {
    #[inline]
    pub fn display_name(&self, is_english: bool) -> &str {
        if is_english && !self.name_en.is_empty() { &self.name_en } else { &self.name }
    }
}

pub fn default_regex_templates() -> Vec<RegexTemplateItem> {
    vec![
        RegexTemplateItem {
            id: "no_tests".to_owned(),
            name: "Без тестов".to_owned(),
            name_en: "No tests".to_owned(),
            pattern: "(^|/)tests?/|(_test|[.]test|[.]spec)[.]".to_owned(),
            scope: RegexTargetScope::FilesExclude,
            is_custom: false,
        },
        RegexTemplateItem {
            id: "only_src".to_owned(),
            name: "Только src/".to_owned(),
            name_en: "Only src/".to_owned(),
            pattern: "(^|/)src/".to_owned(),
            scope: RegexTargetScope::FilesInclude,
            is_custom: false,
        },
        RegexTemplateItem {
            id: "no_gen".to_owned(),
            name: "Без generated".to_owned(),
            name_en: "No generated".to_owned(),
            pattern: "(generated|[.]gen[.])".to_owned(),
            scope: RegexTargetScope::FilesExclude,
            is_custom: false,
        },
        RegexTemplateItem {
            id: "only_crates".to_owned(),
            name: "Только crates/".to_owned(),
            name_en: "Only crates/".to_owned(),
            pattern: "^crates/".to_owned(),
            scope: RegexTargetScope::FilesInclude,
            is_custom: false,
        },
        RegexTemplateItem {
            id: "no_benches".to_owned(),
            name: "Без бенчмарков".to_owned(),
            name_en: "No benches".to_owned(),
            pattern: "(^|/)benches/|(_bench|[.]bench)[.]".to_owned(),
            scope: RegexTargetScope::FilesExclude,
            is_custom: false,
        },
        RegexTemplateItem {
            id: "only_configs".to_owned(),
            name: "Только конфиги".to_owned(),
            name_en: "Only configs".to_owned(),
            pattern: "[.](json|toml|ya?ml|ini|cfg|conf)$".to_owned(),
            scope: RegexTargetScope::FilesInclude,
            is_custom: false,
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFilters {
    #[serde(default)]
    pub include_extensions: Vec<String>,
    #[serde(default)]
    pub exclude_extensions: Vec<String>,
    #[serde(default)]
    pub include_globs: Vec<String>,
    #[serde(default)]
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub include_regex: Option<String>,
    #[serde(default)]
    pub exclude_regex: Option<String>,
    #[serde(default = "default_true")]
    pub regex_case_sensitive: bool,
    #[serde(default = "default_regex_templates")]
    pub regex_templates: Vec<RegexTemplateItem>,
}

impl Default for FileFilters {
    fn default() -> Self {
        Self {
            include_extensions: Vec::new(),
            exclude_extensions: Vec::new(),
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            include_regex: None,
            exclude_regex: None,
            regex_case_sensitive: true,
            regex_templates: default_regex_templates(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderFilters {
    #[serde(default = "default_excluded_folder_names")]
    pub exclude_names: Vec<String>,
    #[serde(default)]
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub only_folders: Vec<String>,
    #[serde(default)]
    pub include_regex_folders: Option<String>,
    #[serde(default)]
    pub exclude_regex_folders: Option<String>,
    #[serde(default)]
    pub regex_case_sensitive: bool,
    #[serde(default)]
    pub folder_overrides: BTreeMap<String, FolderOverrideMode>,
    #[serde(default)]
    pub show_excluded_in_ui: bool,
}

impl Default for FolderFilters {
    fn default() -> Self {
        Self {
            exclude_names: default_excluded_folder_names(),
            exclude_globs: Vec::new(),
            only_folders: Vec::new(),
            include_regex_folders: None,
            exclude_regex_folders: None,
            regex_case_sensitive: false,
            folder_overrides: BTreeMap::new(),
            show_excluded_in_ui: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub respect_gitignore: bool,
    pub follow_symlinks: bool,
    pub minify_code: bool,
    pub normalize_line_endings: bool,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub filters: FileFilters,
    #[serde(default)]
    pub folder_filters: FolderFilters,
    #[serde(default)]
    pub explicitly_allowed_secrets: Vec<String>,
    pub max_depth: usize,
    pub max_files: usize,
    pub large_file_threshold_bytes: u64,
    pub template: OutputTemplate,
    #[serde(default)]
    pub parser: ParserConfig,
}

impl Default for Preset {
    fn default() -> Self {
        Self::default_code_preset()
    }
}

impl Preset {
    #[inline]
    pub fn default_code_preset() -> Self {
        Self {
            name: "Default".to_owned(),
            respect_gitignore: true,
            follow_symlinks: false,
            minify_code: false,
            normalize_line_endings: false,
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            filters: FileFilters::default(),
            folder_filters: FolderFilters::default(),
            explicitly_allowed_secrets: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            max_files: DEFAULT_MAX_FILES,
            large_file_threshold_bytes: DEFAULT_LARGE_FILE_THRESHOLD_BYTES,
            template: OutputTemplate::canonical(),
            parser: ParserConfig::canonical(),
        }
    }

    #[inline]
    pub fn is_secret_explicitly_allowed(&self, rel_path: &str) -> bool {
        self.explicitly_allowed_secrets.iter().any(|allowed| allowed == rel_path)
    }

    #[inline]
    pub fn should_prune_heavy_directory(&self, dir_name: &str) -> bool {
        if dir_name == ".git" || dir_name == ".ctxpack" {
            return true;
        }
        let in_exclude_names = self
            .folder_filters
            .exclude_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(dir_name));
        if !in_exclude_names {
            return false;
        }
        let revived_by_glob =
            self.include_globs.iter().chain(&self.filters.include_globs).any(|glob| {
                let clean = glob.trim_start_matches("./").trim_start_matches(".\\");
                clean == dir_name
                    || clean
                        .strip_prefix(dir_name)
                        .is_some_and(|rest| rest.starts_with('/') || rest.starts_with('\\'))
            });
        let revived_by_only = self.folder_filters.only_folders.iter().any(|folder| {
            let clean = folder.trim_start_matches("./").trim_start_matches(".\\");
            clean == dir_name
                || clean
                    .strip_prefix(dir_name)
                    .is_some_and(|rest| rest.starts_with('/') || rest.starts_with('\\'))
        });
        let revived_by_override = self.folder_filters.folder_overrides.iter().any(|(k, &mode)| {
            if mode != FolderOverrideMode::Include {
                return false;
            }
            let clean = k.trim_start_matches("./").trim_start_matches(".\\");
            clean == dir_name
                || clean
                    .strip_prefix(dir_name)
                    .is_some_and(|rest| rest.starts_with('/') || rest.starts_with('\\'))
        });
        !(revived_by_glob || revived_by_only || revived_by_override)
    }

    pub fn toggle_extension_filter(&mut self, ext: &str) {
        let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        if clean.is_empty() {
            return;
        }
        if let Some(pos) =
            self.filters.include_extensions.iter().position(|e| e.eq_ignore_ascii_case(&clean))
        {
            self.filters.include_extensions.remove(pos);
        } else {
            self.filters.include_extensions.push(clean);
        }
    }

    pub fn toggle_excluded_folder_name(&mut self, folder_name: &str) {
        let clean = folder_name.trim().trim_matches(['/', '\\']);
        if clean.is_empty() || clean == ".git" || clean == ".ctxpack" {
            return;
        }
        if let Some(pos) =
            self.folder_filters.exclude_names.iter().position(|n| n.eq_ignore_ascii_case(clean))
        {
            self.folder_filters.exclude_names.remove(pos);
        } else {
            self.folder_filters.exclude_names.push(clean.to_owned());
        }
    }

    /// Проверяет, является ли папка `dir_path` чисто транзитным (исключённым) предком,
    /// который был пройден только ради вложенного `only_folders` или `folder_overrides: include`.
    pub fn is_excluded_transit_folder(&self, dir_path: &str) -> bool {
        let clean_dir = dir_path.trim_matches('/');
        if clean_dir.is_empty() {
            return false;
        }

        if !self.folder_filters.only_folders.is_empty() {
            let is_inside_only = self.folder_filters.only_folders.iter().any(|only| {
                let clean_only = only.trim_matches('/');
                clean_dir == clean_only
                    || (clean_dir.starts_with(clean_only)
                        && clean_dir.as_bytes().get(clean_only.len()).copied() == Some(b'/'))
            });
            let is_strict_ancestor_of_only = self.folder_filters.only_folders.iter().any(|only| {
                let clean_only = only.trim_matches('/');
                clean_only.starts_with(clean_dir)
                    && clean_only.as_bytes().get(clean_dir.len()).copied() == Some(b'/')
            });
            if !is_inside_only && is_strict_ancestor_of_only {
                return true;
            }
        }

        let mut best_len = 0_usize;
        let mut best_mode: Option<FolderOverrideMode> = None;
        for (pattern, &mode) in &self.folder_filters.folder_overrides {
            let clean_pat = pattern.trim_matches('/');
            if clean_pat.contains(['*', '?', '[']) {
                continue;
            }
            if (clean_dir == clean_pat
                || (clean_dir.starts_with(clean_pat)
                    && clean_dir.as_bytes().get(clean_pat.len()).copied() == Some(b'/')))
                && clean_pat.len() >= best_len
            {
                best_len = clean_pat.len();
                best_mode = Some(mode);
            }
        }
        best_mode == Some(FolderOverrideMode::Exclude)
    }

    #[must_use]
    #[inline]
    pub fn sanitized_from_untrusted_project_config(mut self) -> Self {
        self.explicitly_allowed_secrets.clear();
        self
    }
}
