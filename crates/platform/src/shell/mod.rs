pub(crate) mod linux;
pub(crate) mod macos;
pub(crate) mod manifest;
pub(crate) mod windows;

use core::system::ports::shell::{
    ShellIntegrationError, ShellIntegrationPort, ShellMenuConfig, ShellRegistrationStatus,
};

pub use self::linux::LinuxShellImp;
pub use self::macos::MacOsShellImp;
pub use self::manifest::{
    FileManifest, RegistryKeySpec, ShellFileArtifact, WINDOWS_DEFAULT_CLASSES_PREFIX,
    WindowsRegistryManifest, build_linux_xdg_manifest, build_macos_workflow_manifest,
    build_windows_registry_manifest, resolve_headless_target_path, sanitize_executable_path,
    sanitize_explorer_path_arg,
};
pub use self::windows::WindowsShellImp;

#[derive(Debug, Clone)]
enum PlatformShellBridgeImp {
    Windows(WindowsShellImp),
    MacOs(MacOsShellImp),
    Linux(LinuxShellImp),
}

impl Default for PlatformShellBridgeImp {
    fn default() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows(WindowsShellImp::new())
        } else if cfg!(target_os = "macos") {
            Self::MacOs(MacOsShellImp::new())
        } else {
            Self::Linux(LinuxShellImp::new())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemShellIntegration {
    imp: PlatformShellBridgeImp,
}

impl SystemShellIntegration {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        let imp = if cfg!(target_os = "windows") {
            PlatformShellBridgeImp::Windows(WindowsShellImp::new_in_memory())
        } else if cfg!(target_os = "macos") {
            PlatformShellBridgeImp::MacOs(MacOsShellImp::new_in_memory())
        } else {
            PlatformShellBridgeImp::Linux(LinuxShellImp::new_in_memory())
        };
        Self { imp }
    }
}

impl ShellIntegrationPort for SystemShellIntegration {
    fn status(&self) -> ShellRegistrationStatus {
        match &self.imp {
            PlatformShellBridgeImp::Windows(w) => w.status(),
            PlatformShellBridgeImp::MacOs(m) => m.status(),
            PlatformShellBridgeImp::Linux(l) => l.status(),
        }
    }

    fn status_for_config(&self, config: &ShellMenuConfig) -> ShellRegistrationStatus {
        match &self.imp {
            PlatformShellBridgeImp::Windows(w) => w.status_for_config(config),
            PlatformShellBridgeImp::MacOs(m) => m.status_for_config(config),
            PlatformShellBridgeImp::Linux(l) => l.status_for_config(config),
        }
    }

    fn is_registered(&self) -> bool {
        self.status().is_registered()
    }

    fn register_context_menu(&self) -> Result<(), ShellIntegrationError> {
        match &self.imp {
            PlatformShellBridgeImp::Windows(w) => w.register_context_menu(),
            PlatformShellBridgeImp::MacOs(m) => m.register_context_menu(),
            PlatformShellBridgeImp::Linux(l) => l.register_context_menu(),
        }
    }

    fn register_with_config(&self, config: &ShellMenuConfig) -> Result<(), ShellIntegrationError> {
        match &self.imp {
            PlatformShellBridgeImp::Windows(w) => w.register_with_config(config),
            PlatformShellBridgeImp::MacOs(m) => m.register_with_config(config),
            PlatformShellBridgeImp::Linux(l) => l.register_with_config(config),
        }
    }

    fn unregister_context_menu(&self) -> Result<(), ShellIntegrationError> {
        match &self.imp {
            PlatformShellBridgeImp::Windows(w) => w.unregister_context_menu(),
            PlatformShellBridgeImp::MacOs(m) => m.unregister_context_menu(),
            PlatformShellBridgeImp::Linux(l) => l.unregister_context_menu(),
        }
    }
}
