#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use config::LayeredConfigLoader;
use core::packing::services::pack_service::PackService;
use core::preset::models::preset::PROJECT_CONFIG_FILE_NAME;
use core::preset::services::preset_service::PresetService;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::system::ports::hotkey::{GlobalHotkeyPort, HotkeyShortcut};
use core::system::ports::ipc::{
    IpcCommand, MAX_IPC_FRAME_BYTES, PrimaryElectionResult, SingleInstanceIpcPort,
};
use core::system::ports::shell::{ShellIntegrationPort, ShellMenuConfig};
use divan::counter::{BytesCount, ItemsCount};
use divan::{AllocProfiler, Bencher};
use platform::{
    InMemoryClipboard, IpcEndpointConfig, SystemGlobalHotkey, SystemSingleInstanceIpc,
    WindowsShellImp, build_linux_xdg_manifest, build_macos_workflow_manifest,
    build_windows_registry_manifest, decode_ipc_frame, decode_ipc_frame_ref,
    decode_ipc_json_fallback, encode_ipc_frame, encode_ipc_json_fallback,
    resolve_headless_target_path, sanitize_explorer_path_arg, to_native_hotkey,
};
use scanner::IgnoreFileScanner;
use tempfile::TempDir;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

static BENCH_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    divan::main();
}

// 1. Главный продуктовый SLA Сценария A1: холодный запуск Headless-упаковки 4 МБ в буфер обмена.
#[divan::bench(sample_count = 30)]
fn bench_1_headless_cold_to_clipboard_4mb(bencher: Bencher) {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join(".gitignore"), "target/\nnode_modules/\n").unwrap();
    fs::write(root.join(PROJECT_CONFIG_FILE_NAME), r#"{"schema_version": 1}"#).unwrap();

    let chunk_4kb =
        "pub fn fast(x: u64) -> u64 { x.wrapping_mul(0x517cc1b727220a95) }\n".repeat(62);
    for i in 0..1_000 {
        fs::write(root.join(format!("src/mod_{i}.rs")), &chunk_4kb).unwrap();
    }

    let preset_service =
        PresetService::new(LayeredConfigLoader::with_global_config_path(global_cfg));
    let clipboard = InMemoryClipboard::new();
    let pack_service = PackService::new(IgnoreFileScanner::new(), clipboard);
    let cancel_token = CancelToken::new();

    bencher.counter(BytesCount::new(4_000_000_usize)).bench_local(|| {
        let target = resolve_headless_target_path(divan::black_box(&root.to_string_lossy()));
        let (preset, from_untrusted, _) = preset_service.resolve_headless(&target);
        let res =
            pack_service.pack_headless(&target, preset, from_untrusted, &cancel_token).unwrap();
        assert!(res.total_bytes > 3_800_000);
        divan::black_box(res.total_bytes)
    });
}

// 2. Санитизация 10 000 аргументов командной строки Проводника Windows.
#[divan::bench(sample_count = 100)]
fn bench_2_explorer_path_sanitization_10k(bencher: Bencher) {
    let raw_args: Vec<String> = (0..10_000)
        .map(|i| match i % 4 {
            0 => format!(r#""\\?\C:\Users\mcniki\Documents\Проекты\Rust\Repo_{i}\.""#),
            1 => format!(r#""C:\Workspace\Service_{i}\src\main.rs""#),
            2 => r#""C:\.""#.to_owned(),
            _ => format!("/home/dev/projects/repo_{i}/."),
        })
        .collect();

    bencher.counter(ItemsCount::new(raw_args.len())).bench_local(|| {
        let mut count = 0_usize;
        for arg in divan::black_box(&raw_args) {
            let p = sanitize_explorer_path_arg(arg);
            if !p.as_os_str().is_empty() {
                count += 1;
            }
        }
        assert_eq!(count, 10_000);
        divan::black_box(count)
    });
}

// 3. Генерация декларативных манифестов контекстного меню для Windows, macOS и Linux.
#[divan::bench(args = ["windows_hkcu", "macos_workflow", "linux_xdg"], sample_count = 100)]
fn bench_3_shell_manifest_generation_all_os(bencher: Bencher, os_kind: &str) {
    let cfg = ShellMenuConfig::with_executable(PathBuf::from(
        r"C:\Program Files\CodeContext\codecontext.exe",
    ));

    bencher.bench_local(|| match os_kind {
        "macos_workflow" => {
            let m = build_macos_workflow_manifest(divan::black_box(&cfg));
            divan::black_box(m.artifacts.len())
        }
        "linux_xdg" => {
            let m = build_linux_xdg_manifest(divan::black_box(&cfg));
            divan::black_box(m.artifacts.len())
        }
        _ => {
            let m = build_windows_registry_manifest("Software\\Classes", divan::black_box(&cfg));
            divan::black_box(m.keys.len())
        }
    });
}

// 4. Проверка статуса регистрации контекстного меню для UI настроек.
#[divan::bench(sample_count = 100)]
fn bench_4_shell_status_check_latency(bencher: Bencher) {
    let shell = WindowsShellImp::new_in_memory();
    let cfg = ShellMenuConfig::with_executable(PathBuf::from(
        r"C:\Program Files\CodeContext\codecontext.exe",
    ));
    shell.register_with_config(&cfg).unwrap();

    bencher.bench_local(|| {
        let st = shell.status_for_config(divan::black_box(&cfg));
        assert!(st.is_registered());
        divan::black_box(st)
    });
}

// 5. Парсинг и нормализация 10 000 комбинаций глобального хоткея (0 аллокаций кучи).
#[divan::bench(
    args = ["canonical_ctrl_shift_c", "alias_cmdorctrl", "four_modifiers_f12", "invalid_fast_reject"],
    sample_count = 100
)]
fn bench_5_hotkey_parsing_and_normalization(bencher: Bencher, input_kind: &str) {
    let raw = match input_kind {
        "alias_cmdorctrl" => "  cmdorctrl + shift + keyc ",
        "four_modifiers_f12" => "Ctrl+Alt+Shift+Super+F12",
        "invalid_fast_reject" => "Ctrl+UnknownKey",
        _ => "Ctrl+Shift+C",
    };

    bencher.counter(ItemsCount::new(10_000_usize)).bench_local(|| {
        let mut valid_count = 0_usize;
        for _ in 0..10_000 {
            if let Ok(hk) = HotkeyShortcut::parse(divan::black_box(raw)) {
                let native = to_native_hotkey(hk);
                if native.id() != 0 {
                    valid_count += 1;
                }
            }
        }
        divan::black_box(valid_count)
    });
}

// 6. Сравнение Zero-Copy стекового фрейминга IPC, Owned-фрейминга и JSON-фрейминга.
#[divan::bench(
    args = ["binary_zero_copy_ref", "binary_stack_frame", "json_fallback"],
    sample_count = 100
)]
fn bench_6_ipc_wire_protocol_framing(bencher: Bencher, codec_kind: &str) {
    let cmd = IpcCommand::OpenProjectAndFocus {
        project_root: PathBuf::from(
            "/workspace/Users/mcniki/Проекты/Rust/CodeContext/crates/platform",
        ),
    };
    let mut stack_buf = [0_u8; MAX_IPC_FRAME_BYTES];

    bencher.counter(ItemsCount::new(1_000_usize)).bench_local(|| match codec_kind {
        "binary_zero_copy_ref" => {
            let mut total_bytes = 0_usize;
            for _ in 0..1_000 {
                let len = encode_ipc_frame(divan::black_box(&cmd), &mut stack_buf).unwrap();
                let decoded_ref = decode_ipc_frame_ref(&stack_buf[..len]).unwrap();
                total_bytes += len;
                divan::black_box(decoded_ref);
            }
            divan::black_box(total_bytes)
        }
        "binary_stack_frame" => {
            let mut total_bytes = 0_usize;
            for _ in 0..1_000 {
                let len = encode_ipc_frame(divan::black_box(&cmd), &mut stack_buf).unwrap();
                let decoded = decode_ipc_frame(&stack_buf[..len]).unwrap();
                total_bytes += len;
                divan::black_box(decoded);
            }
            divan::black_box(total_bytes)
        }
        _ => {
            let mut total_bytes = 0_usize;
            for _ in 0..1_000 {
                let bytes = encode_ipc_json_fallback(divan::black_box(&cmd)).unwrap();
                let decoded = decode_ipc_json_fallback(&bytes).unwrap();
                total_bytes += bytes.len();
                divan::black_box(decoded);
            }
            divan::black_box(total_bytes)
        }
    });
}

// 7. Полный Loopback-раундтрип реального локального сокета ОС.
#[divan::bench(sample_count = 40)]
fn bench_7_ipc_real_os_socket_roundtrip(bencher: Bencher) {
    let temp = TempDir::new().unwrap();
    let seq = BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let cfg = IpcEndpointConfig::for_testing(temp.path(), &format!("rt_{seq}"));

    let primary = SystemSingleInstanceIpc::with_endpoint_config(cfg.clone());
    let election = primary.ensure_primary_or_forward(None).unwrap();
    assert!(matches!(election, PrimaryElectionResult::AcquiredPrimary { .. }));

    let secondary = SystemSingleInstanceIpc::with_endpoint_config(cfg);
    let target_path = PathBuf::from("/workspace/CodeContext");

    bencher.counter(ItemsCount::new(1_usize)).bench_local(|| {
        let forwarded = secondary
            .try_send_project_path_to_existing_instance(divan::black_box(target_path.clone()))
            .unwrap();
        assert!(forwarded);
        let received = primary.try_recv_command();
        assert!(received.is_some());
        divan::black_box(received)
    });
}

// 8. Скорость холодного арбитража Primary-инстанса и восстановления после мёртвого сокета.
#[divan::bench(args = ["clean_primary_start", "stale_socket_recovery"], sample_count = 30)]
fn bench_8_ipc_election_and_stale_recovery(bencher: Bencher, mode: &str) {
    let temp = TempDir::new().unwrap();

    bencher.bench_local(|| {
        let seq = BENCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let cfg = IpcEndpointConfig::for_testing(temp.path(), &format!("el_{seq}"));
        if mode == "stale_socket_recovery" {
            fs::write(&cfg.socket_file_path, b"stale_lock").unwrap();
        }
        let ipc = SystemSingleInstanceIpc::with_endpoint_config(cfg);
        let res = ipc.ensure_primary_or_forward(None).unwrap();
        divan::black_box(res)
    });
}

// 9. Скорость перепривязки и опроса статуса глобального хоткея.
#[divan::bench(sample_count = 100)]
fn bench_9_hotkey_state_machine_rebind(bencher: Bencher) {
    let hotkey = SystemGlobalHotkey::new_in_memory();
    hotkey.register_str("Ctrl+Shift+C").unwrap();

    bencher.counter(ItemsCount::new(2_usize)).bench_local(|| {
        let s1 = hotkey.rebind_str(divan::black_box("Ctrl+Alt+C")).unwrap();
        let s2 = hotkey.rebind_str(divan::black_box("Ctrl+Shift+C")).unwrap();
        assert!(s1.is_active() && s2.is_active());
        divan::black_box(hotkey.status())
    });
}
