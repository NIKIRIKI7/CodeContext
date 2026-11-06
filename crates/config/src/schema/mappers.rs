use std::collections::BTreeMap;

use core::preset::models::preset::{
    FileFilters, FolderFilters, FolderOverrideMode, MAX_ALLOWED_MAX_DEPTH, MAX_ALLOWED_MAX_FILES,
    MAX_LARGE_FILE_THRESHOLD_BYTES, MIN_ALLOWED_CHUNK_BYTES, MIN_ALLOWED_MAX_DEPTH,
    MIN_ALLOWED_MAX_FILES, MIN_LARGE_FILE_THRESHOLD_BYTES, Preset, default_regex_templates,
};
use core::preset::models::resolution::{
    ConfigWarning, DEFAULT_GLOBAL_HOTKEY, DEFAULT_UPDATE_CHANNEL, DEFAULT_UPDATE_REPO_URL,
    GlobalConfig, MAX_RECENT_PROJECTS, default_terminal_command_presets,
    normalize_recent_project_path, paths_are_equivalent,
};
use core::preset::models::template::{
    DEFAULT_DIVIDER_MARKER, DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX,
    DEFAULT_FILE_HEADER_SUFFIX, DEFAULT_REPLACE_MARKER, DEFAULT_SEARCH_MARKER, OutputTemplate,
    ParserConfig, SearchReplaceMarkers,
};

use super::dtos::{
    CURRENT_SCHEMA_VERSION, FileFiltersDto, FileFiltersOverlayDto, FolderFiltersDto,
    FolderFiltersOverlayDto, GlobalConfigDto, OutputTemplateDto, ParserConfigDto, PresetDto,
    ProjectOverlayDto, TemplateOverlayDto,
};

pub(crate) fn global_dto_to_domain(
    dto: GlobalConfigDto,
    warnings: &mut Vec<ConfigWarning>,
) -> GlobalConfig {
    let mut presets: Vec<Preset> =
        dto.presets.into_iter().map(|p| preset_dto_to_domain(p, false, warnings)).collect();
    if presets.is_empty() {
        presets.push(Preset::default_code_preset());
    }
    let mut active_preset_name = dto.active_preset_name.trim().to_owned();
    let active_exists = presets
        .iter()
        .any(|p| p.name == active_preset_name || p.name.eq_ignore_ascii_case(&active_preset_name));
    if !active_exists && let Some(first) = presets.first() {
        active_preset_name.clone_from(&first.name);
    }
    let global_hotkey = if dto.global_hotkey.trim().is_empty() {
        DEFAULT_GLOBAL_HOTKEY.to_owned()
    } else {
        dto.global_hotkey.trim().to_owned()
    };
    let update_channel = if dto.update_channel.trim().is_empty() {
        DEFAULT_UPDATE_CHANNEL.to_owned()
    } else {
        dto.update_channel.trim().to_ascii_lowercase()
    };
    let update_repo_url = if dto.update_repo_url.trim().is_empty() {
        DEFAULT_UPDATE_REPO_URL.to_owned()
    } else {
        dto.update_repo_url.trim().to_owned()
    };
    let mut recent_projects: Vec<String> =
        Vec::with_capacity(dto.recent_projects.len().min(MAX_RECENT_PROJECTS));
    for raw_path in dto.recent_projects {
        let normalized = normalize_recent_project_path(&raw_path);
        if !normalized.is_empty()
            && (normalized.contains('/') || normalized.contains('\\'))
            && !recent_projects
                .iter()
                .any(|existing: &String| paths_are_equivalent(existing, normalized))
        {
            recent_projects.push(normalized.to_owned());
            if recent_projects.len() >= MAX_RECENT_PROJECTS {
                break;
            }
        }
    }
    let mut ui_chrome = dto.ui_chrome;
    if !dto.show_bottom_bar {
        ui_chrome.show_bottom_bar = false;
    }
    let terminal_presets = if dto.terminal_presets.is_empty() {
        default_terminal_command_presets()
    } else {
        dto.terminal_presets
    };
    GlobalConfig {
        active_preset_name,
        presets,
        global_hotkey,
        auto_read_clipboard_on_apply: dto.auto_read_clipboard_on_apply,
        create_backups_by_default: dto.create_backups_by_default,
        show_bottom_bar: ui_chrome.show_bottom_bar,
        edition: dto.edition,
        features: dto.features,
        ui_chrome,
        terminal_presets,
        update_channel,
        update_repo_url,
        recent_projects,
        enabled_plugins: dto.enabled_plugins,
        keymap: dto.keymap,
    }
}

pub(crate) fn global_domain_to_dto(domain: &GlobalConfig) -> GlobalConfigDto {
    GlobalConfigDto {
        schema_version: CURRENT_SCHEMA_VERSION,
        active_preset_name: domain.active_preset_name.clone(),
        presets: domain.presets.iter().map(preset_domain_to_dto).collect(),
        global_hotkey: domain.global_hotkey.clone(),
        auto_read_clipboard_on_apply: domain.auto_read_clipboard_on_apply,
        create_backups_by_default: domain.create_backups_by_default,
        show_bottom_bar: domain.ui_chrome.show_bottom_bar,
        edition: domain.edition,
        features: domain.features.clone(),
        ui_chrome: domain.ui_chrome.clone(),
        terminal_presets: domain.terminal_presets.clone(),
        update_channel: domain.update_channel.clone(),
        update_repo_url: domain.update_repo_url.clone(),
        recent_projects: domain.recent_projects.clone(),
        enabled_plugins: domain.enabled_plugins.clone(),
        keymap: domain.keymap.clone(),
    }
}

pub(crate) fn parser_dto_to_domain(dto: ParserConfigDto) -> ParserConfig {
    let search_header = if dto.search_header.trim().is_empty() {
        DEFAULT_SEARCH_MARKER.to_owned()
    } else {
        dto.search_header
    };
    let divider =
        if dto.divider.trim().is_empty() { DEFAULT_DIVIDER_MARKER.to_owned() } else { dto.divider };
    let replace_footer = if dto.replace_footer.trim().is_empty() {
        DEFAULT_REPLACE_MARKER.to_owned()
    } else {
        dto.replace_footer
    };
    ParserConfig {
        sync_with_output_template: dto.sync_with_output_template,
        enable_markdown_fences: dto.enable_markdown_fences,
        enable_search_replace: dto.enable_search_replace,
        enable_unified_diff: dto.enable_unified_diff,
        strip_project_root_prefix: dto.strip_project_root_prefix,
        comment_path_prefixes: dto.comment_path_prefixes,
        search_replace_markers: SearchReplaceMarkers { search_header, divider, replace_footer },
    }
}

pub(crate) fn parser_domain_to_dto(domain: &ParserConfig) -> ParserConfigDto {
    ParserConfigDto {
        sync_with_output_template: domain.sync_with_output_template,
        enable_markdown_fences: domain.enable_markdown_fences,
        enable_search_replace: domain.enable_search_replace,
        enable_unified_diff: domain.enable_unified_diff,
        strip_project_root_prefix: domain.strip_project_root_prefix,
        comment_path_prefixes: domain.comment_path_prefixes.clone(),
        search_header: domain.search_replace_markers.search_header.clone(),
        divider: domain.search_replace_markers.divider.clone(),
        replace_footer: domain.search_replace_markers.replace_footer.clone(),
    }
}

pub(crate) fn preset_dto_to_domain(
    dto: PresetDto,
    allow_secrets: bool,
    warnings: &mut Vec<ConfigWarning>,
) -> Preset {
    let raw = Preset {
        name: dto.name,
        respect_gitignore: dto.respect_gitignore,
        follow_symlinks: dto.follow_symlinks,
        minify_code: dto.minify_code,
        normalize_line_endings: dto.normalize_line_endings,
        include_globs: dto.include_globs,
        exclude_globs: dto.exclude_globs,
        filters: FileFilters {
            include_extensions: dto.filters.include_extensions,
            exclude_extensions: dto.filters.exclude_extensions,
            include_globs: dto.filters.include_globs,
            exclude_globs: dto.filters.exclude_globs,
            include_regex: dto.filters.include_regex,
            exclude_regex: dto.filters.exclude_regex,
            regex_case_sensitive: dto.filters.regex_case_sensitive,
            regex_templates: if dto.filters.regex_templates.is_empty() {
                default_regex_templates()
            } else {
                dto.filters.regex_templates
            },
        },
        folder_filters: FolderFilters {
            exclude_names: dto.folder_filters.exclude_names,
            exclude_globs: dto.folder_filters.exclude_globs,
            only_folders: dto.folder_filters.only_folders,
            include_regex_folders: dto.folder_filters.include_regex_folders,
            exclude_regex_folders: dto.folder_filters.exclude_regex_folders,
            regex_case_sensitive: dto.folder_filters.regex_case_sensitive,
            folder_overrides: dto.folder_filters.folder_overrides,
            show_excluded_in_ui: dto.folder_filters.show_excluded_in_ui,
        },
        explicitly_allowed_secrets: dto.explicitly_allowed_secrets,
        max_depth: dto.max_depth,
        max_files: dto.max_files,
        large_file_threshold_bytes: dto.large_file_threshold_bytes,
        template: OutputTemplate {
            include_file_tree: dto.template.include_file_tree,
            system_prompt_preset: dto.template.system_prompt_preset,
            prompt_blocks: if dto.template.prompt_blocks.is_empty() {
                core::preset::models::template::default_prompt_blocks()
            } else {
                dto.template.prompt_blocks
            },
            preamble: dto.template.preamble,
            file_header_prefix: dto.template.file_header_prefix,
            file_header_suffix: dto.template.file_header_suffix,
            file_footer: dto.template.file_footer,
            postamble: dto.template.postamble,
            max_chunk_bytes: dto.template.max_chunk_bytes,
        },
        parser: parser_dto_to_domain(dto.parser),
    };
    sanitize_and_clamp_preset(raw, allow_secrets, warnings)
}

pub(crate) fn preset_domain_to_dto(preset: &Preset) -> PresetDto {
    PresetDto {
        name: preset.name.clone(),
        respect_gitignore: preset.respect_gitignore,
        follow_symlinks: preset.follow_symlinks,
        minify_code: preset.minify_code,
        normalize_line_endings: preset.normalize_line_endings,
        include_globs: preset.include_globs.clone(),
        exclude_globs: preset.exclude_globs.clone(),
        filters: FileFiltersDto {
            include_extensions: preset.filters.include_extensions.clone(),
            exclude_extensions: preset.filters.exclude_extensions.clone(),
            include_globs: preset.filters.include_globs.clone(),
            exclude_globs: preset.filters.exclude_globs.clone(),
            include_regex: preset.filters.include_regex.clone(),
            exclude_regex: preset.filters.exclude_regex.clone(),
            regex_case_sensitive: preset.filters.regex_case_sensitive,
            regex_templates: preset.filters.regex_templates.clone(),
        },
        folder_filters: FolderFiltersDto {
            exclude_names: preset.folder_filters.exclude_names.clone(),
            exclude_globs: preset.folder_filters.exclude_globs.clone(),
            only_folders: preset.folder_filters.only_folders.clone(),
            include_regex_folders: preset.folder_filters.include_regex_folders.clone(),
            exclude_regex_folders: preset.folder_filters.exclude_regex_folders.clone(),
            regex_case_sensitive: preset.folder_filters.regex_case_sensitive,
            folder_overrides: preset.folder_filters.folder_overrides.clone(),
            show_excluded_in_ui: preset.folder_filters.show_excluded_in_ui,
        },
        explicitly_allowed_secrets: Vec::new(),
        max_depth: preset.max_depth,
        max_files: preset.max_files,
        large_file_threshold_bytes: preset.large_file_threshold_bytes,
        template: OutputTemplateDto {
            include_file_tree: preset.template.include_file_tree,
            system_prompt_preset: preset.template.system_prompt_preset,
            prompt_blocks: preset.template.prompt_blocks.clone(),
            preamble: preset.template.preamble.clone(),
            file_header_prefix: preset.template.file_header_prefix.clone(),
            file_header_suffix: preset.template.file_header_suffix.clone(),
            file_footer: preset.template.file_footer.clone(),
            postamble: preset.template.postamble.clone(),
            max_chunk_bytes: preset.template.max_chunk_bytes,
        },
        parser: parser_domain_to_dto(&preset.parser),
    }
}

pub(crate) fn preset_domain_to_project_overlay_dto(preset: &Preset) -> ProjectOverlayDto {
    let parser_dto = if preset.parser == ParserConfig::canonical() {
        None
    } else {
        Some(parser_domain_to_dto(&preset.parser))
    };
    let filters_ov = if preset.filters == FileFilters::default() {
        None
    } else {
        Some(FileFiltersOverlayDto {
            include_extensions: Some(preset.filters.include_extensions.clone()),
            exclude_extensions: Some(preset.filters.exclude_extensions.clone()),
            include_globs: Some(preset.filters.include_globs.clone()),
            exclude_globs: Some(preset.filters.exclude_globs.clone()),
            include_regex: preset.filters.include_regex.clone(),
            exclude_regex: preset.filters.exclude_regex.clone(),
            regex_case_sensitive: Some(preset.filters.regex_case_sensitive),
            regex_templates: (preset.filters.regex_templates != default_regex_templates())
                .then(|| preset.filters.regex_templates.clone()),
        })
    };
    let folder_filters_ov = if preset.folder_filters == FolderFilters::default() {
        None
    } else {
        Some(FolderFiltersOverlayDto {
            exclude_names: Some(preset.folder_filters.exclude_names.clone()),
            exclude_globs: Some(preset.folder_filters.exclude_globs.clone()),
            only_folders: Some(preset.folder_filters.only_folders.clone()),
            include_regex_folders: preset.folder_filters.include_regex_folders.clone(),
            exclude_regex_folders: preset.folder_filters.exclude_regex_folders.clone(),
            regex_case_sensitive: Some(preset.folder_filters.regex_case_sensitive),
            folder_overrides: Some(preset.folder_filters.folder_overrides.clone()),
            show_excluded_in_ui: Some(preset.folder_filters.show_excluded_in_ui),
        })
    };
    ProjectOverlayDto {
        schema_version: CURRENT_SCHEMA_VERSION,
        name: Some(preset.name.clone()),
        base_preset: None,
        respect_gitignore: Some(preset.respect_gitignore),
        follow_symlinks: Some(preset.follow_symlinks),
        minify_code: Some(preset.minify_code),
        normalize_line_endings: Some(preset.normalize_line_endings),
        include_globs: Some(preset.include_globs.clone()),
        exclude_globs: Some(preset.exclude_globs.clone()),
        filters: filters_ov,
        folder_filters: folder_filters_ov,
        explicitly_allowed_secrets: Vec::new(),
        max_depth: Some(preset.max_depth),
        max_files: Some(preset.max_files),
        large_file_threshold_bytes: Some(preset.large_file_threshold_bytes),
        template: Some(TemplateOverlayDto {
            include_file_tree: Some(preset.template.include_file_tree),
            system_prompt_preset: Some(preset.template.system_prompt_preset),
            prompt_blocks: Some(preset.template.prompt_blocks.clone()),
            preamble: Some(preset.template.preamble.clone()),
            file_header_prefix: Some(preset.template.file_header_prefix.clone()),
            file_header_suffix: Some(preset.template.file_header_suffix.clone()),
            file_footer: Some(preset.template.file_footer.clone()),
            postamble: Some(preset.template.postamble.clone()),
            max_chunk_bytes: preset.template.max_chunk_bytes,
        }),
        parser: parser_dto,
    }
}

pub(crate) fn apply_project_overlay(
    base: &Preset,
    overlay: ProjectOverlayDto,
    warnings: &mut Vec<ConfigWarning>,
) -> Preset {
    if !overlay.explicitly_allowed_secrets.is_empty() {
        warnings.push(ConfigWarning::UntrustedSecretUnlockIgnored {
            blocked_paths: overlay.explicitly_allowed_secrets,
        });
    }
    let mut merged = base.clone().sanitized_from_untrusted_project_config();
    if let Some(name) = overlay.name {
        merged.name = name;
    }
    if let Some(respect_gitignore) = overlay.respect_gitignore {
        merged.respect_gitignore = respect_gitignore;
    }
    if let Some(follow_symlinks) = overlay.follow_symlinks {
        merged.follow_symlinks = follow_symlinks;
    }
    if let Some(minify_code) = overlay.minify_code {
        merged.minify_code = minify_code;
    }
    if let Some(normalize_line_endings) = overlay.normalize_line_endings {
        merged.normalize_line_endings = normalize_line_endings;
    }
    if let Some(include_globs) = overlay.include_globs {
        merged.include_globs = include_globs;
    }
    if let Some(local_excludes) = overlay.exclude_globs {
        merged.exclude_globs.extend(local_excludes);
    }
    if let Some(f_ov) = overlay.filters {
        apply_file_filters_overlay(&mut merged.filters, f_ov);
    }
    if let Some(ff_ov) = overlay.folder_filters {
        apply_folder_filters_overlay(&mut merged.folder_filters, ff_ov);
    }
    if let Some(max_depth) = overlay.max_depth {
        merged.max_depth = max_depth;
    }
    if let Some(max_files) = overlay.max_files {
        merged.max_files = max_files;
    }
    if let Some(large_file_threshold_bytes) = overlay.large_file_threshold_bytes {
        merged.large_file_threshold_bytes = large_file_threshold_bytes;
    }
    if let Some(tpl_ov) = overlay.template {
        apply_template_overlay(&mut merged.template, tpl_ov);
    }
    if let Some(parser_ov) = overlay.parser {
        merged.parser = parser_dto_to_domain(parser_ov);
    }
    sanitize_and_clamp_preset(merged, false, warnings)
}

fn apply_template_overlay(target: &mut OutputTemplate, overlay: TemplateOverlayDto) {
    if let Some(include_file_tree) = overlay.include_file_tree {
        target.include_file_tree = include_file_tree;
    }
    if let Some(system_prompt_preset) = overlay.system_prompt_preset {
        target.system_prompt_preset = system_prompt_preset;
    }
    if let Some(prompt_blocks) = overlay.prompt_blocks
        && !prompt_blocks.is_empty()
    {
        target.prompt_blocks = prompt_blocks;
    }
    if let Some(preamble) = overlay.preamble {
        target.preamble = preamble;
    }
    if let Some(file_header_prefix) = overlay.file_header_prefix {
        target.file_header_prefix = file_header_prefix;
    }
    if let Some(file_header_suffix) = overlay.file_header_suffix {
        target.file_header_suffix = file_header_suffix;
    }
    if let Some(file_footer) = overlay.file_footer {
        target.file_footer = file_footer;
    }
    if let Some(postamble) = overlay.postamble {
        target.postamble = postamble;
    }
    if let Some(chunk_bytes) = overlay.max_chunk_bytes {
        target.max_chunk_bytes = if chunk_bytes == 0 { None } else { Some(chunk_bytes) };
    }
}

pub(crate) fn sanitize_and_clamp_preset(
    mut preset: Preset,
    allow_secrets: bool,
    warnings: &mut Vec<ConfigWarning>,
) -> Preset {
    let trimmed_len = preset.name.trim().len();
    if trimmed_len == 0 {
        "Default".clone_into(&mut preset.name);
    } else if trimmed_len != preset.name.len() {
        let clean_name = preset.name.trim().to_owned();
        preset.name = clean_name;
    }
    let clamped_depth = preset.max_depth.clamp(MIN_ALLOWED_MAX_DEPTH, MAX_ALLOWED_MAX_DEPTH);
    if clamped_depth != preset.max_depth {
        warnings.push(ConfigWarning::ParameterClamped {
            field: "max_depth".to_owned(),
            original: preset.max_depth.to_string(),
            clamped_to: clamped_depth.to_string(),
        });
        preset.max_depth = clamped_depth;
    }
    let clamped_files = preset.max_files.clamp(MIN_ALLOWED_MAX_FILES, MAX_ALLOWED_MAX_FILES);
    if clamped_files != preset.max_files {
        warnings.push(ConfigWarning::ParameterClamped {
            field: "max_files".to_owned(),
            original: preset.max_files.to_string(),
            clamped_to: clamped_files.to_string(),
        });
        preset.max_files = clamped_files;
    }
    let clamped_threshold = preset
        .large_file_threshold_bytes
        .clamp(MIN_LARGE_FILE_THRESHOLD_BYTES, MAX_LARGE_FILE_THRESHOLD_BYTES);
    if clamped_threshold != preset.large_file_threshold_bytes {
        warnings.push(ConfigWarning::ParameterClamped {
            field: "large_file_threshold_bytes".to_owned(),
            original: preset.large_file_threshold_bytes.to_string(),
            clamped_to: clamped_threshold.to_string(),
        });
        preset.large_file_threshold_bytes = clamped_threshold;
    }
    sanitize_template(&mut preset.template, warnings);
    normalize_glob_list(&mut preset.include_globs);
    normalize_glob_list(&mut preset.exclude_globs);
    normalize_extensions_list(&mut preset.filters.include_extensions);
    normalize_extensions_list(&mut preset.filters.exclude_extensions);
    normalize_glob_list(&mut preset.filters.include_globs);
    normalize_glob_list(&mut preset.filters.exclude_globs);
    normalize_optional_regex(&mut preset.filters.include_regex);
    normalize_optional_regex(&mut preset.filters.exclude_regex);
    normalize_folder_names_list(&mut preset.folder_filters.exclude_names);
    normalize_glob_list(&mut preset.folder_filters.exclude_globs);
    normalize_folder_paths_list(&mut preset.folder_filters.only_folders);
    normalize_optional_regex(&mut preset.folder_filters.include_regex_folders);
    normalize_optional_regex(&mut preset.folder_filters.exclude_regex_folders);
    normalize_folder_overrides(&mut preset.folder_filters.folder_overrides);
    if !allow_secrets && !preset.explicitly_allowed_secrets.is_empty() {
        let blocked = std::mem::take(&mut preset.explicitly_allowed_secrets);
        warnings.push(ConfigWarning::UntrustedSecretUnlockIgnored { blocked_paths: blocked });
    }
    preset
}

fn apply_file_filters_overlay(target: &mut FileFilters, overlay: FileFiltersOverlayDto) {
    if let Some(inc_ext) = overlay.include_extensions {
        target.include_extensions = inc_ext;
    }
    if let Some(exc_ext) = overlay.exclude_extensions {
        target.exclude_extensions.extend(exc_ext);
    }
    if let Some(inc_globs) = overlay.include_globs {
        target.include_globs = inc_globs;
    }
    if let Some(exc_globs) = overlay.exclude_globs {
        target.exclude_globs.extend(exc_globs);
    }
    if let Some(inc_re) = overlay.include_regex {
        target.include_regex = (!inc_re.trim().is_empty()).then_some(inc_re);
    }
    if let Some(exc_re) = overlay.exclude_regex {
        target.exclude_regex = (!exc_re.trim().is_empty()).then_some(exc_re);
    }
    if let Some(cs) = overlay.regex_case_sensitive {
        target.regex_case_sensitive = cs;
    }
    if let Some(templates) = overlay.regex_templates
        && !templates.is_empty()
    {
        target.regex_templates = templates;
    }
}

fn apply_folder_filters_overlay(target: &mut FolderFilters, overlay: FolderFiltersOverlayDto) {
    if let Some(exc_names) = overlay.exclude_names {
        target.exclude_names = exc_names;
    }
    if let Some(exc_globs) = overlay.exclude_globs {
        target.exclude_globs.extend(exc_globs);
    }
    if let Some(only) = overlay.only_folders {
        target.only_folders = only;
    }
    if let Some(inc_re) = overlay.include_regex_folders {
        target.include_regex_folders = (!inc_re.trim().is_empty()).then_some(inc_re);
    }
    if let Some(exc_re) = overlay.exclude_regex_folders {
        target.exclude_regex_folders = (!exc_re.trim().is_empty()).then_some(exc_re);
    }
    if let Some(cs) = overlay.regex_case_sensitive {
        target.regex_case_sensitive = cs;
    }
    if let Some(overrides) = overlay.folder_overrides {
        target.folder_overrides.extend(overrides);
    }
    if let Some(show_ex) = overlay.show_excluded_in_ui {
        target.show_excluded_in_ui = show_ex;
    }
}

fn sanitize_template(template: &mut OutputTemplate, warnings: &mut Vec<ConfigWarning>) {
    if template.file_header_prefix.trim().is_empty() {
        DEFAULT_FILE_HEADER_PREFIX.clone_into(&mut template.file_header_prefix);
    }
    if template.file_header_suffix.trim().is_empty() {
        DEFAULT_FILE_HEADER_SUFFIX.clone_into(&mut template.file_header_suffix);
    }
    if template.file_footer.trim().is_empty() {
        DEFAULT_FILE_FOOTER.clone_into(&mut template.file_footer);
    }
    if let Some(chunk_bytes) = template.max_chunk_bytes {
        if chunk_bytes == 0 {
            template.max_chunk_bytes = None;
        } else if chunk_bytes < MIN_ALLOWED_CHUNK_BYTES {
            warnings.push(ConfigWarning::ParameterClamped {
                field: "template.max_chunk_bytes".to_owned(),
                original: chunk_bytes.to_string(),
                clamped_to: MIN_ALLOWED_CHUNK_BYTES.to_string(),
            });
            template.max_chunk_bytes = Some(MIN_ALLOWED_CHUNK_BYTES);
        }
    }
}

fn normalize_glob_list(globs: &mut Vec<String>) {
    let mut clean: Vec<String> = Vec::with_capacity(globs.len());
    for item in globs.drain(..) {
        let trimmed = item.trim();
        if !trimmed.is_empty() && !clean.iter().any(|existing| existing == trimmed) {
            clean.push(trimmed.to_owned());
        }
    }
    *globs = clean;
}

fn normalize_extensions_list(exts: &mut Vec<String>) {
    let mut clean: Vec<String> = Vec::with_capacity(exts.len());
    for item in exts.drain(..) {
        let normalized = item.trim().trim_start_matches('.').to_ascii_lowercase();
        if !normalized.is_empty() && !clean.iter().any(|existing| existing == &normalized) {
            clean.push(normalized);
        }
    }
    *exts = clean;
}

fn normalize_folder_names_list(names: &mut Vec<String>) {
    let mut clean: Vec<String> = Vec::with_capacity(names.len());
    for item in names.drain(..) {
        let trimmed = item.trim().trim_matches(['/', '\\']);
        if !trimmed.is_empty() && !clean.iter().any(|existing| existing == trimmed) {
            clean.push(trimmed.to_owned());
        }
    }
    *names = clean;
}

fn normalize_folder_paths_list(paths: &mut Vec<String>) {
    let mut clean: Vec<String> = Vec::with_capacity(paths.len());
    for item in paths.drain(..) {
        let norm = normalize_rel_folder_pattern(&item);
        if !norm.is_empty() && !clean.iter().any(|existing| existing == &norm) {
            clean.push(norm);
        }
    }
    *paths = clean;
}

fn normalize_folder_overrides(overrides: &mut BTreeMap<String, FolderOverrideMode>) {
    let old = std::mem::take(overrides);
    for (raw_key, mode) in old {
        let clean_key = normalize_rel_folder_pattern(&raw_key);
        if !clean_key.is_empty() {
            overrides.insert(clean_key, mode);
        }
    }
}

fn normalize_rel_folder_pattern(raw: &str) -> String {
    let replaced = raw.trim().replace('\\', "/");
    let mut s = replaced.as_str();
    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped;
    }
    s = s.trim_matches('/');
    s.to_owned()
}

fn normalize_optional_regex(opt: &mut Option<String>) {
    if let Some(val) = opt {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            *opt = None;
        } else if trimmed.len() != val.len() {
            *opt = Some(trimmed.to_owned());
        }
    }
}
