//! Математический контроль аллокаций кучи (`dhat-rs`), проверка инвариантов поддомена `packing`
//! и wall-clock SLA (< 1 сек).
//!
//! Запуск в релизном профиле:
//! `cargo test --release --package scanner --test heap_allocations -- --test-threads=1 --nocapture`

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::fs;
use std::sync::Mutex;
use std::time::Instant;

use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};
use core::packing::services::chunker::Chunker;
use core::packing::services::pack_service::PackService;
use core::packing::services::secret_guard::SecretGuard;
use core::preset::models::preset::Preset;
use core::preset::models::template::OutputTemplate;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::shared_kernel::fs::file_kind::FileKind;
use scanner::IgnoreFileScanner;
use tempfile::TempDir;

#[derive(Debug, Default)]
struct MockClipboard {
    copied: Mutex<String>,
}

impl ClipboardPort for MockClipboard {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardPortError> {
        text.clone_into(&mut self.copied.lock().unwrap());
        Ok(())
    }
}

#[test]
fn test_all_allocation_invariants_and_wall_clock_budget() {
    // Подготавливаем входные данные ДО включения профайлера кучи:
    let line_40b = "pub fn calc(x: u64) -> u64 { x ^ 0x55 }\n"; // ровно 40 байт
    let line_crlf = "pub fn calc(x: u64) -> u64 {\r\n    x ^ 0x55\r\n}\r\n";

    // Профиль А: 1 файл × 4 МБ
    let files_1_x_4mb = vec![("src/monolith.rs".to_owned(), line_40b.repeat(100_000))];

    // Профиль Б: 1 000 файлов × 4 КБ
    let chunk_4kb = line_40b.repeat(100);
    let files_1000_x_4kb: Vec<(String, String)> =
        (0..1_000).map(|i| (format!("crates/mod_{i}/src/lib.rs"), chunk_4kb.clone())).collect();

    // Профиль В: 10 000 мелких файлов × 400 байт
    let chunk_400b = line_40b.repeat(10);
    let files_10000_x_400b: Vec<(String, String)> = (0..10_000)
        .map(|i| (format!("packages/pkg_{}/item_{i}.ts", i % 100), chunk_400b.clone()))
        .collect();

    // Профиль Г: 1 000 файлов с CRLF
    let chunk_crlf = line_crlf.repeat(90);
    let files_1000_crlf: Vec<(String, String)> =
        (0..1_000).map(|i| (format!("win/file_{i}.rs"), chunk_crlf.clone())).collect();

    let canonical_tpl = OutputTemplate::canonical();

    // ========================================================================
    // ЧАСТЬ 1: СТРОГИЙ КОНТРОЛЬ АЛЛОКАЦИЙ КУЧИ (DHAT)
    // ========================================================================
    {
        let _profiler = dhat::Profiler::builder().testing().build();

        // 1.1. Детектор бинарников + `SecretGuard::inspect_path` + `FileKind` (РОВНО 0 аллокаций!)
        assert!(!IgnoreFileScanner::is_binary(chunk_4kb.as_bytes()));
        assert!(IgnoreFileScanner::is_binary(b"PNG\r\n\x1a\n\x00binary"));
        for (path, _) in &files_10000_x_400b {
            assert!(SecretGuard::inspect_path(path).is_none());
            assert!(!FileKind::is_known_binary_extension(path));
        }
        assert!(SecretGuard::inspect_path(".env.production").is_some());
        assert!(FileKind::is_known_binary_extension("assets/icon.png"));

        let s0 = dhat::HeapStats::get();
        dhat::assert_eq!(s0.total_blocks, 0);
        dhat::assert_eq!(s0.total_bytes, 0);

        // 1.2. Профиль А: 1 файл × 4 МБ -> ровно 1 аллокация
        assert_single_allocation(&files_1_x_4mb, false, false);

        // 1.3. Профиль Б: 1 000 файлов × 4 КБ -> ровно 1 аллокация
        assert_single_allocation(&files_1000_x_4kb, false, false);

        // 1.4. Профиль В: 10 000 мелких файлов × 400 Б -> ровно 1 аллокация
        assert_single_allocation(&files_10000_x_400b, false, false);

        // 1.5. Потоковая минификация (`minify = true`) -> ровно 1 аллокация
        assert_single_allocation(&files_1000_x_4kb, true, false);

        // 1.6. Потоковая AVX2-нормализация `CRLF -> LF` -> ровно 1 аллокация
        assert_single_allocation(&files_1000_crlf, false, true);

        // 1.7. Доменный `Chunker` без лимита чанка: 1 аллокация строки + 1 вектор из 1 элемента
        let before_chunk = dhat::HeapStats::get();
        let chunks = Chunker::pack_into_chunks(&files_1000_x_4kb, &canonical_tpl, false, false);
        let after_chunk = dhat::HeapStats::get();
        assert_eq!(chunks.len(), 1);
        dhat::assert_eq!(after_chunk.total_blocks - before_chunk.total_blocks, 2);
        drop(chunks);

        // 1.8. Повторные вызовы в долгоживущем GUI (5 циклов подряд): 0 утечек
        for _ in 0..5 {
            let buf = IgnoreFileScanner::pack_slices_preallocated(&files_1000_x_4kb, false);
            assert!(buf.len() > 4_000_000);
        }
        let end_stats = dhat::HeapStats::get();
        dhat::assert_eq!(end_stats.curr_blocks, 0);
        dhat::assert_eq!(end_stats.curr_bytes, 0);
    }

    // ========================================================================
    // ЧАСТЬ 2: МНОГОПОТОЧНЫЙ СКВОЗНОЙ ТЕСТ С ДИСКА (`PackService` + `SecretGuard` < 1 СЕК)
    // ========================================================================
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    fs::write(root.join(".gitignore"), "node_modules/\n*.bin\n").unwrap();

    // `corrupted.bin` отсекается через `.gitignore` (`require_git(false)`),
    // а `disguised_binary.rs` отсекается на Фазе 2 по `NUL`-байту в первых 8 КБ:
    fs::write(root.join("corrupted.bin"), b"ELF\x00\x01\x02\x03").unwrap();
    fs::write(root.join("disguised_binary.rs"), b"fn main() {}\x00binary_tail").unwrap();
    fs::write(root.join(".env.production"), "DATABASE_URL=postgres://secret\n").unwrap();
    fs::write(root.join("id_rsa"), "-----BEGIN RSA PRIVATE KEY-----\nSECRET\n").unwrap();

    let nm_dir = root.join("node_modules/react");
    fs::create_dir_all(&nm_dir).unwrap();
    fs::write(nm_dir.join("index.js"), "module.exports = {};").unwrap();

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    for i in 0..1_000 {
        fs::write(src_dir.join(format!("mod_{i}.rs")), &chunk_4kb).unwrap();
    }

    let service = PackService::new(IgnoreFileScanner::new(), MockClipboard::default());
    let cancel_token = CancelToken::new();

    let start = Instant::now();
    let pack_result = service.pack_headless(root, Preset::default(), true, &cancel_token).unwrap();
    let elapsed = start.elapsed();

    let packed_text = pack_result.primary_text();
    assert!(!packed_text.contains("corrupted.bin"));
    assert!(!packed_text.contains("disguised_binary.rs"));
    assert!(!packed_text.contains(".env.production"));
    assert!(!packed_text.contains("DATABASE_URL"));
    assert!(!packed_text.contains("id_rsa"));
    assert!(!packed_text.contains("node_modules"));

    assert_eq!(pack_result.packed_files_count, 1_000);
    assert_eq!(pack_result.blocked_secrets_count, 2);
    assert_eq!(pack_result.skipped_binary_count, 1);
    assert!(pack_result.total_bytes >= 4_000_000);
    assert!(
        elapsed.as_millis() < 1_000,
        "Нарушен главный KPI скорости: сквозная упаковка 4 МБ через PackService заняла {} мс (лимит < 1000 мс)",
        elapsed.as_millis()
    );
}

fn assert_single_allocation(files: &[(String, String)], minify: bool, normalize_lf: bool) {
    let expected_cap = IgnoreFileScanner::calculate_exact_capacity(files);
    let before = dhat::HeapStats::get();
    let out = IgnoreFileScanner::pack_slices_with_options(files, minify, normalize_lf);
    let after = dhat::HeapStats::get();

    let delta_blocks = after.total_blocks - before.total_blocks;
    let delta_bytes = after.total_bytes - before.total_bytes;

    dhat::assert_eq!(delta_blocks, 1);
    dhat::assert_eq!(delta_bytes, expected_cap as u64);
    assert_eq!(out.capacity(), expected_cap);

    if !minify && !normalize_lf {
        assert_eq!(out.len(), expected_cap);
    }
}
