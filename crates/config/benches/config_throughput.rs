//! Бенчмарки пропускной способности каскадного загрузчика конфигурации (`divan`).
//! Запуск: `cargo bench --package config`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::assigning_clones,
    clippy::assert_is_empty
)]

use std::fs;
use std::path::PathBuf;

use config::LayeredConfigLoader;
use core::preset::models::preset::{MAX_CONFIG_FILE_BYTES, PROJECT_CONFIG_FILE_NAME, Preset};
use core::preset::models::resolution::GlobalConfig;
use core::preset::ports::config_repo::ConfigRepositoryPort;
use divan::{AllocProfiler, Bencher};
use tempfile::TempDir;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

struct ConfigBenchFixture {
    _temp: TempDir,
    loader: LayeredConfigLoader,
    project_with_local_cfg: PathBuf,
    project_without_local_cfg: PathBuf,
    project_with_legacy_v0_cfg: PathBuf,
    project_with_oversized_cfg: PathBuf,
}

fn setup_fixture() -> ConfigBenchFixture {
    let temp = TempDir::new().unwrap();
    let global_path = temp.path().join("global/config.json");
    let loader = LayeredConfigLoader::with_global_config_path(global_path);

    let mut presets = Vec::with_capacity(5);
    for i in 0..5 {
        let mut p = Preset::default_code_preset();
        p.name = if i == 0 { "Default".to_owned() } else { format!("Profile_{i}") };
        p.exclude_globs = vec!["*.bak".to_owned(), ".idea/**".to_owned()];
        presets.push(p);
    }
    let global_cfg = GlobalConfig {
        active_preset_name: "Default".to_owned(),
        presets,
        ..GlobalConfig::default()
    };
    loader.save_global(&global_cfg).unwrap();

    let project_with_local_cfg = temp.path().join("proj_with_local");
    fs::create_dir_all(&project_with_local_cfg).unwrap();
    fs::write(
        project_with_local_cfg.join(PROJECT_CONFIG_FILE_NAME),
        r#"{
            "schema_version": 1,
            "minify_code": true,
            "include_globs": ["crates/**"],
            "exclude_globs": ["legacy/**"],
            "template": { "include_file_tree": true }
        }"#,
    )
    .unwrap();

    let project_without_local_cfg = temp.path().join("proj_no_local");
    fs::create_dir_all(&project_without_local_cfg).unwrap();

    let project_with_legacy_v0_cfg = temp.path().join("proj_legacy_v0");
    fs::create_dir_all(&project_with_legacy_v0_cfg).unwrap();
    fs::write(
        project_with_legacy_v0_cfg.join(PROJECT_CONFIG_FILE_NAME),
        r#"{
            "use_gitignore": true,
            "minify": true,
            "max_file_size_bytes": 2097152
        }"#,
    )
    .unwrap();

    let project_with_oversized_cfg = temp.path().join("proj_oversized");
    fs::create_dir_all(&project_with_oversized_cfg).unwrap();
    let max_bytes = usize::try_from(MAX_CONFIG_FILE_BYTES).unwrap();
    let big_payload = "x".repeat(max_bytes + 65_536);
    fs::write(project_with_oversized_cfg.join(PROJECT_CONFIG_FILE_NAME), big_payload).unwrap();

    ConfigBenchFixture {
        _temp: temp,
        loader,
        project_with_local_cfg,
        project_without_local_cfg,
        project_with_legacy_v0_cfg,
        project_with_oversized_cfg,
    }
}

/// 1. Полное разрешение 3 уровней каскада с диска (`Defaults -> Global JSON -> .ctxpack.json`).
#[divan::bench(sample_count = 100)]
fn bench_1_headless_cascade_full_3_layers(bencher: Bencher) {
    let fixture = setup_fixture();
    bencher.bench_local(|| {
        let res = fixture
            .loader
            .resolve_headless_preset(divan::black_box(&fixture.project_with_local_cfg));
        assert!(res.1);
        divan::black_box(res)
    });
}

/// 2. Быстрый путь, когда в папке проекта нет `.ctxpack.json` (1 `stat` промах + `Global JSON`).
#[divan::bench(sample_count = 100)]
fn bench_2_headless_missing_local_config(bencher: Bencher) {
    let fixture = setup_fixture();
    bencher.bench_local(|| {
        let res = fixture
            .loader
            .resolve_headless_preset(divan::black_box(&fixture.project_without_local_cfg));
        assert!(!res.1);
        divan::black_box(res)
    });
}

/// 3. Сравнение прямой десериализации `v1` (`Zero-Overhead Schema Check`) против миграции `v0 -> v1`.
#[divan::bench(args = ["v1_fast_path", "v0_migration_fallback"], sample_count = 100)]
fn bench_3_schema_v1_vs_v0_migration(bencher: Bencher, mode: &str) {
    let fixture = setup_fixture();
    let target_dir = if mode == "v1_fast_path" {
        &fixture.project_with_local_cfg
    } else {
        &fixture.project_with_legacy_v0_cfg
    };

    bencher.bench_local(|| {
        let res = fixture.loader.resolve_headless_preset(divan::black_box(target_dir));
        assert!(res.0.minify_code);
        divan::black_box(res)
    });
}

/// 4. Мгновенное отсечение аномального `.ctxpack.json` (> 1 МБ) по `fs::metadata` без чтения в ОЗУ.
#[divan::bench(sample_count = 100)]
fn bench_4_oversized_config_guard_zero_read_alloc(bencher: Bencher) {
    let fixture = setup_fixture();
    bencher.bench_local(|| {
        let res = fixture
            .loader
            .resolve_headless_preset(divan::black_box(&fixture.project_with_oversized_cfg));
        assert!(!res.2.is_empty());
        divan::black_box(res)
    });
}
