use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use core::system::ports::shell::{
    ShellIntegrationError, ShellIntegrationPort, ShellMenuConfig, ShellRegistrationStatus,
};

use super::manifest::{
    FileManifest, InMemoryFileStore, LINUX_PRIMARY_DESKTOP_REL_PATH, build_linux_xdg_manifest,
    evaluate_command_status, expected_unix_pack_marker,
};

#[derive(Debug, Clone)]
enum LinuxStorageBackend {
    InMemory(Arc<Mutex<InMemoryFileStore>>),
    FileSystem { data_root: PathBuf },
}

impl Default for LinuxStorageBackend {
    fn default() -> Self {
        if cfg!(target_os = "linux")
            && let Some(data_dir) = dirs::data_local_dir()
        {
            return Self::FileSystem { data_root: data_dir };
        }
        Self::InMemory(Arc::new(Mutex::new(InMemoryFileStore::new())))
    }
}

#[derive(Debug, Clone, Default)]
pub struct LinuxShellImp {
    backend: LinuxStorageBackend,
}

impl LinuxShellImp {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        Self {
            backend: LinuxStorageBackend::InMemory(Arc::new(Mutex::new(InMemoryFileStore::new()))),
        }
    }

    #[inline]
    pub const fn with_data_dir(data_root: PathBuf) -> Self {
        Self { backend: LinuxStorageBackend::FileSystem { data_root } }
    }

    #[inline]
    pub fn build_manifest(config: &ShellMenuConfig) -> FileManifest {
        build_linux_xdg_manifest(config)
    }
}

impl ShellIntegrationPort for LinuxShellImp {
    fn status(&self) -> ShellRegistrationStatus {
        self.status_for_config(&ShellMenuConfig::default())
    }

    fn status_for_config(&self, config: &ShellMenuConfig) -> ShellRegistrationStatus {
        let expected_marker = expected_unix_pack_marker(config);
        match &self.backend {
            LinuxStorageBackend::InMemory(store) => {
                let Ok(guard) = store.lock() else {
                    return ShellRegistrationStatus::NotRegistered;
                };
                evaluate_command_status(guard.get(LINUX_PRIMARY_DESKTOP_REL_PATH), &expected_marker)
            }
            LinuxStorageBackend::FileSystem { data_root } => {
                let target = data_root.join(LINUX_PRIMARY_DESKTOP_REL_PATH);
                let content_opt = fs::read_to_string(target).ok();
                evaluate_command_status(content_opt.as_deref(), &expected_marker)
            }
        }
    }

    #[inline]
    fn is_registered(&self) -> bool {
        self.status().is_registered()
    }

    fn register_context_menu(&self) -> Result<(), ShellIntegrationError> {
        self.register_with_config(&ShellMenuConfig::default())
    }

    fn register_with_config(&self, config: &ShellMenuConfig) -> Result<(), ShellIntegrationError> {
        let manifest = Self::build_manifest(config);
        match &self.backend {
            LinuxStorageBackend::InMemory(store) => {
                let mut guard = store
                    .lock()
                    .map_err(|e| ShellIntegrationError::RegistrationFailed(e.to_string()))?;
                guard.apply_manifest(&manifest);
                Ok(())
            }
            LinuxStorageBackend::FileSystem { data_root } => {
                for artifact in &manifest.artifacts {
                    let full_path = data_root.join(&artifact.rel_path);
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            ShellIntegrationError::RegistrationFailed(e.to_string())
                        })?;
                    }
                    fs::write(&full_path, &artifact.content)
                        .map_err(|e| ShellIntegrationError::RegistrationFailed(e.to_string()))?;
                    #[cfg(unix)]
                    if artifact.executable {
                        use std::os::unix::fs::PermissionsExt as _;
                        let perms = fs::Permissions::from_mode(0o755);
                        let _ = fs::set_permissions(&full_path, perms);
                    }
                }
                Ok(())
            }
        }
    }

    fn unregister_context_menu(&self) -> Result<(), ShellIntegrationError> {
        let manifest = Self::build_manifest(&ShellMenuConfig::default());
        match &self.backend {
            LinuxStorageBackend::InMemory(store) => {
                let mut guard = store
                    .lock()
                    .map_err(|e| ShellIntegrationError::UnregistrationFailed(e.to_string()))?;
                guard.remove_manifest(&manifest);
                Ok(())
            }
            LinuxStorageBackend::FileSystem { data_root } => {
                for rel_file in &manifest.root_entries_to_delete {
                    let target = data_root.join(rel_file);
                    if target.exists() {
                        fs::remove_file(&target).map_err(|e| {
                            ShellIntegrationError::UnregistrationFailed(e.to_string())
                        })?;
                    }
                }
                Ok(())
            }
        }
    }
}
