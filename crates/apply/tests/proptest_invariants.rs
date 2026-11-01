#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::wildcard_imports,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::Path;

use apply::{ParsedFile, ResponseApplier};
use core::applying::ports::applier::ApplierPort;
use core::preset::models::template::{OutputTemplate, ParserConfig};
use proptest::prelude::*;
use scanner::IgnoreFileScanner;
use tempfile::TempDir;

#[test]
fn regression_1_strips_conversational_preamble_and_postamble() {
    let ai_response = concat!(
        "Конечно! Вот исправленные файлы для вашего проекта:\n\n",
        "=== ",
        "FILE: src/main.rs ===\n",
        "fn main() {\n    println!(\"Hello\");\n}\n",
        "=== ",
        "END FILE ===\n\n",
        "=== ",
        "FILE: Cargo.toml ===\n",
        "[package]\nname = \"demo\"\n",
        "=== ",
        "END FILE ===\n\n",
        "Надеюсь, это поможет! Дайте знать, если нужны ещё правки."
    );
    let fixed = ResponseApplier::autofix(ai_response);
    assert!(fixed.preamble_stripped);
    assert!(!fixed.closed_truncated_block);
    let parsed = ResponseApplier::parse(&fixed.fixed_text);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].rel_path, "src/main.rs");
    assert_eq!(parsed[0].content, "fn main() {\n    println!(\"Hello\");\n}");
    assert!(!parsed[0].is_truncated);
    assert_eq!(parsed[1].rel_path, "Cargo.toml");
}

#[test]
fn regression_2_recovers_truncated_response_cut_off_mid_file() {
    let truncated_ai_response = concat!(
        "Вот обновлённый модуль:\n\n",
        "=== ",
        "FILE: crates/core/src/lib.rs ===\n",
        "pub mod packing;\n\n",
        "pub fn unfinished_function(x: u32) -> u32 {\n    x + 1"
    );
    let raw_parsed = ResponseApplier::parse(truncated_ai_response);
    assert_eq!(raw_parsed.len(), 1);
    assert!(raw_parsed[0].is_truncated);

    let fixed = ResponseApplier::autofix(truncated_ai_response);
    assert!(fixed.preamble_stripped);
    assert!(fixed.closed_truncated_block);

    let after_fix = ResponseApplier::parse(&fixed.fixed_text);
    assert_eq!(after_fix.len(), 1);
    assert_eq!(after_fix[0].rel_path, "crates/core/src/lib.rs");
    assert!(after_fix[0].content.contains("unfinished_function"));
}

#[test]
fn regression_3_parses_markdown_headers_and_unclosed_code_fences() {
    let markdown_response = concat!(
        "Я переписал конфигурацию:\n\n",
        "### `crates/config/src/lib.rs`\n",
        "\x60\x60\x60rust\n",
        "pub struct AppConfig {\n    pub threads: usize,\n}\n",
        "\x60\x60\x60\n\n",
        "### `crates/config/src/schema.rs`\n",
        "\x60\x60\x60rust\n",
        "pub struct SchemaDto {\n    pub version: u32,"
    );
    let fixed = ResponseApplier::autofix(markdown_response);
    assert!(fixed.preamble_stripped);
    assert!(fixed.closed_truncated_block);
    let parsed = ResponseApplier::parse(&fixed.fixed_text);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].rel_path, "crates/config/src/lib.rs");
    assert_eq!(parsed[1].rel_path, "crates/config/src/schema.rs");
}

#[test]
fn regression_4_preserves_original_crlf_and_lf_line_endings() {
    let crlf_response = concat!(
        "=== ",
        "FILE: win/script.bat ===\r\n",
        "@echo off\r\necho Done\r\n",
        "=== ",
        "END FILE ===\r\n"
    );
    let parsed = ResponseApplier::parse(crlf_response);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].rel_path, "win/script.bat");
    assert_eq!(parsed[0].content, "@echo off\r\necho Done");
}

#[test]
fn regression_5_parses_web_chat_first_line_comments_and_strips_root_prefix() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("CodeContext");
    fs::create_dir_all(&root).unwrap();

    let web_chat_response = concat!(
        "Вот исправления:\n\n",
        "\x60\x60\x60rust\n",
        "// filepath: CodeContext/crates/app/src/main.rs\n",
        "fn main() {}\n",
        "\x60\x60\x60\n\n",
        "\x60\x60\x60python\n",
        "# scripts/deploy.py\n",
        "print('ok')\n",
        "\x60\x60\x60\n"
    );
    let applier = ResponseApplier::new();
    let plan = applier.build_apply_plan(
        Some(&root),
        web_chat_response,
        &OutputTemplate::canonical(),
        &ParserConfig::canonical(),
    );
    assert_eq!(plan.items.len(), 2);
    assert_eq!(plan.items[0].rel_path, "crates/app/src/main.rs");
    assert_eq!(plan.items[0].resolved_content, "fn main() {}\n");
    assert_eq!(plan.items[1].rel_path, "scripts/deploy.py");
    assert_eq!(plan.items[1].resolved_content, "print('ok')\n");
}

#[test]
fn regression_6_applies_search_replace_and_unified_diff_blocks_from_web_chat() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn greet() -> &'static str {\n    \"old\"\n}\n")
        .unwrap();

    let sr_response = concat!(
        "### `src/lib.rs`\n",
        "\x60\x60\x60rust\n",
        "<<<<<<< ",
        "SEARCH\n",
        "    \"old\"\n",
        "=======\n",
        "    \"new_from_search_replace\"\n",
        ">>>>>>> ",
        "REPLACE\n",
        "\x60\x60\x60\n"
    );
    let applier = ResponseApplier::new();
    let plan = applier.build_apply_plan(
        Some(root),
        sr_response,
        &OutputTemplate::canonical(),
        &ParserConfig::canonical(),
    );
    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].added_lines, 1);
    assert_eq!(plan.items[0].removed_lines, 1);
    assert!(plan.items[0].resolved_content.contains("new_from_search_replace"));

    let manifest = applier.execute_apply_plan(root, &plan, true).unwrap();
    assert_eq!(manifest.applied_files_count, 1);
    let on_disk = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    assert!(on_disk.contains("new_from_search_replace"));
}

#[test]
fn regression_7_creates_new_files_deletes_obsolete_files_and_undoes_both() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/legacy.rs"), "pub fn old_legacy() {}\n").unwrap();

    let chat_response = concat!(
        "\x60\x60\x60text\n",
        "=== ",
        "CREATE FILE: src/sub/new_feature.rs ===\n",
        "\x60\x60\x60rust\n",
        "pub fn new_feature() -> u32 { 77 }\n",
        "\x60\x60\x60\n",
        "=== ",
        "END FILE ===\n\n",
        "### Удалить файл `src/legacy.rs`\n",
        "\x60\x60\x60\n"
    );
    let fixed = ResponseApplier::autofix(chat_response);
    let applier = ResponseApplier::new();
    let plan = applier.build_apply_plan(
        Some(root),
        &fixed.fixed_text,
        &OutputTemplate::canonical(),
        &ParserConfig::canonical(),
    );
    assert_eq!(plan.items.len(), 2);

    let manifest = applier.execute_apply_plan(root, &plan, true).unwrap();
    assert_eq!(manifest.applied_files_count, 2);
    assert!(root.join("src/sub/new_feature.rs").exists());
    assert!(!root.join("src/legacy.rs").exists());

    let created_body = fs::read_to_string(root.join("src/sub/new_feature.rs")).unwrap();
    assert_eq!(created_body, "pub fn new_feature() -> u32 { 77 }");

    let undone = applier.undo_last_backup(root).unwrap();
    assert_eq!(undone, 2);
    assert!(!root.join("src/sub/new_feature.rs").exists());
    assert!(root.join("src/legacy.rs").exists());
}

#[test]
fn regression_8_chain_of_responsibility_extracts_mixed_search_replace_and_canonical_create_in_one_response()
 {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/existing.rs"), "pub fn compute() -> u32 {\n    10\n}\n").unwrap();

    let mixed_response = concat!(
        "### `src/existing.rs`\n",
        "\x60\x60\x60rust\n",
        "<<<<<<< ",
        "SEARCH\n",
        "    10\n",
        "=======\n",
        "    20\n",
        ">>>>>>> ",
        "REPLACE\n",
        "\x60\x60\x60\n\n",
        "=== ",
        "CREATE FILE: src/created_module.rs ===\n",
        "pub const CREATED: bool = true;\n",
        "=== ",
        "END FILE ===\n"
    );

    let applier = ResponseApplier::new();
    let plan = applier.build_apply_plan(
        Some(root),
        mixed_response,
        &OutputTemplate::canonical(),
        &ParserConfig::canonical(),
    );
    assert_eq!(plan.items.len(), 2);
    assert_eq!(plan.items[0].rel_path, "src/existing.rs");
    assert!(plan.items[0].resolved_content.contains("20"));
    assert_eq!(plan.items[1].rel_path, "src/created_module.rs");
    assert_eq!(plan.items[1].resolved_content, "pub const CREATED: bool = true;");
}

#[test]
fn regression_9_autofix_glued_fence_and_lone_footer_is_idempotent() {
    let corrupted = concat!("¡\x60\x60\x60\n=== ", "END FILE ===\n");
    let fix1 = ResponseApplier::autofix(corrupted);
    let fix2 = ResponseApplier::autofix(&fix1.fixed_text);
    assert_eq!(fix1.fixed_text, fix2.fixed_text);
}

#[test]
fn regression_10_parses_extensionless_root_files_without_leaking_search_replace_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("crates/app/src")).unwrap();
    fs::write(root.join("crates/app/src/main.rs"), "fn main() {\n    run();\n}\n").unwrap();
    fs::write(root.join("Justfile"), "default:\n    @just --list\n").unwrap();

    let stream = concat!(
        "=== ",
        "FILE: crates/app/src/main.rs ===\n",
        "<<<<<<< ",
        "SEARCH\n",
        "fn main() {\n    run();\n}\n",
        "=======\n",
        "fn main() {\n    init();\n    run();\n}\n",
        ">>>>>>> ",
        "REPLACE\n",
        "=== ",
        "END FILE ===\n\n",
        "=== ",
        "FILE: Justfile ===\n",
        "<<<<<<< ",
        "SEARCH\n",
        "default:\n    @just --list\n",
        "=======\n",
        "default:\n    @just --list\n\nversion:\n    @cargo --version\n",
        ">>>>>>> ",
        "REPLACE\n",
        "=== ",
        "END FILE ===\n\n",
        "=== ",
        "FILE: VERSION ===\n",
        "<<<<<<< ",
        "SEARCH\n",
        "=======\n",
        "0.1.0\n",
        ">>>>>>> ",
        "REPLACE\n",
        "=== ",
        "END FILE ===\n"
    );

    let applier = ResponseApplier::new();
    let plan = applier.build_apply_plan(
        Some(root),
        stream,
        &OutputTemplate::canonical(),
        &ParserConfig::canonical(),
    );
    assert_eq!(plan.items.len(), 3);
    assert!(!plan.has_blocking_conflicts());
    assert_eq!(plan.items[0].rel_path, "crates/app/src/main.rs");
    assert_eq!(plan.items[1].rel_path, "Justfile");
    assert_eq!(plan.items[2].rel_path, "VERSION");
}

#[test]
fn sandbox_negative_scenarios_matrix() {
    let root = Path::new("/workspace/project");
    let forbidden_cases = [
        "",
        "   ",
        "src/main\0.rs",
        "/etc/passwd",
        "\\Windows\\System32\\drivers\\etc\\hosts",
        "\\\\evil-server\\share\\malware.dll",
        "\\\\?\\C:\\Windows\\win.ini",
        "C:\\Users\\Admin\\.ssh\\id_rsa",
        "D:relative_drive_escape.rs",
        "src/main.rs:hidden_ads_stream",
        "../outside.rs",
        "..\\outside.rs",
        "crates/core/../../../outside.rs",
        "a/b/c/../../../../root_escape.txt",
    ];
    for bad_path in forbidden_cases {
        let res = ResponseApplier::validate_path(root, bad_path);
        assert!(
            res.is_err(),
            "Песочница обязана заблокировать путь `{bad_path}`, но вернула: {res:?}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1500))]

    #[test]
    fn prop_1_sandbox_blocks_any_traversal_or_absolute_path(
        prefix in prop_oneof![
            Just("../".to_string()),
            Just("..\\".to_string()),
            Just("/etc/".to_string()),
            Just("C:\\Windows\\".to_string()),
            Just("\\\\server\\share\\".to_string()),
            Just("\\\\?\\C:\\".to_string()),
            Just("src/../../".to_string()),
            Just("a/b/../../../".to_string()),
        ],
        suffix in "[a-zA-Z0-9_./\\\\:]{0,40}"
    ) {
        let root = Path::new("/workspace/my_project");
        let malicious_input = format!("{prefix}{suffix}");
        let result = ResponseApplier::validate_path(root, &malicious_input);
        prop_assert!(
            result.is_err(),
            "Песочница пропустила опасный путь: `{malicious_input}` -> {result:?}"
        );
    }

    #[test]
    fn prop_2_sandbox_allows_valid_nested_project_paths(
        segments in prop::collection::vec("[a-zA-Z0-9_-]{1,15}", 1..6),
        ext in "(rs|toml|json|md)"
    ) {
        let root = Path::new("/workspace/my_project");
        let rel = format!("{}.{}", segments.join("/"), ext);
        let resolved = ResponseApplier::validate_path(root, &rel).unwrap();
        prop_assert!(resolved.starts_with(root));
    }

    #[test]
    fn prop_3_parser_and_autofixer_never_panic_and_autofix_is_idempotent(
        garbage_prefix in ".*",
        file_marker in prop_oneof![
            Just(concat!("=== ", "FILE: src/main.rs ===\n")),
            Just("### `src/lib.rs`\n\x60\x60\x60rust\n"),
            Just(concat!("=== ", "FILE: ../outside.rs ===\n")),
            Just(concat!("=== ", "FILE: ")),
            Just("\x60\x60\x60"),
        ],
        code_body in ".*",
        closing_marker in prop_oneof![
            Just(concat!("\n=== ", "END FILE ===\n")),
            Just("\n\x60\x60\x60\n"),
            Just(""),
        ]
    ) {
        let corrupted = format!("{garbage_prefix}{file_marker}{code_body}{closing_marker}");
        let _ = ResponseApplier::parse(&corrupted);
        let fix1 = ResponseApplier::autofix(&corrupted);
        let fix2 = ResponseApplier::autofix(&fix1.fixed_text);
        prop_assert_eq!(&fix1.fixed_text, &fix2.fixed_text);
        let _ = ResponseApplier::parse(&fix1.fixed_text);
    }

    #[test]
    fn prop_4_pack_then_parse_roundtrip_preserves_files_and_line_endings(
        raw_files in prop::collection::vec(
            (
                "[a-z]{2,8}/[a-z]{2,8}\\.(rs|toml|md)",
                "[a-zA-Z0-9_ \t\r\n(){};=+-]{1,300}"
            ),
            1..15
        )
    ) {
        let packed = IgnoreFileScanner::pack_slices_preallocated(&raw_files, false);
        let fixed = ResponseApplier::autofix(&packed);
        let parsed_direct = ResponseApplier::parse(&packed);
        let parsed_after_fix = ResponseApplier::parse(&fixed.fixed_text);
        prop_assert_eq!(&parsed_direct, &parsed_after_fix);
        prop_assert_eq!(parsed_direct.len(), raw_files.len());
        for ((expected_path, expected_content), actual) in raw_files.iter().zip(parsed_direct.iter()) {
            let expected = ParsedFile {
                rel_path: expected_path.clone(),
                content: expected_content.clone(),
                is_truncated: false,
            };
            prop_assert_eq!(actual, &expected);
        }
    }
}
