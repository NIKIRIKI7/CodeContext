use std::collections::BTreeMap;
use std::sync::OnceLock;

use core::preset::models::preset::{
    RegexTemplateItem, default_excluded_folder_names, default_regex_templates,
};
use core::preset::models::resolution::{
    DEFAULT_GLOBAL_HOTKEY, DEFAULT_UPDATE_CHANNEL, DEFAULT_UPDATE_REPO_URL, KeymapConfig,
};
use core::preset::models::template::{
    DEFAULT_DIVIDER_MARKER, DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX,
    DEFAULT_FILE_HEADER_SUFFIX, DEFAULT_REPLACE_MARKER, DEFAULT_SEARCH_MARKER, PromptBlockItem,
    SearchReplaceMarkers, default_prompt_blocks,
};
use serde::{Deserialize, Serialize};

pub const EMBEDDED_CONFIG_DEFAULTS_JSON: &str =
    include_str!("../../../../assets/config_defaults.json");
pub const EMBEDDED_FILTER_TEMPLATES_JSON: &str =
    include_str!("../../../../assets/filter_templates.json");
pub const EMBEDDED_PROMPT_TEMPLATES_JSON: &str =
    include_str!("../../../../assets/prompt_templates.json");
pub const EMBEDDED_SECURITY_SYNTAX_JSON: &str =
    include_str!("../../../../assets/security_and_syntax.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigLimitsTemplateDto {
    pub max_config_file_bytes: u64,
    pub default_max_depth: usize,
    pub min_allowed_max_depth: usize,
    pub max_allowed_max_depth: usize,
    pub default_max_files: usize,
    pub min_allowed_max_files: usize,
    pub max_allowed_max_files: usize,
    pub default_large_file_threshold_bytes: u64,
    pub min_large_file_threshold_bytes: u64,
    pub max_large_file_threshold_bytes: u64,
    pub min_allowed_chunk_bytes: usize,
    pub chunk_presets_bytes: Vec<usize>,
}

impl Default for ConfigLimitsTemplateDto {
    fn default() -> Self {
        Self {
            max_config_file_bytes: 1_048_576,
            default_max_depth: 32,
            min_allowed_max_depth: 1,
            max_allowed_max_depth: 128,
            default_max_files: 50_000,
            min_allowed_max_files: 1,
            max_allowed_max_files: 500_000,
            default_large_file_threshold_bytes: 1_048_576,
            min_large_file_threshold_bytes: 1_024,
            max_large_file_threshold_bytes: 104_857_600,
            min_allowed_chunk_bytes: 4_096,
            chunk_presets_bytes: vec![524_288, 1_048_576, 2_097_152],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigDefaultsBundleDto {
    pub schema_version: u32,
    pub active_preset_name: String,
    pub global_hotkey: String,
    pub hotkey_cycle_presets: Vec<String>,
    pub auto_read_clipboard_on_apply: bool,
    pub create_backups_by_default: bool,
    pub show_bottom_bar: bool,
    pub update_channel: String,
    pub update_repo_url: String,
    pub keymap: KeymapConfig,
    pub limits: ConfigLimitsTemplateDto,
}

impl Default for ConfigDefaultsBundleDto {
    fn default() -> Self {
        Self {
            schema_version: 2,
            active_preset_name: "Default".to_owned(),
            global_hotkey: DEFAULT_GLOBAL_HOTKEY.to_owned(),
            hotkey_cycle_presets: vec![
                "Ctrl+K".to_owned(),
                "Ctrl+Shift+C".to_owned(),
                "Ctrl+Alt+C".to_owned(),
            ],
            auto_read_clipboard_on_apply: false,
            create_backups_by_default: true,
            show_bottom_bar: true,
            update_channel: DEFAULT_UPDATE_CHANNEL.to_owned(),
            update_repo_url: DEFAULT_UPDATE_REPO_URL.to_owned(),
            keymap: KeymapConfig::default(),
            limits: ConfigLimitsTemplateDto::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterTemplatesBundleDto {
    pub default_excluded_folder_names: Vec<String>,
    pub quick_folder_chips: Vec<String>,
    pub extension_groups: BTreeMap<String, Vec<String>>,
    pub regex_templates: Vec<RegexTemplateItem>,
}

impl Default for FilterTemplatesBundleDto {
    fn default() -> Self {
        Self {
            default_excluded_folder_names: default_excluded_folder_names(),
            quick_folder_chips: vec![
                "target".to_owned(),
                "node_modules".to_owned(),
                "dist".to_owned(),
                "build".to_owned(),
                "tests".to_owned(),
                "benches".to_owned(),
                "docs".to_owned(),
                "assets".to_owned(),
                ".venv".to_owned(),
                "vendor".to_owned(),
            ],
            extension_groups: BTreeMap::new(),
            regex_templates: default_regex_templates(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputTemplateDefaultsDto {
    pub include_file_tree: bool,
    pub file_header_prefix: String,
    pub file_header_suffix: String,
    pub file_footer: String,
}

impl Default for OutputTemplateDefaultsDto {
    fn default() -> Self {
        Self {
            include_file_tree: false,
            file_header_prefix: DEFAULT_FILE_HEADER_PREFIX.to_owned(),
            file_header_suffix: DEFAULT_FILE_HEADER_SUFFIX.to_owned(),
            file_footer: DEFAULT_FILE_FOOTER.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParserConfigDefaultsDto {
    pub sync_with_output_template: bool,
    pub enable_markdown_fences: bool,
    pub enable_search_replace: bool,
    pub enable_unified_diff: bool,
    pub strip_project_root_prefix: bool,
    pub comment_path_prefixes: Vec<String>,
    pub search_replace_markers: SearchReplaceMarkers,
}

impl Default for ParserConfigDefaultsDto {
    fn default() -> Self {
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
            search_replace_markers: SearchReplaceMarkers {
                search_header: DEFAULT_SEARCH_MARKER.to_owned(),
                divider: DEFAULT_DIVIDER_MARKER.to_owned(),
                replace_footer: DEFAULT_REPLACE_MARKER.to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PromptTemplatesBundleDto {
    pub output_template: OutputTemplateDefaultsDto,
    pub parser_config: ParserConfigDefaultsDto,
    pub prompt_blocks: Vec<PromptBlockItem>,
}

impl Default for PromptTemplatesBundleDto {
    fn default() -> Self {
        Self {
            output_template: OutputTemplateDefaultsDto::default(),
            parser_config: ParserConfigDefaultsDto::default(),
            prompt_blocks: default_prompt_blocks(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SecretGuardRulesDto {
    pub sensitive_dirs: Vec<String>,
    pub safe_code_extensions: Vec<String>,
    pub exact_secret_filenames: Vec<String>,
    pub safe_dotenv_templates: Vec<String>,
    pub secret_extensions: Vec<String>,
    pub data_extensions_for_secret_substring: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityAndSyntaxBundleDto {
    pub secret_guard: SecretGuardRulesDto,
    pub binary_extensions: Vec<String>,
}

static CONFIG_DEFAULTS_CACHE: OnceLock<ConfigDefaultsBundleDto> = OnceLock::new();
static FILTER_TEMPLATES_CACHE: OnceLock<FilterTemplatesBundleDto> = OnceLock::new();
static PROMPT_TEMPLATES_CACHE: OnceLock<PromptTemplatesBundleDto> = OnceLock::new();
static SECURITY_SYNTAX_CACHE: OnceLock<SecurityAndSyntaxBundleDto> = OnceLock::new();

#[inline]
pub fn config_defaults_bundle() -> &'static ConfigDefaultsBundleDto {
    CONFIG_DEFAULTS_CACHE.get_or_init(|| {
        let external = std::fs::read_to_string("assets/config_defaults.json").ok();
        let raw = external.as_deref().unwrap_or(EMBEDDED_CONFIG_DEFAULTS_JSON);
        serde_json::from_str::<ConfigDefaultsBundleDto>(raw).unwrap_or_default()
    })
}

#[inline]
pub fn filter_templates_bundle() -> &'static FilterTemplatesBundleDto {
    FILTER_TEMPLATES_CACHE.get_or_init(|| {
        let external = std::fs::read_to_string("assets/filter_templates.json").ok();
        let raw = external.as_deref().unwrap_or(EMBEDDED_FILTER_TEMPLATES_JSON);
        serde_json::from_str::<FilterTemplatesBundleDto>(raw).unwrap_or_default()
    })
}

#[inline]
pub fn prompt_templates_bundle() -> &'static PromptTemplatesBundleDto {
    PROMPT_TEMPLATES_CACHE.get_or_init(|| {
        let external = std::fs::read_to_string("assets/prompt_templates.json").ok();
        let raw = external.as_deref().unwrap_or(EMBEDDED_PROMPT_TEMPLATES_JSON);
        serde_json::from_str::<PromptTemplatesBundleDto>(raw).unwrap_or_default()
    })
}

#[inline]
pub fn security_and_syntax_bundle() -> &'static SecurityAndSyntaxBundleDto {
    SECURITY_SYNTAX_CACHE.get_or_init(|| {
        let external = std::fs::read_to_string("assets/security_and_syntax.json").ok();
        let raw = external.as_deref().unwrap_or(EMBEDDED_SECURITY_SYNTAX_JSON);
        serde_json::from_str::<SecurityAndSyntaxBundleDto>(raw).unwrap_or_default()
    })
}
