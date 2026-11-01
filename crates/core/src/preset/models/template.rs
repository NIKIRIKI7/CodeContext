use serde::{Deserialize, Serialize};

pub const DEFAULT_FILE_HEADER_PREFIX: &str = "=== FILE: ";
pub const DEFAULT_FILE_HEADER_SUFFIX: &str = " ===\n";
pub const DEFAULT_FILE_FOOTER: &str = "\n=== END FILE ===\n\n";

pub const DEFAULT_SEARCH_MARKER: &str = "<<<<<<< SEARCH";
pub const DEFAULT_DIVIDER_MARKER: &str = "=======";
pub const DEFAULT_REPLACE_MARKER: &str = ">>>>>>> REPLACE";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchReplaceMarkers {
    pub search_header: String,
    pub divider: String,
    pub replace_footer: String,
}

impl Default for SearchReplaceMarkers {
    fn default() -> Self {
        Self {
            search_header: DEFAULT_SEARCH_MARKER.to_owned(),
            divider: DEFAULT_DIVIDER_MARKER.to_owned(),
            replace_footer: DEFAULT_REPLACE_MARKER.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserConfig {
    pub sync_with_output_template: bool,
    pub enable_markdown_fences: bool,
    pub enable_search_replace: bool,
    pub enable_unified_diff: bool,
    pub strip_project_root_prefix: bool,
    pub comment_path_prefixes: Vec<String>,
    pub search_replace_markers: SearchReplaceMarkers,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::canonical()
    }
}

impl ParserConfig {
    #[inline]
    pub fn canonical() -> Self {
        Self {
            sync_with_output_template: true,
            enable_markdown_fences: true,
            enable_search_replace: true,
            enable_unified_diff: true,
            strip_project_root_prefix: true,
            comment_path_prefixes: vec![
                "filepath:".to_owned(),
                "file:".to_owned(),
                "path:".to_owned(),
                "файл:".to_owned(),
            ],
            search_replace_markers: SearchReplaceMarkers::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputTemplate {
    pub include_file_tree: bool,
    pub preamble: String,
    pub file_header_prefix: String,
    pub file_header_suffix: String,
    pub file_footer: String,
    pub postamble: String,
    pub max_chunk_bytes: Option<usize>,
}

impl Default for OutputTemplate {
    fn default() -> Self {
        Self::canonical()
    }
}

impl OutputTemplate {
    #[inline]
    pub fn canonical() -> Self {
        Self {
            include_file_tree: false,
            preamble: String::new(),
            file_header_prefix: DEFAULT_FILE_HEADER_PREFIX.to_owned(),
            file_header_suffix: DEFAULT_FILE_HEADER_SUFFIX.to_owned(),
            file_footer: DEFAULT_FILE_FOOTER.to_owned(),
            postamble: String::new(),
            max_chunk_bytes: None,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_file_tree(mut self, include_file_tree: bool) -> Self {
        self.include_file_tree = include_file_tree;
        self
    }

    #[must_use]
    #[inline]
    pub fn with_max_chunk_bytes(mut self, max_chunk_bytes: Option<usize>) -> Self {
        self.max_chunk_bytes = max_chunk_bytes;
        self
    }

    #[inline]
    pub fn fixed_marker_bytes_per_file(&self) -> usize {
        self.file_header_prefix.len() + self.file_header_suffix.len() + self.file_footer.len()
    }

    #[inline]
    pub fn header_suffix_trimmed(&self) -> &str {
        self.file_header_suffix.trim_end_matches(['\r', '\n'])
    }

    #[inline]
    pub fn footer_trimmed(&self) -> &str {
        self.file_footer.trim_matches(['\r', '\n'])
    }
}
