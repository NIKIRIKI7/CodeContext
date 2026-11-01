#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::wildcard_imports,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use config::LayeredConfigLoader;
use core::packing::ports::clipboard::ClipboardPort as _;
use core::packing::services::pack_service::PackService;
use core::preset::models::preset::PROJECT_CONFIG_FILE_NAME;
use core::preset::services::preset_service::PresetService;
use core::shared_kernel::execution::cancel_token::CancelToken;
use core::system::ports::hotkey::{
    GlobalHotkeyPort as _, HotkeyKey, HotkeyModifiers, HotkeyRegistrationStatus, HotkeyShortcut,
};
use core::system::ports::ipc::{
    IpcCommand, MAX_IPC_FRAME_BYTES, PrimaryElectionResult, SingleInstanceIpcPort as _,
};
use core::system::ports::shell::{ShellIntegrationPort, ShellMenuConfig, ShellRegistrationStatus};
use platform::{
    InMemoryClipboard, IpcEndpointConfig, LinuxShellImp, MacOsShellImp, SystemGlobalHotkey,
    SystemSingleInstanceIpc, WindowsShellImp, build_linux_xdg_manifest,
    build_macos_workflow_manifest, build_windows_registry_manifest, decode_ipc_frame,
    encode_ipc_frame, resolve_headless_target_path, sanitize_executable_path,
    sanitize_explorer_path_arg,
};
use proptest::prelude::*;
use scanner::IgnoreFileScanner;
use tempfile::TempDir;

#[test]
fn test_1_windows_manifest_contains_safe_explorer_percent_v_dot_and_icons() {
    let cfg = ShellMenuConfig::with_executable(PathBuf::from(
        r"\\?\C:\Users\mcniki\Проекты\CodeContext\target\release\app.exe",
    ));
    let manifest = build_windows_registry_manifest("Software\\Classes", &cfg);

    assert_eq!(
        manifest.expected_pack_dir_command,
        r#""C:\Users\mcniki\Проекты\CodeContext\target\release\app.exe" --pack-headless "%V\.""#,
        "Должен быть удалён UNC-префикс \\\\?\\ и добавлена защита \"%V\\.\" от экранирования кавычки"
    );
    assert_eq!(
        manifest.expected_apply_dir_command,
        r#""C:\Users\mcniki\Проекты\CodeContext\target\release\app.exe" --apply-headless "%V\.""#,
    );
    assert_eq!(manifest.keys.len(), 21);
    assert_eq!(manifest.root_keys_to_delete.len(), 8);
    assert!(
        manifest.keys[0].named_values.iter().any(|(k, v)| k == "SubCommands" && v.is_empty()),
        "Корневой ключ обязан содержать пустой SubCommands для выпадающего списка Проводника"
    );
}

#[test]
fn test_2_cross_platform_state_machine_register_stale_repair_and_unregister() {
    let temp = TempDir::new().unwrap();
    let macos_dir = temp.path().join("Library/Services");
    let linux_dir = temp.path().join("local/share");

    let win_shell = WindowsShellImp::new_in_memory();
    let mac_shell = MacOsShellImp::with_services_dir(macos_dir.clone());
    let lin_shell = LinuxShellImp::with_data_dir(linux_dir.clone());

    let old_cfg = ShellMenuConfig::with_executable(PathBuf::from("/opt/old_v1/codecontext"));
    let new_cfg = ShellMenuConfig::with_executable(PathBuf::from("/opt/new_v2/codecontext"));

    let adapters: Vec<&dyn ShellIntegrationPort> = vec![&win_shell, &mac_shell, &lin_shell];

    for adapter in adapters {
        assert_eq!(adapter.status_for_config(&old_cfg), ShellRegistrationStatus::NotRegistered);

        adapter.register_with_config(&old_cfg).unwrap();
        adapter.register_with_config(&old_cfg).unwrap();
        assert_eq!(adapter.status_for_config(&old_cfg), ShellRegistrationStatus::RegisteredHealthy);

        let stale_status = adapter.status_for_config(&new_cfg);
        assert!(
            stale_status.is_stale(),
            "Ожидался статус StaleExecutablePath при смене пути бинарника, получено: {stale_status:?}"
        );

        adapter.register_with_config(&new_cfg).unwrap();
        assert_eq!(adapter.status_for_config(&new_cfg), ShellRegistrationStatus::RegisteredHealthy);

        adapter.unregister_context_menu().unwrap();
        adapter.unregister_context_menu().unwrap();
        assert_eq!(adapter.status_for_config(&new_cfg), ShellRegistrationStatus::NotRegistered);
    }

    assert!(!macos_dir.join("CodeContext Copy.workflow").exists());
    assert!(!linux_dir.join("kio/servicemenus/codecontext.desktop").exists());
}

#[cfg(windows)]
#[test]
fn test_3_real_windows_hkcu_registry_sandboxed_roundtrip() {
    struct HkcuTestGuard {
        sandbox_prefix: String,
    }

    impl Drop for HkcuTestGuard {
        fn drop(&mut self) {
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            let _ = hkcu.delete_subkey_all(&self.sandbox_prefix);
        }
    }

    let pid = std::process::id();
    let sandbox_prefix = format!("Software\\CodeContext_Test_Sandbox_{pid}");
    let _guard = HkcuTestGuard { sandbox_prefix: sandbox_prefix.clone() };

    let shell = WindowsShellImp::with_hkcu_prefix(&sandbox_prefix);
    let cfg_v1 = ShellMenuConfig::with_executable(PathBuf::from(r"C:\Apps\v1\codecontext.exe"));
    let cfg_v2 = ShellMenuConfig::with_executable(PathBuf::from(r"C:\Apps\v2\codecontext.exe"));

    assert_eq!(shell.status_for_config(&cfg_v1), ShellRegistrationStatus::NotRegistered);

    shell.register_with_config(&cfg_v1).unwrap();
    assert_eq!(shell.status_for_config(&cfg_v1), ShellRegistrationStatus::RegisteredHealthy);
    assert!(shell.status_for_config(&cfg_v2).is_stale());

    shell.register_with_config(&cfg_v2).unwrap();
    assert_eq!(shell.status_for_config(&cfg_v2), ShellRegistrationStatus::RegisteredHealthy);

    shell.unregister_context_menu().unwrap();
    assert_eq!(shell.status_for_config(&cfg_v2), ShellRegistrationStatus::NotRegistered);
}

#[test]
fn test_4_headless_fast_path_packs_when_invoked_on_file_or_directory_and_blocks_secrets() {
    let temp = TempDir::new().unwrap();
    let global_cfg = temp.path().join("global/config.json");
    let repo_root = temp.path().join("MyRepo");
    fs::create_dir_all(repo_root.join("crates/core/src")).unwrap();

    fs::write(
        repo_root.join(PROJECT_CONFIG_FILE_NAME),
        r#"{"schema_version": 1, "explicitly_allowed_secrets": [".env"]}"#,
    )
    .unwrap();
    fs::write(repo_root.join(".env"), "AWS_SECRET=top_secret_value\n").unwrap();
    let deep_file = repo_root.join("crates/core/src/lib.rs");
    fs::write(&deep_file, "pub fn headless_works() -> bool { true }\n").unwrap();

    let explorer_arg = format!("\"{}\\.\"", deep_file.display());
    let resolved_root = resolve_headless_target_path(&explorer_arg);
    assert_eq!(resolved_root, repo_root);

    let preset_service =
        PresetService::new(LayeredConfigLoader::with_global_config_path(global_cfg));
    let clipboard = InMemoryClipboard::new();
    let pack_service = PackService::new(IgnoreFileScanner::new(), clipboard.clone());

    let (preset, from_untrusted, _warnings) = preset_service.resolve_headless(&resolved_root);
    assert!(from_untrusted);

    let res = pack_service
        .pack_headless(&resolved_root, preset, from_untrusted, &CancelToken::new())
        .unwrap();

    assert_eq!(res.packed_files_count, 2);
    assert_eq!(res.blocked_secrets_count, 1);
    let clip_text = clipboard.read_text().unwrap();
    assert!(clip_text.contains("pub fn headless_works()"));
    assert!(!clip_text.contains("top_secret_value"));
}

#[test]
fn test_5_ipc_leader_follower_handoff_roundtrip() {
    let temp = TempDir::new().unwrap();
    let cfg = IpcEndpointConfig::for_testing(temp.path(), "handoff");

    let primary = SystemSingleInstanceIpc::with_endpoint_config(cfg.clone());
    let res1 = primary.ensure_primary_or_forward(None).unwrap();
    assert_eq!(res1, PrimaryElectionResult::AcquiredPrimary { stale_socket_recovered: false });

    let secondary = SystemSingleInstanceIpc::with_endpoint_config(cfg);
    let explorer_target = PathBuf::from(r#""\\?\C:\Users\mcniki\Проекты\RustApp\.""#);
    let res2 = secondary.ensure_primary_or_forward(Some(explorer_target)).unwrap();
    assert_eq!(res2, PrimaryElectionResult::ForwardedToPrimary);

    let received =
        primary.try_recv_command().expect("Primary обязан получить команду от Secondary");
    assert_eq!(
        received,
        IpcCommand::OpenProjectAndFocus {
            project_root: PathBuf::from(r"C:\Users\mcniki\Проекты\RustApp"),
        }
    );
}

#[test]
fn test_6_ipc_concurrent_spawn_race_condition_10_threads() {
    let temp = TempDir::new().unwrap();
    let cfg = IpcEndpointConfig::for_testing(temp.path(), "race10");
    let thread_count = 10_usize;
    let barrier = Arc::new(Barrier::new(thread_count));

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let b = Arc::clone(&barrier);
        let c = cfg.clone();
        handles.push(thread::spawn(move || {
            let ipc = SystemSingleInstanceIpc::with_endpoint_config(c);
            b.wait();
            let res = ipc
                .ensure_primary_or_forward(Some(PathBuf::from(format!("/workspace/proj_{idx}"))))
                .unwrap();
            (res, ipc)
        }));
    }

    let mut primaries = Vec::new();
    let mut forwarded_count = 0_usize;
    for h in handles {
        let (res, ipc) = h.join().unwrap();
        match res {
            PrimaryElectionResult::AcquiredPrimary { .. } => primaries.push(ipc),
            PrimaryElectionResult::ForwardedToPrimary => forwarded_count += 1,
        }
    }

    assert_eq!(primaries.len(), 1, "Ровно 1 поток должен стать Primary-сервером");
    assert_eq!(forwarded_count, 9, "Остальные 9 потоков обязаны стать Secondary-клиентами");

    let primary = &primaries[0];
    let deadline = Instant::now() + Duration::from_millis(500);
    let mut received_paths = Vec::new();
    while received_paths.len() < 9 && Instant::now() < deadline {
        if let Some(IpcCommand::OpenProjectAndFocus { project_root }) = primary.try_recv_command() {
            received_paths.push(project_root);
        } else {
            thread::sleep(Duration::from_millis(2));
        }
    }
    assert_eq!(received_paths.len(), 9);
}

#[test]
fn test_7_ipc_stale_socket_recovery() {
    let temp = TempDir::new().unwrap();
    let cfg = IpcEndpointConfig::for_testing(temp.path(), "stale_recovery");

    fs::write(&cfg.socket_file_path, b"dead_socket_from_sigkill").unwrap();
    assert!(cfg.socket_file_path.exists());

    let ipc = SystemSingleInstanceIpc::with_endpoint_config(cfg);
    let election = ipc.ensure_primary_or_forward(None).unwrap();
    assert_eq!(election, PrimaryElectionResult::AcquiredPrimary { stale_socket_recovered: true });
}

#[test]
fn test_8_ipc_oversized_and_malformed_payload_protection() {
    let temp = TempDir::new().unwrap();
    let cfg = IpcEndpointConfig::for_testing(temp.path(), "malformed_guard");

    let primary = SystemSingleInstanceIpc::with_endpoint_config(cfg.clone());
    primary.ensure_primary_or_forward(None).unwrap();

    let _ = primary.send_raw_bytes_for_testing(b"XX\x01\x01\x00\x05hello");
    let _ = primary.send_raw_bytes_for_testing(b"CC\x01\x01\xEA\x60overflow");

    let client = SystemSingleInstanceIpc::with_endpoint_config(cfg);
    let ok = client
        .try_send_project_path_to_existing_instance(PathBuf::from("/workspace/healthy_repo"))
        .unwrap();
    assert!(ok);

    let cmd = primary.try_recv_command().unwrap();
    assert_eq!(
        cmd,
        IpcCommand::OpenProjectAndFocus { project_root: PathBuf::from("/workspace/healthy_repo") }
    );
}

#[test]
fn test_9_hotkey_lifecycle_register_rebind_conflict_and_wayland_degradation() {
    let hotkey = SystemGlobalHotkey::new_in_memory();
    assert_eq!(hotkey.status(), HotkeyRegistrationStatus::NotRegistered);

    let st = hotkey.register_str("ctrl + shift + c").unwrap();
    assert_eq!(
        st,
        HotkeyRegistrationStatus::RegisteredActive { shortcut: Arc::from("Ctrl+Shift+C") }
    );

    let conflict_target =
        HotkeyShortcut::new(HotkeyModifiers::CTRL.with(HotkeyModifiers::ALT), HotkeyKey::Char('K'));
    hotkey.simulate_conflict(conflict_target);
    let conflict_res = hotkey.rebind_str("Ctrl+Alt+K");
    assert!(conflict_res.is_err());
    assert!(hotkey.status().is_conflict());

    let rebound = hotkey.rebind_str("Ctrl+Shift+P").unwrap();
    assert_eq!(
        rebound,
        HotkeyRegistrationStatus::RegisteredActive { shortcut: Arc::from("Ctrl+Shift+P") }
    );

    hotkey.simulate_trigger();
    assert!(hotkey.try_recv_hotkey_trigger());
    assert!(!hotkey.try_recv_hotkey_trigger());

    hotkey.unregister().unwrap();
    hotkey.unregister().unwrap();
    assert_eq!(hotkey.status(), HotkeyRegistrationStatus::NotRegistered);

    let wayland_hk = SystemGlobalHotkey::new_simulated_wayland_restricted();
    assert!(wayland_hk.register_str("Ctrl+Shift+C").is_err());
    assert!(wayland_hk.status().is_unsupported_display());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn prop_1_explorer_path_sanitizer_handles_quotes_unc_and_trailing_dots(
        drive in "[A-Z]",
        segments in prop::collection::vec("[a-zA-Z0-9_ А-Яа-я]{1,12}", 1..5),
        wrap_quotes in any::<bool>(),
        add_unc in any::<bool>(),
        add_trailing_dot in any::<bool>(),
    ) {
        let clean_segments: Vec<&str> = segments
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let joined = if clean_segments.is_empty() {
            "Project".to_owned()
        } else {
            clean_segments.join("\\")
        };

        let base = format!("{drive}:\\{joined}");
        let mut raw = base.clone();
        if add_unc {
            raw = format!("\\\\?\\{raw}");
        }
        if add_trailing_dot {
            raw.push_str("\\.");
        }
        if wrap_quotes {
            raw = format!("\"{raw}\"");
        }

        let sanitized = sanitize_explorer_path_arg(&raw);
        prop_assert_eq!(sanitized, PathBuf::from(&base));
    }

    #[test]
    fn prop_2_manifest_builders_never_panic_on_arbitrary_exe_and_labels(
        exe_str in ".*",
        pack_label in ".*",
        open_label in ".*",
    ) {
        let cfg = ShellMenuConfig {
            executable_path: PathBuf::from(exe_str),
            root_menu_label: "CodeContext".to_owned(),
            pack_menu_label: pack_label.clone(),
            apply_menu_label: pack_label,
            open_gui_label: open_label,
        };
        let clean_exe = sanitize_executable_path(&cfg.executable_path);
        prop_assert!(!clean_exe.contains('"'));
        prop_assert!(!clean_exe.starts_with("\\\\?\\"));

        let win = build_windows_registry_manifest("Software\\Classes", &cfg);
        let mac = build_macos_workflow_manifest(&cfg);
        let lin = build_linux_xdg_manifest(&cfg);

        prop_assert_eq!(win.keys.len(), 21);
        prop_assert_eq!(mac.artifacts.len(), 2);
        prop_assert_eq!(lin.artifacts.len(), 3);
    }

    #[test]
    fn prop_3_hotkey_parser_never_panics_and_roundtrips_canonical_format(
        random_garbage in ".*",
        use_ctrl in any::<bool>(),
        use_alt in any::<bool>(),
        use_shift in any::<bool>(),
        use_super in any::<bool>(),
        letter_idx in 0_u8..26_u8,
    ) {
        let _ = HotkeyShortcut::parse(&random_garbage);

        let mut mods = HotkeyModifiers::empty();
        if use_ctrl { mods = mods.with(HotkeyModifiers::CTRL); }
        if use_alt { mods = mods.with(HotkeyModifiers::ALT); }
        if use_shift { mods = mods.with(HotkeyModifiers::SHIFT); }
        if use_super { mods = mods.with(HotkeyModifiers::SUPER); }
        if mods.is_empty() {
            mods = HotkeyModifiers::CTRL;
        }

        let ch = (b'A' + letter_idx) as char;
        let expected = HotkeyShortcut::new(mods, HotkeyKey::Char(ch));
        let canonical = expected.to_canonical_string();
        let reparsed = HotkeyShortcut::parse(&canonical).unwrap();
        prop_assert_eq!(reparsed, expected);
    }

    #[test]
    fn prop_4_ipc_frame_codec_roundtrip_and_arbitrary_bytes_safety(
        valid_path in "[a-zA-Z0-9_./\\\\:-]{1,256}",
        arbitrary_bytes in prop::collection::vec(any::<u8>(), 0..512),
    ) {
        let _ = decode_ipc_frame(&arbitrary_bytes);

        let cmd = IpcCommand::OpenProjectAndFocus {
            project_root: PathBuf::from(&valid_path),
        };
        let mut stack_buf = [0_u8; MAX_IPC_FRAME_BYTES];
        let written = encode_ipc_frame(&cmd, &mut stack_buf).unwrap();
        let decoded = decode_ipc_frame(&stack_buf[..written]).unwrap();
        prop_assert_eq!(decoded, cmd);
    }
}
