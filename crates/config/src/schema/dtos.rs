use std::collections::BTreeMap;

use core::preset::models::preset::{
    DEFAULT_LARGE_FILE_THRESHOLD_BYTES, DEFAULT_MAX_DEPTH, DEFAULT_MAX_FILES, FolderOverrideMode,
    RegexTemplateItem,
};
use core::preset::models::resolution::{
    KeymapConfig, TerminalCommandPreset, UiChromeConfig, default_terminal_command_presets,
};
use core::preset::models::template::{PromptBlockItem, SystemPromptMode};
use serde::{Deserialize, Serialize};

use super::templates::{config_defaults_bundle, filter_templates_bundle, prompt_templates_bundle};

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

const fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

const fn default_true() -> bool {
    true
}

fn default_preset_name() -> String {
    config_defaults_bundle().active_preset_name.clone()
}

fn default_hotkey() -> String {
    config_defaults_bundle().global_hotkey.clone()
}

fn default_update_channel() -> String {
    config_defaults_bundle().update_channel.clone()
}

fn default_update_repo_url() -> String {
    config_defaults_bundle().update_repo_url.clone()
}

const fn default_max_depth() -> usize {
    DEFAULT_MAX_DEPTH
}

const fn default_max_files() -> usize {
    DEFAULT_MAX_FILES
}

const fn default_large_file_threshold() -> u64 {
    DEFAULT_LARGE_FILE_THRESHOLD_BYTES
}

fn default_header_prefix() -> String {
    prompt_templates_bundle().output_template.file_header_prefix.clone()
}

fn default_header_suffix() -> String {
    prompt_templates_bundle().output_template.file_header_suffix.clone()
}

fn default_file_footer() -> String {
    prompt_templates_bundle().output_template.file_footer.clone()
}

fn default_search_marker() -> String {
    prompt_templates_bundle().parser_config.search_replace_markers.search_header.clone()
}

fn default_divider_marker() -> String {
    prompt_templates_bundle().parser_config.search_replace_markers.divider.clone()
}

fn default_replace_marker() -> String {
    prompt_templates_bundle().parser_config.search_replace_markers.replace_footer.clone()
}

fn default_comment_prefixes() -> Vec<String> {
    prompt_templates_bundle().parser_config.comment_path_prefixes.clone()
}

fn default_prompt_blocks_dto() -> Vec<PromptBlockItem> {
    prompt_templates_bundle().prompt_blocks.clone()
}

fn default_excluded_folder_names_dto() -> Vec<String> {
    filter_templates_bundle().default_excluded_folder_names.clone()
}

fn default_regex_templates_dto() -> Vec<RegexTemplateItem> {
    filter_templates_bundle().regex_templates.clone()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParserConfigDto {
    #[serde(default = "default_true")]
    pub sync_with_output_template: bool,
    #[serde(default = "default_true")]
    pub enable_markdown_fences: bool,
    #[serde(default = "default_true")]
    pub enable_search_replace: bool,
    #[serde(default = "default_true")]
    pub enable_unified_diff: bool,
    #[serde(default = "default_true")]
    pub strip_project_root_prefix: bool,
    #[serde(default = "default_comment_prefixes")]
    pub comment_path_prefixes: Vec<String>,
    #[serde(default = "default_search_marker")]
    pub search_header: String,
    #[serde(default = "default_divider_marker")]
    pub divider: String,
    #[serde(default = "default_replace_marker")]
    pub replace_footer: String,
}

impl Default for ParserConfigDto {
    fn default() -> Self {
        Self {
            sync_with_output_template: true,
            enable_markdown_fences: true,
            enable_search_replace: true,
            enable_unified_diff: true,
            strip_project_root_prefix: true,
            comment_path_prefixes: default_comment_prefixes(),
            search_header: default_search_marker(),
            divider: default_divider_marker(),
            replace_footer: default_replace_marker(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputTemplateDto {
    pub include_file_tree: bool,
    #[serde(default)]
    pub system_prompt_preset: SystemPromptMode,
    #[serde(default = "default_prompt_blocks_dto")]
    pub prompt_blocks: Vec<PromptBlockItem>,
    pub preamble: String,
    #[serde(default = "default_header_prefix")]
    pub file_header_prefix: String,
    #[serde(default = "default_header_suffix")]
    pub file_header_suffix: String,
    #[serde(default = "default_file_footer")]
    pub file_footer: String,
    pub postamble: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chunk_bytes: Option<usize>,
}

impl Default for OutputTemplateDto {
    fn default() -> Self {
        Self {
            include_file_tree: false,
            system_prompt_preset: SystemPromptMode::None,
            prompt_blocks: default_prompt_blocks_dto(),
            preamble: String::new(),
            file_header_prefix: default_header_prefix(),
            file_header_suffix: default_header_suffix(),
            file_footer: default_file_footer(),
            postamble: String::new(),
            max_chunk_bytes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileFiltersDto {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_globs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_globs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_regex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_regex: Option<String>,
    #[serde(default = "default_true")]
    pub regex_case_sensitive: bool,
    #[serde(default = "default_regex_templates_dto", skip_serializing_if = "Vec::is_empty")]
    pub regex_templates: Vec<RegexTemplateItem>,
}

impl Default for FileFiltersDto {
    fn default() -> Self {
        Self {
            include_extensions: Vec::new(),
            exclude_extensions: Vec::new(),
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            include_regex: None,
            exclude_regex: None,
            regex_case_sensitive: true,
            regex_templates: default_regex_templates_dto(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FolderFiltersDto {
    #[serde(default = "default_excluded_folder_names_dto")]
    pub exclude_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_globs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only_folders: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_regex_folders: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_regex_folders: Option<String>,
    #[serde(default)]
    pub regex_case_sensitive: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub folder_overrides: BTreeMap<String, FolderOverrideMode>,
    #[serde(default)]
    pub show_excluded_in_ui: bool,
}

impl Default for FolderFiltersDto {
    fn default() -> Self {
        Self {
            exclude_names: default_excluded_folder_names_dto(),
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileFiltersOverlayDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_globs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_globs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_case_sensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_templates: Option<Vec<RegexTemplateItem>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FolderFiltersOverlayDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_globs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_folders: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_regex_folders: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_regex_folders: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_case_sensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_overrides: Option<BTreeMap<String, FolderOverrideMode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_excluded_in_ui: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PresetDto {
    #[serde(default = "default_preset_name")]
    pub name: String,
    #[serde(default = "default_true")]
    pub respect_gitignore: bool,
    pub follow_symlinks: bool,
    pub minify_code: bool,
    pub normalize_line_endings: bool,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub filters: FileFiltersDto,
    #[serde(default)]
    pub folder_filters: FolderFiltersDto,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explicitly_allowed_secrets: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    #[serde(default = "default_large_file_threshold")]
    pub large_file_threshold_bytes: u64,
    pub template: OutputTemplateDto,
    #[serde(default)]
    pub parser: ParserConfigDto,
}

impl Default for PresetDto {
    fn default() -> Self {
        Self {
            name: default_preset_name(),
            respect_gitignore: true,
            follow_symlinks: false,
            minify_code: false,
            normalize_line_endings: false,
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            filters: FileFiltersDto::default(),
            folder_filters: FolderFiltersDto::default(),
            explicitly_allowed_secrets: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            max_files: DEFAULT_MAX_FILES,
            large_file_threshold_bytes: DEFAULT_LARGE_FILE_THRESHOLD_BYTES,
            template: OutputTemplateDto::default(),
            parser: ParserConfigDto::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalConfigDto {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_preset_name")]
    pub active_preset_name: String,
    pub presets: Vec<PresetDto>,
    #[serde(default = "default_hotkey")]
    pub global_hotkey: String,
    pub auto_read_clipboard_on_apply: bool,
    #[serde(default = "default_true")]
    pub create_backups_by_default: bool,
    #[serde(default = "default_true")]
    pub show_bottom_bar: bool,
    #[serde(default)]
    pub ui_chrome: UiChromeConfig,
    #[serde(default = "default_terminal_command_presets")]
    pub terminal_presets: Vec<TerminalCommandPreset>,
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
    #[serde(default = "default_update_repo_url")]
    pub update_repo_url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_projects: Vec<String>,
    #[serde(default)]
    pub keymap: KeymapConfig,
}

impl Default for GlobalConfigDto {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            active_preset_name: default_preset_name(),
            presets: vec![PresetDto::default()],
            global_hotkey: default_hotkey(),
            auto_read_clipboard_on_apply: false,
            create_backups_by_default: true,
            show_bottom_bar: true,
            ui_chrome: UiChromeConfig::default(),
            terminal_presets: default_terminal_command_presets(),
            update_channel: default_update_channel(),
            update_repo_url: default_update_repo_url(),
            recent_projects: Vec::new(),
            keymap: KeymapConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateOverlayDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_file_tree: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_preset: Option<SystemPromptMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_blocks: Option<Vec<PromptBlockItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_header_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_header_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_footer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chunk_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectOverlayDto {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respect_gitignore: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_symlinks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minify_code: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalize_line_endings: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_globs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_globs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<FileFiltersOverlayDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_filters: Option<FolderFiltersOverlayDto>,
    #[serde(default, skip_serializing)]
    pub explicitly_allowed_secrets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_file_threshold_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateOverlayDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser: Option<ParserConfigDto>,
}

impl Default for ProjectOverlayDto {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: None,
            base_preset: None,
            respect_gitignore: None,
            follow_symlinks: None,
            minify_code: None,
            normalize_line_endings: None,
            include_globs: None,
            exclude_globs: None,
            filters: None,
            folder_filters: None,
            explicitly_allowed_secrets: Vec::new(),
            max_depth: None,
            max_files: None,
            large_file_threshold_bytes: None,
            template: None,
            parser: None,
        }
    }
}
