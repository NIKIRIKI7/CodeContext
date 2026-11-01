mod desktop;
mod lifecycle;
mod shell;

pub use desktop::clipboard::{InMemoryClipboard, SystemClipboard};
pub use desktop::hotkey::{SystemGlobalHotkey, to_native_hotkey};
pub use lifecycle::ipc::{
    IpcEndpointConfig, SystemSingleInstanceIpc, decode_ipc_frame, decode_ipc_frame_ref,
    decode_ipc_json_fallback, default_ipc_endpoint_config, encode_ipc_frame,
    encode_ipc_json_fallback,
};
pub use lifecycle::updater::{
    SystemUpdater, atomic_swap_executable, canonical_install_binary_path, compute_sha256_hex,
    current_target_triple, parse_forge_releases_tags, sign_release_payload,
    verify_payload_integrity, verify_release_signature,
};
pub use shell::{
    FileManifest, LinuxShellImp, MacOsShellImp, RegistryKeySpec, ShellFileArtifact,
    SystemShellIntegration, WINDOWS_DEFAULT_CLASSES_PREFIX, WindowsRegistryManifest,
    WindowsShellImp, build_linux_xdg_manifest, build_macos_workflow_manifest,
    build_windows_registry_manifest, resolve_headless_target_path, sanitize_executable_path,
    sanitize_explorer_path_arg,
};
