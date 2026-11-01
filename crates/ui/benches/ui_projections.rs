//! Бенчмарки пропускной способности виртуализированных проекций UI (`divan`).
//! Запуск: `cargo bench --package ui`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value
)]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use core::packing::aggregates::file_entry::FileEntry;
use core::packing::aggregates::project::Project;
use core::packing::ports::clipboard::ClipboardPort as _;
use core::preset::models::preset::Preset;
use core::shared_kernel::fs::file_kind::FileKind;
use core::system::ports::ipc::IpcCommand;
use di::AppContainer;
use divan::counter::{BytesCount, ItemsCount};
use divan::{AllocProfiler, Bencher};
use tempfile::TempDir;
use ui::features::change_applier::state::ChangeApplierState;
use ui::features::context_packer::components::file_tree::FlatTreeProjection;
use ui::features::context_packer::state::ContextPackerState;
use ui::features::main_window::state::{MainWindowState, WorkspaceTab};
use ui::shared::containers::virtual_text::VirtualTextBuffer;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

fn generate_50k_project() -> Project {
    let entries: Vec<FileEntry> = (0..50_000)
        .map(|i| {
            FileEntry::new(
                format!("crates/crate_{:02}/src/mod_{:02}/file_{i:05}.rs", i % 25, i % 10),
                4096,
                FileKind::SourceCode,
            )
        })
        .collect();
    Project::new(PathBuf::from("/workspace"), entries, Preset::default(), Vec::new())
}

// 1. Построение плоской проекции FlatTreeProjection и расчёт TriState на дереве из 50 000 файлов.
#[divan::bench(sample_count = 40)]
fn bench_1_build_flat_tree_50k_files(bencher: Bencher) {
    let project = generate_50k_project();
    let collapsed = HashSet::new();

    bencher.counter(ItemsCount::new(50_000_usize)).bench_local(|| {
        let proj =
            FlatTreeProjection::build(divan::black_box(&project), divan::black_box(&collapsed), "");
        assert!(proj.visible_nodes().len() > 50_000);
        divan::black_box(proj)
    });
}

// 2. Сценарий A2: Переключение чекбокса директории на 50 000 файлах + пересчёт FlatTreeProjection.
#[divan::bench(sample_count = 60)]
fn bench_2_toggle_dir_checkbox_and_reproject_50k(bencher: Bencher) {
    let mut project = generate_50k_project();
    let mut collapsed = HashSet::new();
    for c in 1..25 {
        collapsed.insert(format!("crates/crate_{c:02}"));
    }

    bencher.counter(ItemsCount::new(50_000_usize)).bench_local(|| {
        let off = project.set_directory_selected("crates/crate_00", false);
        let proj_off = FlatTreeProjection::build(&project, &collapsed, "");
        let on = project.set_directory_selected("crates/crate_00", true);
        let proj_on = FlatTreeProjection::build(&project, &collapsed, "");
        assert_eq!(off + on, 4_000);
        divan::black_box((proj_off.selected_files(), proj_on.selected_files()))
    });
}

// 3. Индексация байтовых смещений строк line_offsets для 4 МБ упакованного текста (~56 000 строк).
#[divan::bench(sample_count = 50)]
fn bench_3_virtual_text_index_4mb(bencher: Bencher) {
    let line = "pub fn compute_fast(v: u64) -> u64 { v.wrapping_mul(0x517cc1b727220a95) }\n";
    let text_4mb: std::sync::Arc<str> = line.repeat(56_000).into();
    let bytes_len = text_4mb.len();

    bencher.counter(BytesCount::new(bytes_len)).bench_local(|| {
        let buf = VirtualTextBuffer::new(divan::black_box(text_4mb.clone()));
        assert_eq!(buf.line_count(), 56_001);
        divan::black_box(buf)
    });
}

// 4. Рендер видимого вьюпорта (80 строк) из 4 МБ VirtualTextBuffer (0 аллокаций кучи).
#[divan::bench(args = [0, 28_000, 55_900], sample_count = 200)]
fn bench_4_virtual_text_viewport_slice_80_rows(bencher: Bencher, start_row: usize) {
    let line = "pub fn compute_fast(v: u64) -> u64 { v.wrapping_mul(0x517cc1b727220a95) }\n";
    let buffer = VirtualTextBuffer::new(line.repeat(56_000));

    bencher.counter(ItemsCount::new(80_usize)).bench_local(|| {
        let mut sum_bytes = 0_usize;
        for row in start_row..(start_row + 80) {
            let (slice, _) = buffer.display_line(divan::black_box(row));
            sum_bytes += slice.len();
        }
        divan::black_box(sum_bytes)
    });
}

// 5. Полная пересборка 4 МБ Live Preview из кэша в памяти при переключении чекбокса папки.
#[divan::bench(sample_count = 30)]
fn bench_5_in_memory_live_preview_repack_4mb(bencher: Bencher) {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("crates/a")).unwrap();
    fs::create_dir_all(root.join("crates/b")).unwrap();

    let chunk_4kb = "pub fn item(x: u64) -> u64 { x ^ 0x55 }\n".repeat(100);
    for i in 0..500 {
        fs::write(root.join(format!("crates/a/f_{i}.rs")), &chunk_4kb).unwrap();
        fs::write(root.join(format!("crates/b/f_{i}.rs")), &chunk_4kb).unwrap();
    }

    let container = AppContainer::for_testing(global_cfg);
    let mut state = ContextPackerState::new(container, Some(root));

    bencher.counter(BytesCount::new(4_000_000_usize)).bench_local(|| {
        state.set_directory_selected("crates/b", false);
        state.set_directory_selected("crates/b", true);
        divan::black_box(state.preview_buffer().total_bytes())
    });
}

// 6. Диспетчеризация внешних событий глобального хоткея (с дедупликацией буфера обмена) и Single-Instance IPC.
#[divan::bench(args = ["hotkey_dedup_clipboard", "ipc_focus_ping"], sample_count = 100)]
fn bench_6_external_ipc_and_hotkey_state_dispatch(bencher: Bencher, event_kind: &str) {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn demo() {}\n").unwrap();

    let container = AppContainer::for_testing(global_cfg);
    let ai_payload = concat!(
        "=== ",
        "FILE: src/lib.rs ===\n",
        "pub fn demo() -> u32 { 42 }\n",
        "=== ",
        "END FILE ===\n"
    );
    container.clipboard().copy_text(ai_payload).unwrap();

    let mut main_state = MainWindowState::new();
    main_state.set_active_tab(WorkspaceTab::ChangeApplier);
    let mut packer = ContextPackerState::new(container.clone(), Some(root.clone()));
    let mut applier = ChangeApplierState::new(container.clone(), Some(root));
    applier.load_response_from_clipboard();

    bencher.counter(ItemsCount::new(1_usize)).bench_local(|| {
        if event_kind == "hotkey_dedup_clipboard" {
            container.hotkey().simulate_trigger();
        } else {
            container.ipc().push_simulated_command(IpcCommand::FocusWindow);
        }
        let processed = main_state.poll_system_events(&container, &mut packer, &mut applier, true);
        assert_eq!(processed, 1);
        divan::black_box(applier.parsed_files().len())
    });
}
