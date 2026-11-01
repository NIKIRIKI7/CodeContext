//! Комплексные интеграционные, аллокационные (`dhat`) и property-based (`proptest`) тесты `crates/ui`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::wildcard_imports,
    clippy::indexing_slicing
)]

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use core::packing::aggregates::file_entry::FileEntry;
use core::packing::aggregates::project::Project;
use core::packing::ports::clipboard::ClipboardPort as _;
use core::preset::models::preset::{PROJECT_CONFIG_FILE_NAME, Preset};
use core::shared_kernel::fs::file_kind::FileKind;
use core::system::ports::hotkey::HotkeyShortcut;
use core::system::ports::ipc::{IpcCommand, MAX_IPC_FRAME_BYTES, encode_ipc_frame};
use di::AppContainer;
use proptest::prelude::*;
use tempfile::TempDir;
use ui::features::change_applier::state::ChangeApplierState;
use ui::features::context_packer::components::file_tree::{FlatTreeProjection, TriState};
use ui::features::context_packer::components::rules_editor::RuleToggleKind;
use ui::features::context_packer::state::ContextPackerState;
use ui::features::main_window::state::{MainWindowState, WorkspaceTab};
use ui::features::settings_modal::state::SettingsModalState;
use ui::shared::containers::virtual_text::VirtualTextBuffer;
use ui::shared::primitives::warning_banner::BannerSeverity;

#[test]
fn scenario_a2_directory_checkbox_toggle_updates_tristate_and_live_preview_without_disk_io() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("workspace");
    fs::create_dir_all(root.join("crates/core/src")).unwrap();
    fs::create_dir_all(root.join("crates/ui/src")).unwrap();

    fs::write(root.join("crates/core/src/lib.rs"), "pub fn core_logic() -> u32 { 42 }\n").unwrap();
    fs::write(root.join("crates/ui/src/view.rs"), "pub fn render_ui() {}\n").unwrap();
    fs::write(root.join("crates/ui/src/state.rs"), "pub struct UiState;\n").unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let mut state = ContextPackerState::new(container.clone(), Some(root.clone()));

    assert_eq!(state.tree_projection().selected_files(), 3);
    assert!(state.preview_buffer().as_str().contains("crates/ui/src/view.rs"));

    fs::remove_dir_all(root.join("crates/ui")).unwrap();

    let t0 = Instant::now();
    let changed = state.toggle_tree_node_checkbox("crates/ui", true);
    let elapsed = t0.elapsed();

    assert_eq!(changed, 2);
    assert!(
        elapsed.as_millis() < 16,
        "Пересчёт чекбокса директории превысил SLA 16 мс: {} мс",
        elapsed.as_millis()
    );
    assert_eq!(state.tree_projection().selected_files(), 1);
    assert!(!state.preview_buffer().as_str().contains("crates/ui/src/view.rs"));
    assert!(state.preview_buffer().as_str().contains("crates/core/src/lib.rs"));

    let crates_node = state
        .tree_projection()
        .visible_nodes()
        .iter()
        .find(|n| n.path == "crates" && n.is_dir())
        .unwrap();
    assert_eq!(crates_node.check_state, TriState::Indeterminate);

    let ui_node = state
        .tree_projection()
        .visible_nodes()
        .iter()
        .find(|n| n.path == "crates/ui" && n.is_dir())
        .unwrap();
    assert_eq!(ui_node.check_state, TriState::Unchecked);

    let copied_bytes = state.copy_active_chunk_to_clipboard().unwrap();
    assert!(copied_bytes > 0);
    assert!(state.ui_status().copied_recently);
    let clip_text = container.clipboard().read_text().unwrap();
    assert!(clip_text.contains("pub fn core_logic()"));
    assert!(!clip_text.contains("render_ui"));
}

#[test]
fn interactive_search_master_checkbox_and_modals_work_ergonomically() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("workspace");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/alpha.rs"), "pub fn alpha() {}\n").unwrap();
    fs::write(root.join("src/beta.rs"), "pub fn beta() {}\n").unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let mut packer = ContextPackerState::new(container, Some(root));

    assert_eq!(packer.tree_projection().master_check_state(), TriState::Checked);
    packer.toggle_select_all_files();
    assert_eq!(packer.tree_projection().master_check_state(), TriState::Unchecked);
    packer.toggle_select_all_files();
    assert_eq!(packer.tree_projection().master_check_state(), TriState::Checked);

    for ch in "alpha".chars() {
        packer.append_search_char(ch);
    }
    assert_eq!(packer.search_query(), "alpha");
    assert!(packer.tree_projection().visible_nodes().iter().any(|n| n.display_name == "alpha.rs"));
    assert!(!packer.tree_projection().visible_nodes().iter().any(|n| n.display_name == "beta.rs"));

    packer.clear_search_query();
    assert_eq!(packer.search_query(), "");

    packer.toggle_filters_popover();
    assert!(packer.ui_status().filters_popover_open);

    let mut main_win = MainWindowState::new();
    main_win.toggle_project_meta();
    assert!(main_win.is_project_meta_open());
    main_win.toggle_settings();
    assert!(main_win.is_settings_open());
    assert!(!main_win.is_project_meta_open());
    main_win.set_active_tab(WorkspaceTab::ChangeApplier);
    assert_eq!(main_win.active_tab(), WorkspaceTab::ChangeApplier);
}

#[test]
fn scenario_a3_untrusted_project_config_raises_security_banner_and_blocks_secrets() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("malicious_repo");
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join(".env.production"), "SECRET_KEY=super_secret_123\n").unwrap();
    fs::write(
        root.join(PROJECT_CONFIG_FILE_NAME),
        r#"{
            "schema_version": 1,
            "include_globs": ["src/**", ".env.production"],
            "explicitly_allowed_secrets": [".env.production"]
        }"#,
    )
    .unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let mut state = ContextPackerState::new(container.clone(), Some(root.clone()));

    let banners = state.active_banner_items();
    assert!(
        banners.iter().any(|b| b.severity == BannerSeverity::SecurityCritical),
        "Ожидался критический баннер безопасности для Сценария A3"
    );

    assert!(!state.preview_buffer().as_str().contains("super_secret_123"));

    state.toggle_secret_unlock_in_session(".env.production");
    assert!(state.preview_buffer().as_str().contains("super_secret_123"));

    state.save_current_preset_to_project().unwrap();
    let saved_ctxpack = fs::read_to_string(root.join(PROJECT_CONFIG_FILE_NAME)).unwrap();
    assert!(!saved_ctxpack.contains("explicitly_allowed_secrets"));

    let reloaded_state = ContextPackerState::new(container, Some(root));
    assert!(!reloaded_state.preview_buffer().as_str().contains("super_secret_123"));
}

#[test]
fn rule_toggles_and_chunk_switching_update_preview_in_memory() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("src")).unwrap();

    let payload = "pub fn a() {\r\n    let x = 1;\r\n\r\n}\r\n".repeat(120);
    fs::write(root.join("src/a.rs"), &payload).unwrap();
    fs::write(root.join("src/b.rs"), &payload).unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let mut state = ContextPackerState::new(container, Some(root));

    state.toggle_rule(RuleToggleKind::IncludeFileTree);
    assert!(state.preview_buffer().as_str().starts_with(concat!("=== ", "PROJECT TREE ===")));

    state.set_max_chunk_bytes(Some(4_096));
    assert!(state.pack_result().unwrap().chunks.len() >= 2);

    state.select_chunk(1);
    assert_eq!(state.active_chunk_index(), 1);
    assert!(state.preview_buffer().as_str().contains(concat!("=== ", "FILE: src/b.rs ===")));
}

#[test]
fn external_ipc_command_switches_project_in_packer_and_applier() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let proj_a = temp.path().join("project_a");
    let proj_b = temp.path().join("project_b");
    fs::create_dir_all(proj_a.join("src")).unwrap();
    fs::create_dir_all(proj_b.join("src")).unwrap();
    fs::write(proj_a.join("src/a.rs"), "pub fn from_a() {}\n").unwrap();
    fs::write(proj_b.join("src/b.rs"), "pub fn from_b() {}\n").unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let mut main_state = MainWindowState::new();
    let mut packer = ContextPackerState::new(container.clone(), Some(proj_a.clone()));
    let mut applier = ChangeApplierState::new(container.clone(), Some(proj_a));

    assert!(packer.preview_buffer().as_str().contains("from_a"));

    container
        .ipc()
        .push_simulated_command(IpcCommand::OpenProjectAndFocus { project_root: proj_b.clone() });

    let processed = main_state.poll_system_events(&container, &mut packer, &mut applier, false);
    assert_eq!(processed, 1);
    assert_eq!(packer.project_root(), Some(proj_b.as_path()));
    assert_eq!(applier.project_root(), Some(proj_b.as_path()));
    assert!(packer.preview_buffer().as_str().contains("from_b"));
}

#[test]
fn global_hotkey_activation_auto_reads_and_deduplicates_clipboard_on_apply_tab() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn old() {}\n").unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let mut settings = SettingsModalState::new(container.clone());
    assert!(settings.hotkey_status().is_active());
    settings.toggle_auto_read_clipboard().unwrap();
    assert!(settings.global_config().auto_read_clipboard_on_apply);

    let rebind_st = settings.cycle_hotkey_preset().unwrap();
    assert!(rebind_st.is_active());

    let mut main_state = MainWindowState::new();
    main_state.set_active_tab(WorkspaceTab::ChangeApplier);
    let mut packer = ContextPackerState::new(container.clone(), Some(root.clone()));
    let mut applier = ChangeApplierState::new(container.clone(), Some(root));

    let ai_stream = concat!(
        "=== ",
        "FILE: src/lib.rs ===\n",
        "pub fn updated_via_hotkey() {}\n",
        "=== ",
        "END FILE ===\n",
        "=== ",
        "FILE: src/second.rs ===\n",
        "pub fn second() {}\n",
        "=== ",
        "END FILE ===\n"
    );
    container.clipboard().copy_text(ai_stream).unwrap();

    container.hotkey().simulate_trigger();
    let events = main_state.poll_system_events(&container, &mut packer, &mut applier, true);
    assert_eq!(events, 1);
    assert_eq!(applier.parsed_files().len(), 2);

    applier.select_parsed_file(1);
    assert_eq!(applier.selected_file_index(), 1);

    container.hotkey().simulate_trigger();
    let events_second = main_state.poll_system_events(&container, &mut packer, &mut applier, true);
    assert_eq!(events_second, 1);
    assert_eq!(applier.selected_file_index(), 1);
}

#[test]
fn dhat_zero_heap_allocations_during_virtual_text_and_hotkey_ipc_hot_paths() {
    let line = "pub fn compute_fast(v: u64) -> u64 { v.wrapping_mul(0x517cc1b727220a95) }\n";
    let four_mb_text = line.repeat(56_000);
    let buffer = VirtualTextBuffer::new(four_mb_text);
    assert_eq!(buffer.line_count(), 56_001);

    let ipc_cmd = IpcCommand::OpenProjectAndFocus {
        project_root: PathBuf::from("/workspace/CodeContext/crates/core"),
    };
    let mut stack_frame = [0_u8; MAX_IPC_FRAME_BYTES];

    let _profiler = dhat::Profiler::builder().testing().build();

    let mut total_chars_rendered = 0_usize;
    for start_row in [0_usize, 25_000, 55_900] {
        for row_idx in start_row..(start_row + 80) {
            let (slice, _clamped) = buffer.display_line(row_idx);
            total_chars_rendered += slice.len();
        }
    }
    assert!(total_chars_rendered > 10_000);

    for raw_hk in ["Ctrl+Shift+C", "cmdorctrl + alt + f12", "Super+Shift+KeyP"] {
        let parsed = HotkeyShortcut::parse(raw_hk).unwrap();
        assert!(!parsed.modifiers.is_empty());
    }

    let frame_len = encode_ipc_frame(&ipc_cmd, &mut stack_frame).unwrap();
    assert!(frame_len > 6);

    let stats = dhat::HeapStats::get();
    dhat::assert_eq!(stats.total_blocks, 0);
    dhat::assert_eq!(stats.total_bytes, 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn prop_virtual_text_buffer_never_panics_on_arbitrary_utf8_and_clamping(
        random_text in ".*",
        clamp_limit in 16_usize..256_usize,
        query_row in 0_usize..120_usize,
    ) {
        let buf = VirtualTextBuffer::new(random_text).with_max_display_line_bytes(clamp_limit);
        let total = buf.line_count();
        for idx in 0..total {
            let (slice, is_clamped) = buf.display_line(idx);
            prop_assert!(slice.len() <= clamp_limit);
            if !is_clamped {
                prop_assert_eq!(slice, buf.raw_line(idx));
            }
        }
        let _ = buf.display_line(query_row);
    }

    #[test]
    fn prop_flat_tree_tristate_invariants_hold_under_partial_selections(
        selected_flags in prop::collection::vec(any::<bool>(), 1..30)
    ) {
        let mut entries = Vec::with_capacity(selected_flags.len());
        for (i, &is_sel) in selected_flags.iter().enumerate() {
            let mut e = FileEntry::new(
                format!("crates/mod_a/file_{i:02}.rs"),
                256,
                FileKind::SourceCode,
            );
            e.set_selected(is_sel, false);
            entries.push(e);
        }

        let project = Project::new(
            PathBuf::from("/workspace"),
            entries,
            Preset::default(),
            Vec::new(),
        );
        let proj = FlatTreeProjection::build(&project, &HashSet::new(), "");

        let dir_node = proj
            .visible_nodes()
            .iter()
            .find(|n| n.path == "crates/mod_a" && n.is_dir())
            .unwrap();

        let all_true = selected_flags.iter().all(|&b| b);
        let all_false = selected_flags.iter().all(|&b| !b);

        if all_true {
            prop_assert_eq!(dir_node.check_state, TriState::Checked);
        } else if all_false {
            prop_assert_eq!(dir_node.check_state, TriState::Unchecked);
        } else {
            prop_assert_eq!(dir_node.check_state, TriState::Indeterminate);
        }
    }
}
