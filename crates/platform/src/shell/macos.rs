use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use core::system::ports::shell::{
    ShellIntegrationError, ShellIntegrationPort, ShellMenuConfig, ShellRegistrationStatus,
};

use super::manifest::{
    FileManifest, InMemoryFileStore, MACOS_PRIMARY_WFLOW_REL_PATH, build_macos_workflow_manifest,
    evaluate_command_status, expected_unix_pack_marker,
};

#[derive(Debug, Clone)]
enum MacOsStorageBackend {
    InMemory(Arc<Mutex<InMemoryFileStore>>),
    FileSystem { services_root: PathBuf },
}

impl Default for MacOsStorageBackend {
    fn default() -> Self {
        if cfg!(target_os = "macos")
            && let Some(home) = dirs::home_dir()
        {
            return Self::FileSystem { services_root: home.join("Library/Services") };
        }
        Self::InMemory(Arc::new(Mutex::new(InMemoryFileStore::new())))
    }
}

#[derive(Debug, Clone, Default)]
pub struct MacOsShellImp {
    backend: MacOsStorageBackend,
}

impl MacOsShellImp {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        Self {
            backend: MacOsStorageBackend::InMemory(Arc::new(Mutex::new(InMemoryFileStore::new()))),
        }
    }

    #[inline]
    pub const fn with_services_dir(services_root: PathBuf) -> Self {
        Self { backend: MacOsStorageBackend::FileSystem { services_root } }
    }

    #[inline]
    pub fn build_manifest(config: &ShellMenuConfig) -> FileManifest {
        build_macos_workflow_manifest(config)
    }
}

impl ShellIntegrationPort for MacOsShellImp {
    fn status(&self) -> ShellRegistrationStatus {
        self.status_for_config(&ShellMenuConfig::default())
    }

    fn status_for_config(&self, config: &ShellMenuConfig) -> ShellRegistrationStatus {
        let expected_marker = expected_unix_pack_marker(config);
        match &self.backend {
            MacOsStorageBackend::InMemory(store) => {
                let Ok(guard) = store.lock() else {
                    return ShellRegistrationStatus::NotRegistered;
                };
                evaluate_command_status(guard.get(MACOS_PRIMARY_WFLOW_REL_PATH), &expected_marker)
            }
            MacOsStorageBackend::FileSystem { services_root } => {
                let target = services_root.join(MACOS_PRIMARY_WFLOW_REL_PATH);
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
            MacOsStorageBackend::InMemory(store) => {
                let mut guard = store
                    .lock()
                    .map_err(|e| ShellIntegrationError::RegistrationFailed(e.to_string()))?;
                guard.apply_manifest(&manifest);
                Ok(())
            }
            MacOsStorageBackend::FileSystem { services_root } => {
                for artifact in &manifest.artifacts {
                    let full_path = services_root.join(&artifact.rel_path);
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            ShellIntegrationError::RegistrationFailed(e.to_string())
                        })?;
                    }
                    fs::write(&full_path, &artifact.content)
                        .map_err(|e| ShellIntegrationError::RegistrationFailed(e.to_string()))?;
                }
                Ok(())
            }
        }
    }

    fn unregister_context_menu(&self) -> Result<(), ShellIntegrationError> {
        let manifest = Self::build_manifest(&ShellMenuConfig::default());
        match &self.backend {
            MacOsStorageBackend::InMemory(store) => {
                let mut guard = store
                    .lock()
                    .map_err(|e| ShellIntegrationError::UnregistrationFailed(e.to_string()))?;
                guard.remove_manifest(&manifest);
                Ok(())
            }
            MacOsStorageBackend::FileSystem { services_root } => {
                for entry in &manifest.root_entries_to_delete {
                    let target = services_root.join(entry);
                    if target.exists() {
                        fs::remove_dir_all(&target).map_err(|e| {
                            ShellIntegrationError::UnregistrationFailed(e.to_string())
                        })?;
                    }
                }
                Ok(())
            }
        }
    }
}
