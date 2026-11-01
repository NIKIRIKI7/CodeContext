//! Интеграционные тесты каскадного загрузчика (`Testing Diamond`) и `proptest`-фаззинг инвариантов.
//! Запуск: `cargo nextest run --release --package config --test cascade_and_invariants`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::wildcard_imports,
    clippy::indexing_slicing,
    clippy::assigning_clones,
    clippy::assert_is_empty
)]

use std::fs;

use config::{CURRENT_SCHEMA_VERSION, LayeredConfigLoader, ProjectOverlayDto, TemplateOverlayDto};
use core::packing::services::secret_guard::SecretGuard;
use core::preset::models::preset::{
    MAX_ALLOWED_MAX_DEPTH, MAX_ALLOWED_MAX_FILES, MAX_CONFIG_FILE_BYTES,
    MAX_LARGE_FILE_THRESHOLD_BYTES, MIN_ALLOWED_CHUNK_BYTES, MIN_ALLOWED_MAX_DEPTH,
    MIN_ALLOWED_MAX_FILES, MIN_LARGE_FILE_THRESHOLD_BYTES, PROJECT_CONFIG_FILE_NAME, Preset,
};
use core::preset::models::resolution::{ConfigOrigin, ConfigWarning, GlobalConfig};
use core::preset::ports::config_repo::ConfigRepositoryPort;
use core::preset::services::preset_service::PresetService;
use core::shared_kernel::fs::file_kind::FileKind;
use proptest::prelude::*;
use tempfile::TempDir;

// ============================================================================
// ЧАСТЬ 1: ИНТЕГРАЦИОННЫЕ ТЕСТЫ КАСКАДА (DEFAULTS -> GLOBAL -> LOCAL)
// ============================================================================

#[test]
fn cascade_1_cold_start_without_files_returns_builtin_defaults() {
    let temp = TempDir::new().unwrap();
    let global_path = temp.path().join("global/config.json");
    let project_root = temp.path().join("project");
    fs::create_dir_all(&project_root).unwrap();

    let loader = LayeredConfigLoader::with_global_config_path(global_path.clone());
    let resolved = loader.resolve_for_project(Some(&project_root));

    assert_eq!(resolved.origin, ConfigOrigin::BuiltInDefault);
    assert!(!resolved.from_untrusted_project_config());
    assert!(resolved.warnings.is_empty());
    assert_eq!(resolved.effective_preset, Preset::default_code_preset());
    assert!(!global_path.exists(), "Чтение не должно создавать мусорные файлы на диске");
}

#[test]
fn cascade_2_project_overlay_merges_partially_without_losing_global_settings() {
    let temp = TempDir::new().unwrap();
    let global_path = temp.path().join("global/config.json");
    let project_root = temp.path().join("project");
    fs::create_dir_all(&project_root).unwrap();

    let loader = LayeredConfigLoader::with_global_config_path(global_path);

    // Level 1: Глобальный конфиг разработчика с его персональными исключениями и настройками
    let mut global_preset = Preset::default_code_preset();
    global_preset.name = "MyGlobal".to_owned();
    global_preset.minify_code = true;
    global_preset.max_depth = 48;
    global_preset.exclude_globs = vec![".idea/**".to_owned(), "*.swp".to_owned()];
    global_preset.template.preamble = "=== GLOBAL PREAMBLE ===\n".to_owned();

    let global_cfg = GlobalConfig {
        active_preset_name: "MyGlobal".to_owned(),
        presets: vec![global_preset],
        ..GlobalConfig::default()
    };
    loader.save_global(&global_cfg).unwrap();

    // Level 2: Локальный `.ctxpack.json`, переопределяющий только `include_file_tree` и добавляющий `exclude_globs`
    let local_json = r#"{
        "schema_version": 1,
        "include_globs": ["crates/**"],
        "exclude_globs": ["legacy/**", "*.swp"],
        "template": {
            "include_file_tree": true
        }
    }"#;
    fs::write(project_root.join(PROJECT_CONFIG_FILE_NAME), local_json).unwrap();

    let resolved = loader.resolve_for_project(Some(&project_root));

    assert_eq!(resolved.origin, ConfigOrigin::LocalProjectOverride);
    assert!(resolved.from_untrusted_project_config());
    assert!(resolved.warnings.is_empty());

    let eff = &resolved.effective_preset;
    // Наследовано от Level 1 (Global):
    assert!(eff.minify_code);
    assert_eq!(eff.max_depth, 48);
    assert_eq!(eff.template.preamble, "=== GLOBAL PREAMBLE ===\n");
    assert_eq!(eff.template.file_header_prefix, "=== FILE: ");
    // Переопределено в Level 2 (.ctxpack.json):
    assert!(eff.template.include_file_tree);
    assert_eq!(eff.include_globs, vec!["crates/**".to_owned()]);
    // Объединено и дедуплицировано (Global U Local):
    assert_eq!(
        eff.exclude_globs,
        vec![".idea/**".to_owned(), "*.swp".to_owned(), "legacy/**".to_owned()]
    );
}

#[test]
fn cascade_3_scenario_a3_blocks_secret_unlock_in_local_project_config() {
    let temp = TempDir::new().unwrap();
    let global_path = temp.path().join("global/config.json");
    let project_root = temp.path().join("malicious_repo");
    fs::create_dir_all(&project_root).unwrap();

    let malicious_ctxpack = r#"{
        "schema_version": 1,
        "include_globs": [".env", "id_rsa", "src/**"],
        "explicitly_allowed_secrets": [".env", "id_rsa"]
    }"#;
    fs::write(project_root.join(PROJECT_CONFIG_FILE_NAME), malicious_ctxpack).unwrap();

    let loader = LayeredConfigLoader::with_global_config_path(global_path);
    let resolved = loader.resolve_for_project(Some(&project_root));

    assert!(resolved.from_untrusted_project_config());
    assert!(resolved.has_security_warnings());
    assert!(resolved.effective_preset.explicitly_allowed_secrets.is_empty());

    let (kind, reason, anomaly) = SecretGuard::classify_entry(".env", &resolved.effective_preset);
    assert_eq!(kind, FileKind::Secret);
    assert!(reason.is_some());
    assert!(anomaly.is_some());
}

#[test]
fn cascade_4_graceful_degradation_on_corrupted_or_oversized_configs() {
    let temp = TempDir::new().unwrap();
    let global_path = temp.path().join("global/config.json");
    let project_root = temp.path().join("project");
    fs::create_dir_all(&project_root).unwrap();

    let loader = LayeredConfigLoader::with_global_config_path(global_path);
    let mut global_cfg = GlobalConfig::default();
    global_cfg.presets[0].minify_code = true;
    loader.save_global(&global_cfg).unwrap();

    // 1. Записываем битый JSON с маркером git-конфликта в `.ctxpack.json`
    fs::write(project_root.join(PROJECT_CONFIG_FILE_NAME), "<<<<<<< HEAD\n{ broken json").unwrap();

    let resolved_broken = loader.resolve_for_project(Some(&project_root));
    assert_eq!(resolved_broken.origin, ConfigOrigin::GlobalUserConfig);
    assert!(resolved_broken.effective_preset.minify_code);
    assert!(
        resolved_broken
            .warnings
            .iter()
            .any(|w| matches!(w, ConfigWarning::CorruptedProjectConfig { .. }))
    );

    // 2. Записываем файл > 1 МБ в `.ctxpack.json`
    let max_bytes = usize::try_from(MAX_CONFIG_FILE_BYTES).unwrap();
    let oversized_payload = "x".repeat(max_bytes + 1024);
    fs::write(project_root.join(PROJECT_CONFIG_FILE_NAME), oversized_payload).unwrap();

    let resolved_oversized = loader.resolve_for_project(Some(&project_root));
    assert_eq!(resolved_oversized.origin, ConfigOrigin::GlobalUserConfig);
    assert!(
        resolved_oversized
            .warnings
            .iter()
            .any(|w| matches!(w, ConfigWarning::OversizedConfigFile { .. }))
    );
}

#[test]
fn cascade_5_import_export_and_migrating_legacy_v0_preset() {
    let temp = TempDir::new().unwrap();
    let global_path = temp.path().join("global/config.json");
    let export_path = temp.path().join("exported_preset.json");
    let legacy_path = temp.path().join("legacy_v0.json");

    let service = PresetService::new(LayeredConfigLoader::with_global_config_path(global_path));

    let mut custom = Preset::default_code_preset();
    custom.name = "TeamPreset".to_owned();
    custom.normalize_line_endings = true;
    custom.explicitly_allowed_secrets.push(".env.local".to_owned());

    service.export_preset_to_file(&custom, &export_path).unwrap();
    let (reimported, _, warnings) = service.import_and_register_preset(&export_path, true).unwrap();
    assert!(warnings.is_empty());
    assert_eq!(reimported.name, "TeamPreset");
    assert!(reimported.normalize_line_endings);
    assert!(reimported.explicitly_allowed_secrets.is_empty());

    // Проверяем импорт легаси-пресета v0 со старыми названиями полей
    fs::write(&legacy_path, r#"{"name": "OldV0", "use_gitignore": false, "minify": true}"#)
        .unwrap();
    let (migrated, global_after, _) =
        service.import_and_register_preset(&legacy_path, true).unwrap();
    assert_eq!(migrated.name, "OldV0");
    assert!(!migrated.respect_gitignore);
    assert!(migrated.minify_code);
    assert_eq!(global_after.active_preset_name, "OldV0");
}

// ============================================================================
// ЧАСТЬ 2: PROPERTY-BASED ФАЗЗИНГ ИНВАРИАНТОВ (PROPTEST)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_1_arbitrary_corrupted_json_never_panics_and_falls_back_cleanly(
        random_garbage in ".*"
    ) {
        let temp = TempDir::new().unwrap();
        let global_path = temp.path().join("config.json");
        let project_root = temp.path().join("proj");
        fs::create_dir_all(&project_root).unwrap();

        fs::write(&global_path, &random_garbage).unwrap();
        fs::write(project_root.join(PROJECT_CONFIG_FILE_NAME), &random_garbage).unwrap();

        let loader = LayeredConfigLoader::with_global_config_path(global_path);
        let resolved = loader.resolve_for_project(Some(&project_root));

        prop_assert!(resolved.effective_preset.max_depth >= MIN_ALLOWED_MAX_DEPTH);
        prop_assert!(resolved.effective_preset.max_depth <= MAX_ALLOWED_MAX_DEPTH);
    }

    #[test]
    fn prop_2_extreme_numbers_and_untrusted_secrets_are_always_clamped_and_stripped(
        raw_depth in any::<usize>(),
        raw_files in any::<usize>(),
        raw_threshold in any::<u64>(),
        raw_chunk in proptest::option::of(any::<usize>()),
        secret_paths in prop::collection::vec("[a-zA-Z0-9_./-]{1,20}", 0..5),
    ) {
        let temp = TempDir::new().unwrap();
        let global_path = temp.path().join("config.json");
        let project_root = temp.path().join("proj");
        fs::create_dir_all(&project_root).unwrap();

        let overlay = ProjectOverlayDto {
            schema_version: CURRENT_SCHEMA_VERSION,
            max_depth: Some(raw_depth),
            max_files: Some(raw_files),
            large_file_threshold_bytes: Some(raw_threshold),
            template: Some(TemplateOverlayDto {
                max_chunk_bytes: raw_chunk,
                ..TemplateOverlayDto::default()
            }),
            ..ProjectOverlayDto::default()
        };

        let mut json_val = serde_json::to_value(&overlay).unwrap();
        json_val.as_object_mut().unwrap().insert(
            "explicitly_allowed_secrets".to_owned(),
            serde_json::to_value(&secret_paths).unwrap(),
        );
        fs::write(
            project_root.join(PROJECT_CONFIG_FILE_NAME),
            serde_json::to_string(&json_val).unwrap(),
        )
        .unwrap();

        let loader = LayeredConfigLoader::with_global_config_path(global_path);
        let (eff, from_untrusted, warnings) = loader.resolve_headless_preset(&project_root);

        prop_assert!(from_untrusted);
        prop_assert!(eff.explicitly_allowed_secrets.is_empty());
        if !secret_paths.is_empty() {
            prop_assert!(warnings.iter().any(ConfigWarning::is_security_warning));
        }

        prop_assert!((MIN_ALLOWED_MAX_DEPTH..=MAX_ALLOWED_MAX_DEPTH).contains(&eff.max_depth));
        prop_assert!((MIN_ALLOWED_MAX_FILES..=MAX_ALLOWED_MAX_FILES).contains(&eff.max_files));
        prop_assert!(
            (MIN_LARGE_FILE_THRESHOLD_BYTES..=MAX_LARGE_FILE_THRESHOLD_BYTES)
                .contains(&eff.large_file_threshold_bytes)
        );
        if let Some(chunk_len) = eff.template.max_chunk_bytes {
            prop_assert!(chunk_len >= MIN_ALLOWED_CHUNK_BYTES);
        }
    }
}
