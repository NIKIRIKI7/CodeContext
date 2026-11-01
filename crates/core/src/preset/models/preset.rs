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
        if !DEFAULT_HEAVY_EXCLUDED_DIRS.contains(&dir_name) {
            return false;
        }
        if dir_name == ".git" || dir_name == ".ctxpack" {
            return true;
        }
        !self.include_globs.iter().any(|glob| {
            let clean = glob.trim_start_matches("./");
            clean == dir_name
                || clean
                    .strip_prefix(dir_name)
                    .is_some_and(|rest| rest.starts_with('/') || rest.starts_with('\\'))
        })
    }

    #[must_use]
    #[inline]
    pub fn sanitized_from_untrusted_project_config(mut self) -> Self {
        self.explicitly_allowed_secrets.clear();
        self
    }
}
