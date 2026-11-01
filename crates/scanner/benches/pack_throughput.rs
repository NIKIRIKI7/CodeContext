//! Комплексные бенчмарки пропускной способности (`divan`).
//! Запуск: `cargo bench --package scanner`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value
)]

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use core::packing::aggregates::file_entry::FileEntry;
use core::packing::aggregates::project::Project;
use core::packing::services::chunker::Chunker;
use core::packing::services::secret_guard::SecretGuard;
use core::preset::models::preset::Preset;
use core::preset::models::template::OutputTemplate;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::shared_kernel::fs::file_kind::FileKind;
use divan::counter::{BytesCount, ItemsCount};
use divan::{AllocProfiler, Bencher};
use scanner::IgnoreFileScanner;
use tempfile::TempDir;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    print_environment_banner();
    divan::main();
}

fn print_environment_banner() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let logical_cores = std::thread::available_parallelism().map_or(1, usize::from);
    let rayon_threads = rayon::current_num_threads();
    let temp_dir = std::env::temp_dir();

    let simd_features = [
        #[cfg(target_feature = "avx2")]
        "AVX2",
        #[cfg(target_feature = "bmi2")]
        "BMI2",
        #[cfg(target_feature = "fma")]
        "FMA",
        #[cfg(target_feature = "neon")]
        "NEON",
    ];

    println!("==========================================================================");
    println!(" CodeContext Benchmark Environment");
    println!(" OS / Arch       : {os} / {arch}");
    println!(" CPU / Rayon     : {logical_cores} logical cores / {rayon_threads} rayon threads");
    println!(" Compile SIMD    : {simd_features:?}");
    println!(" Temp FS Path    : {}", temp_dir.display());
    println!("==========================================================================");
}

struct SyntheticRepo {
    _temp_dir: TempDir,
    root: PathBuf,
    expected_code_files: usize,
    ignored_node_modules_files: usize,
}

fn generate_fs_repo(
    code_files: usize,
    ignored_files: usize,
    tree_depth: usize,
    bytes_per_file: usize,
) -> SyntheticRepo {
    let temp_dir = TempDir::new().expect("Не удалось создать TempDir");
    let root = temp_dir.path().to_path_buf();

    fs::write(root.join(".ignore"), "node_modules/\ntarget/\n*.log\n").unwrap();
    fs::write(root.join(".gitignore"), "node_modules/\ntarget/\n*.log\n").unwrap();

    let line = "pub fn compute(x: u64) -> u64 { x.wrapping_mul(6364136223846793005) }\n";
    let payload = line.repeat((bytes_per_file / line.len()).max(1));

    for i in 0..ignored_files {
        let pkg_dir = root.join(format!("node_modules/pkg_{}/dist", i % 50));
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join(format!("bundle_{i}.js")), &payload).unwrap();
    }

    let depth = tree_depth.max(1);
    for i in 0..code_files {
        let mut dir = root.join("src");
        for level in 0..depth {
            dir = dir.join(format!("lvl_{level}_mod_{}", (i + level) % 8));
        }
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("file_{i}.rs")), &payload).unwrap();
    }

    SyntheticRepo {
        _temp_dir: temp_dir,
        root,
        expected_code_files: code_files,
        ignored_node_modules_files: ignored_files,
    }
}

fn generate_in_memory_files(
    file_count: usize,
    total_bytes: usize,
    use_crlf: bool,
) -> Vec<(String, String)> {
    let bytes_per_file = (total_bytes / file_count).max(64);
    let line = if use_crlf {
        "pub fn fast_hash(v: u64) -> u64 {\r\n    v.rotate_left(13) ^ 0x517cc1b727220a95\r\n}\r\n"
    } else {
        "pub fn fast_hash(v: u64) -> u64 {\n    v.rotate_left(13) ^ 0x517cc1b727220a95\n}\n"
    };
    let chunk = line.repeat((bytes_per_file / line.len()).max(1));

    (0..file_count)
        .map(|i| (format!("crates/domain_{}/src/entity_{i}.rs", i % 32), chunk.clone()))
        .collect()
}

#[divan::bench(args = [true, false], sample_count = 40)]
fn bench_1a_walk_gitignore_vs_no_gitignore(bencher: Bencher, respect_gitignore: bool) {
    let repo = generate_fs_repo(1_000, 4_000, 3, 1024);
    let scanner = IgnoreFileScanner::new();
    let cancel_token = CancelToken::new();

    bencher.bench_local(|| {
        let index = scanner.scan_project_index(&repo.root, respect_gitignore, 32, &cancel_token);
        if respect_gitignore {
            assert_eq!(index.len(), repo.expected_code_files);
        } else {
            assert_eq!(index.len(), repo.expected_code_files + repo.ignored_node_modules_files);
        }
        divan::black_box(index)
    });
}

#[divan::bench(args = [2, 12], sample_count = 40)]
fn bench_1b_walk_shallow_vs_deep_tree(bencher: Bencher, tree_depth: usize) {
    let repo = generate_fs_repo(2_000, 500, tree_depth, 512);
    let scanner = IgnoreFileScanner::new();
    let cancel_token = CancelToken::new();

    bencher.counter(ItemsCount::new(repo.expected_code_files)).bench_local(|| {
        let index = scanner.scan_project_index(&repo.root, true, 32, &cancel_token);
        assert_eq!(index.len(), repo.expected_code_files);
        divan::black_box(index)
    });
}

#[divan::bench(args = [false, true], sample_count = 30)]
fn bench_1c_real_pipeline_end_to_end_4mb(bencher: Bencher, minify: bool) {
    let repo = generate_fs_repo(1_000, 2_000, 4, 4096);
    let scanner = IgnoreFileScanner::new();
    let cancel_token = CancelToken::new();

    bencher.counter(BytesCount::new(1_000_usize * 4096)).bench_local(|| {
        let packed =
            scanner.pack_project_end_to_end(&repo.root, &["*.log"], minify, &cancel_token).unwrap();
        assert!(packed.len() > 3_500_000);
        divan::black_box(packed)
    });
}

#[divan::bench(args = [4096, 8192, 65_536, 1_048_576], sample_count = 100)]
fn bench_2a_simd_detector_pure_text(bencher: Bencher, size: usize) {
    let buffer = vec![b'x'; size];

    bencher.counter(BytesCount::new(size)).bench_local(|| {
        let is_bin = IgnoreFileScanner::is_binary_full_slice(divan::black_box(&buffer));
        assert!(!is_bin);
        divan::black_box(is_bin)
    });
}

#[divan::bench(args = [4096, 65_536, 1_048_576], sample_count = 100)]
fn bench_2b_simd_detector_binary_early_nul(bencher: Bencher, size: usize) {
    let mut buffer = vec![b'x'; size];
    buffer[4] = 0;

    bencher.counter(BytesCount::new(size)).bench_local(|| {
        let is_bin = IgnoreFileScanner::is_binary_full_slice(divan::black_box(&buffer));
        assert!(is_bin);
        divan::black_box(is_bin)
    });
}

#[divan::bench(args = [1, 4, 16, 64], sample_count = 20)]
fn bench_3a_pack_scaling_mb_preallocated(bencher: Bencher, size_mb: usize) {
    let target_bytes = size_mb * 1024 * 1024;
    let files = generate_in_memory_files(size_mb * 250, target_bytes, false);
    let exact_bytes = IgnoreFileScanner::calculate_exact_capacity(&files);

    bencher.counter(BytesCount::new(exact_bytes)).bench_local(|| {
        let out = IgnoreFileScanner::pack_slices_preallocated(divan::black_box(&files), false);
        divan::black_box(out)
    });
}

#[divan::bench(args = [1, 4, 16, 64], sample_count = 20)]
fn bench_3b_pack_scaling_mb_naive(bencher: Bencher, size_mb: usize) {
    let target_bytes = size_mb * 1024 * 1024;
    let files = generate_in_memory_files(size_mb * 250, target_bytes, false);
    let exact_bytes = IgnoreFileScanner::calculate_exact_capacity(&files);

    bencher.counter(BytesCount::new(exact_bytes)).bench_local(|| {
        let mut buffer = String::new();
        for (path, content) in divan::black_box(&files) {
            let _ = writeln!(buffer, "=== FILE: {path} ===");
            buffer.push_str(content);
            buffer.push_str("\n=== END FILE ===\n\n");
        }
        divan::black_box(buffer)
    });
}

#[divan::bench(args = ["1_scan_index_only", "2_full_scan_read_pack"], sample_count = 30)]
fn bench_1d_stage_breakdown_4mb(bencher: Bencher, stage: &str) {
    let repo = generate_fs_repo(1_000, 2_000, 4, 4096);
    let scanner = IgnoreFileScanner::new();
    let cancel_token = CancelToken::new();

    bencher.bench_local(|| {
        if stage == "1_scan_index_only" {
            let index = scanner.scan_project_index(&repo.root, true, 32, &cancel_token);
            assert_eq!(index.len(), 1_000);
            divan::black_box(index.len())
        } else {
            let packed =
                scanner.pack_project_end_to_end(&repo.root, &[], false, &cancel_token).unwrap();
            divan::black_box(packed.len())
        }
    });
}

#[divan::bench(args = [10_000, 1_000, 1], sample_count = 40)]
fn bench_4a_pack_4mb_small_vs_large_files(bencher: Bencher, file_count: usize) {
    let files = generate_in_memory_files(file_count, 4 * 1024 * 1024, false);
    let exact_bytes = IgnoreFileScanner::calculate_exact_capacity(&files);

    bencher.counter(BytesCount::new(exact_bytes)).bench_local(|| {
        let out = IgnoreFileScanner::pack_slices_preallocated(divan::black_box(&files), false);
        divan::black_box(out)
    });
}

#[divan::bench(args = [false, true], sample_count = 30)]
fn bench_4b_pack_4mb_lf_vs_crlf_normalization(bencher: Bencher, is_crlf_input: bool) {
    let files = generate_in_memory_files(1_000, 4 * 1024 * 1024, is_crlf_input);
    let exact_bytes = IgnoreFileScanner::calculate_exact_capacity(&files);

    bencher.counter(BytesCount::new(exact_bytes)).bench_local(|| {
        let out =
            IgnoreFileScanner::pack_slices_with_options(divan::black_box(&files), false, true);
        divan::black_box(out)
    });
}

/// 5A. Скорость классификации 10 000 путей через `SecretGuard::inspect_path` + `FileKind::classify_by_path` (0 аллокаций!).
#[divan::bench(sample_count = 100)]
fn bench_5a_secret_guard_and_ext_filter_10k_paths(bencher: Bencher) {
    let paths: Vec<String> = (0..10_000)
        .map(|i| match i % 20 {
            0 => format!("config/service_{i}/.env.production"),
            1 => format!("assets/icons/logo_{i}.png"),
            _ => format!("crates/domain_{}/src/module_{i}.rs", i % 16),
        })
        .collect();

    bencher.counter(ItemsCount::new(paths.len())).bench_local(|| {
        let mut secrets = 0_usize;
        let mut binaries = 0_usize;
        for path in divan::black_box(&paths) {
            if SecretGuard::inspect_path(path).is_some() {
                secrets += 1;
            } else if FileKind::classify_by_path(path).is_binary() {
                binaries += 1;
            }
        }
        assert_eq!(secrets, 500);
        assert_eq!(binaries, 500);
        divan::black_box((secrets, binaries))
    });
}

/// 5B. Сценарий A2: Переключение чекбокса директории на дереве из 50 000 файлов (SLA < 16 мс, по факту < 0.5 мс).
#[divan::bench(sample_count = 100)]
fn bench_5b_project_tree_checkbox_recompute_50k_files(bencher: Bencher) {
    let entries: Vec<FileEntry> = (0..50_000)
        .map(|i| {
            FileEntry::new(
                format!("crates/crate_{}/src/file_{i}.rs", i % 25),
                4096,
                FileKind::SourceCode,
            )
        })
        .collect();

    let mut project =
        Project::new(PathBuf::from("/workspace"), entries, Preset::default(), Vec::new());

    bencher.counter(ItemsCount::new(50_000_usize)).bench_local(|| {
        let off = project.set_directory_selected("crates/crate_7", false);
        let on = project.set_directory_selected("crates/crate_7", true);
        assert_eq!(off, 2_000);
        assert_eq!(on, 2_000);
        divan::black_box(project.selected_bytes())
    });
}

/// 5C. Разбивка 4 МБ контекста на 4 сегмента `PackChunk` по ~1 МБ через `Chunker` (с предрасчётом ёмкости каждого тома).
#[divan::bench(sample_count = 40)]
fn bench_5c_chunker_split_4mb_into_1mb_chunks(bencher: Bencher) {
    let files = generate_in_memory_files(1_000, 4 * 1024 * 1024, false);
    let exact_bytes = IgnoreFileScanner::calculate_exact_capacity(&files);
    let template = OutputTemplate::canonical().with_max_chunk_bytes(Some(1_048_576));

    bencher.counter(BytesCount::new(exact_bytes)).bench_local(|| {
        let chunks = Chunker::pack_into_chunks(divan::black_box(&files), &template, false, false);
        assert!(chunks.len() >= 4);
        divan::black_box(chunks)
    });
}
