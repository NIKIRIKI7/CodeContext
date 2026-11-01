#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value
)]

use std::fmt::Write as _;
use std::path::Path;

use apply::{ParsedFile, ResponseApplier, apply_operation_to_text, build_unified_diff_preview};
use core::applying::aggregates::apply_plan::FileOperation;
use core::applying::ports::applier::ApplierPort;
use core::applying::values::patch_status::SearchReplaceBlock;
use core::preset::models::template::{OutputTemplate, ParserConfig};
use core::shared_kernel::fs::line_ending::LineEnding;
use divan::counter::{BytesCount, ItemsCount};
use divan::{AllocProfiler, Bencher};
use tempfile::TempDir;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    print_apply_bench_banner();
    divan::main();
}

fn print_apply_bench_banner() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let logical_cores = std::thread::available_parallelism().map_or(1, usize::from);
    println!("==========================================================================");
    println!(" CodeContext Module 2 (Change Applier) Benchmark Environment");
    println!(" OS / Arch       : {os} / {arch}");
    println!(" CPU Logical     : {logical_cores} logical cores");
    println!("==========================================================================");
}

fn generate_canonical_stream(file_count: usize) -> String {
    let mut out = String::with_capacity(file_count * 512);
    for i in 0..file_count {
        let _ = writeln!(out, "{}crates/domain/src/mod_{i}.rs ===", concat!("=== ", "FILE: "));
        out.push_str(
            "pub fn compute_hash(x: u64) -> u64 {\n    x.wrapping_mul(0x517cc1b727220a95)\n}\n",
        );
        out.push_str(concat!("=== ", "END FILE ===\n\n"));
    }
    out
}

fn generate_markdown_first_line_comment_stream(file_count: usize) -> String {
    let mut out = String::with_capacity(file_count * 512);
    for i in 0..file_count {
        out.push_str("\x60\x60\x60rust\n");
        let _ = writeln!(out, "// filepath: crates/domain/src/mod_{i}.rs");
        out.push_str(
            "pub fn compute_hash(x: u64) -> u64 {\n    x.wrapping_mul(0x517cc1b727220a95)\n}\n",
        );
        out.push_str("\x60\x60\x60\n\n");
    }
    out
}

fn generate_search_replace_stream(file_count: usize) -> String {
    let mut out = String::with_capacity(file_count * 512);
    for i in 0..file_count {
        let _ = writeln!(out, "### `crates/domain/src/mod_{i}.rs`");
        out.push_str(concat!(
            "\x60\x60\x60rust\n",
            "<<<<<<< ",
            "SEARCH\n    let old = 1;\n=======\n    let new = 2;\n>>>>>>> ",
            "REPLACE\n\x60\x60\x60\n\n"
        ));
    }
    out
}

fn generate_unified_diff_stream(file_count: usize) -> String {
    let mut out = String::with_capacity(file_count * 512);
    for i in 0..file_count {
        let _ = writeln!(out, "--- a/crates/domain/src/mod_{i}.rs");
        let _ = writeln!(out, "+++ b/crates/domain/src/mod_{i}.rs");
        out.push_str(
            "@@ -1,3 +1,3 @@\n pub fn run() {\n-    let old = 1;\n+    let new = 2;\n }\n",
        );
    }
    out
}

#[divan::bench(
    args = ["canonical", "md_first_line_comment", "search_replace", "unified_diff"],
    sample_count = 60
)]
fn bench_1_parse_formats_comparison(bencher: Bencher, format_kind: &str) {
    let file_count = 200;
    let payload = match format_kind {
        "md_first_line_comment" => generate_markdown_first_line_comment_stream(file_count),
        "search_replace" => generate_search_replace_stream(file_count),
        "unified_diff" => generate_unified_diff_stream(file_count),
        _ => generate_canonical_stream(file_count),
    };
    let applier = ResponseApplier::new();
    let tpl = OutputTemplate::canonical();
    let cfg = ParserConfig::canonical();

    bencher.counter(BytesCount::new(payload.len())).bench_local(|| {
        let plan = applier.build_apply_plan(None, divan::black_box(&payload), &tpl, &cfg);
        assert_eq!(plan.items.len(), file_count);
        divan::black_box(plan)
    });
}

#[divan::bench(args = ["canonical", "md_first_line_comment"], sample_count = 100)]
fn bench_1b_pure_parser_only_throughput(bencher: Bencher, format_kind: &str) {
    let file_count = 200;
    let payload = if format_kind == "md_first_line_comment" {
        generate_markdown_first_line_comment_stream(file_count)
    } else {
        generate_canonical_stream(file_count)
    };

    bencher.counter(BytesCount::new(payload.len())).bench_local(|| {
        let parsed = ResponseApplier::parse(divan::black_box(&payload));
        assert_eq!(parsed.len(), file_count);
        divan::black_box(parsed)
    });
}

#[divan::bench(args = [50, 250, 1000], sample_count = 40)]
fn bench_2_autofix_corrupted_stream_scaling(bencher: Bencher, file_count: usize) {
    let mut corrupted = String::from("Конечно! Вот исправленные файлы вашего проекта:\n\n");
    corrupted.push_str(&generate_canonical_stream(file_count));
    corrupted.push_str(concat!(
        "=== ",
        "FILE: crates/domain/src/truncated_tail.rs ===\npub fn cut_off() {"
    ));

    bencher.counter(BytesCount::new(corrupted.len())).bench_local(|| {
        let fixed = ResponseApplier::autofix(divan::black_box(&corrupted));
        assert!(fixed.preamble_stripped);
        assert!(fixed.closed_truncated_block);
        divan::black_box(fixed)
    });
}

#[divan::bench(args = ["exact", "fuzzy_indent"], sample_count = 60)]
fn bench_3_patcher_search_replace_exact_vs_fuzzy(bencher: Bencher, mode: &str) {
    let mut base_file = String::with_capacity(64_000);
    for i in 0..1_000 {
        let _ =
            writeln!(base_file, "pub fn item_{i}(v: u64) -> u64 {{\n    v.wrapping_add({i})\n}}");
    }

    let blocks: Vec<SearchReplaceBlock> = (0..20)
        .map(|k| {
            let idx = k * 45;
            let search = if mode == "exact" {
                format!("pub fn item_{idx}(v: u64) -> u64 {{\n    v.wrapping_add({idx})\n}}")
            } else {
                format!("pub fn item_{idx}(v: u64) -> u64 {{\n  v.wrapping_add({idx})\n}}")
            };
            let replace =
                format!("pub fn item_{idx}(v: u64) -> u64 {{\n    v.saturating_add({idx})\n}}");
            SearchReplaceBlock::new(search, replace)
        })
        .collect();
    let op = FileOperation::SearchReplace { blocks };

    bencher.counter(BytesCount::new(base_file.len())).bench_local(|| {
        let out = apply_operation_to_text(
            "src/large.rs",
            Some(divan::black_box(&base_file)),
            divan::black_box(&op),
            false,
        );
        divan::black_box(out)
    });
}

#[divan::bench(args = [200, 1_000, 4_000], sample_count = 50)]
fn bench_4_similar_diff_calculation(bencher: Bencher, line_count: usize) {
    let mut old_text = String::with_capacity(line_count * 48);
    let mut new_text = String::with_capacity(line_count * 48);
    for i in 0..line_count {
        let _ = writeln!(old_text, "    let value_{i} = compute_step({i});");
        if i % 25 == 0 {
            let _ = writeln!(new_text, "    let value_{i} = compute_fast_step({i});");
        } else {
            let _ = writeln!(new_text, "    let value_{i} = compute_step({i});");
        }
    }

    bencher.counter(ItemsCount::new(line_count)).bench_local(|| {
        let summary = build_unified_diff_preview(
            "src/engine.rs",
            Some(divan::black_box(&old_text)),
            divan::black_box(&new_text),
            LineEnding::Lf,
        );
        assert!(summary.added_lines > 0);
        divan::black_box(summary)
    });
}

#[divan::bench(sample_count = 100)]
fn bench_5_sandbox_validation_10k_paths(bencher: Bencher) {
    let root = Path::new("/workspace/project");
    let paths: Vec<String> = (0..10_000)
        .map(|i| match i % 10 {
            0 => format!("crates/core/../../../etc/passwd_{i}"),
            1 => format!("C:\\Windows\\System32\\cmd_{i}.exe"),
            _ => format!("crates/module_{}/src/service_{i}.rs", i % 16),
        })
        .collect();

    bencher.counter(ItemsCount::new(paths.len())).bench_local(|| {
        let mut allowed = 0_usize;
        for p in divan::black_box(&paths) {
            if ResponseApplier::validate_path(root, p).is_ok() {
                allowed += 1;
            }
        }
        assert_eq!(allowed, 8_000);
        divan::black_box(allowed)
    });
}

#[divan::bench(args = [10, 50], sample_count = 20)]
fn bench_6_transactional_apply_and_undo_io(bencher: Bencher, file_count: usize) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let files: Vec<ParsedFile> = (0..file_count)
        .map(|i| ParsedFile {
            rel_path: format!("src/mod_{i}.rs"),
            content: format!("pub const ID_{i}: usize = {i};\n"),
            is_truncated: false,
        })
        .collect();

    bencher.counter(ItemsCount::new(file_count)).bench_local(|| {
        let rep = ResponseApplier::apply_files(root, divan::black_box(&files), true).unwrap();
        assert_eq!(rep.applied_files_count, file_count);
        let undone = ResponseApplier::undo_last_backup(root).unwrap();
        assert_eq!(undone, file_count);
        divan::black_box(undone)
    });
}

#[divan::bench(sample_count = 80)]
fn bench_7_create_and_delete_directives_plan(bencher: Bencher) {
    let mut stream = String::with_capacity(16_384);
    for i in 0..50 {
        let _ = writeln!(stream, "{}src/new_mod_{i}.rs ===", concat!("=== ", "CREATE FILE: "));
        stream.push_str("pub const NEW_MOD: bool = true;\n");
        stream.push_str(concat!("=== ", "END FILE ===\n\n"));
        let _ = writeln!(stream, "### Удалить файл `src/obsolete_{i}.rs`\n");
    }
    let applier = ResponseApplier::new();
    let tpl = OutputTemplate::canonical();
    let cfg = ParserConfig::canonical();

    bencher.counter(ItemsCount::new(100_usize)).bench_local(|| {
        let plan = applier.build_apply_plan(None, divan::black_box(&stream), &tpl, &cfg);
        assert_eq!(plan.items.len(), 100);
        divan::black_box(plan)
    });
}
