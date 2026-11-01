use core::preset::models::preset::{
    MAX_ALLOWED_MAX_DEPTH, MAX_ALLOWED_MAX_FILES, MAX_LARGE_FILE_THRESHOLD_BYTES,
    MIN_ALLOWED_CHUNK_BYTES, MIN_ALLOWED_MAX_DEPTH, MIN_ALLOWED_MAX_FILES,
    MIN_LARGE_FILE_THRESHOLD_BYTES, Preset,
};
use core::preset::models::resolution::{ConfigWarning, DEFAULT_GLOBAL_HOTKEY, GlobalConfig};
use core::preset::models::template::{
    DEFAULT_DIVIDER_MARKER, DEFAULT_FILE_FOOTER, DEFAULT_FILE_HEADER_PREFIX,
    DEFAULT_FILE_HEADER_SUFFIX, DEFAULT_REPLACE_MARKER, DEFAULT_SEARCH_MARKER, OutputTemplate,
    ParserConfig, SearchReplaceMarkers,
};

use super::dtos::{
    CURRENT_SCHEMA_VERSION, GlobalConfigDto, OutputTemplateDto, ParserConfigDto, PresetDto,
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
    GlobalConfig {
        active_preset_name,
        presets,
        global_hotkey,
        auto_read_clipboard_on_apply: dto.auto_read_clipboard_on_apply,
        create_backups_by_default: dto.create_backups_by_default,
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
        explicitly_allowed_secrets: dto.explicitly_allowed_secrets,
        max_depth: dto.max_depth,
        max_files: dto.max_files,
        large_file_threshold_bytes: dto.large_file_threshold_bytes,
        template: OutputTemplate {
            include_file_tree: dto.template.include_file_tree,
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
        explicitly_allowed_secrets: Vec::new(),
        max_depth: preset.max_depth,
        max_files: preset.max_files,
        large_file_threshold_bytes: preset.large_file_threshold_bytes,
        template: OutputTemplateDto {
            include_file_tree: preset.template.include_file_tree,
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
        explicitly_allowed_secrets: Vec::new(),
        max_depth: Some(preset.max_depth),
        max_files: Some(preset.max_files),
        large_file_threshold_bytes: Some(preset.large_file_threshold_bytes),
        template: Some(TemplateOverlayDto {
            include_file_tree: Some(preset.template.include_file_tree),
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
    if !allow_secrets && !preset.explicitly_allowed_secrets.is_empty() {
        let blocked = std::mem::take(&mut preset.explicitly_allowed_secrets);
        warnings.push(ConfigWarning::UntrustedSecretUnlockIgnored { blocked_paths: blocked });
    }
    preset
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
