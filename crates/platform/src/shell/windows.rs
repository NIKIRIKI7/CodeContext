use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use core::system::ports::shell::{
    ShellIntegrationError, ShellIntegrationPort, ShellMenuConfig, ShellRegistrationStatus,
};

use super::manifest::{
    WINDOWS_DEFAULT_CLASSES_PREFIX, WindowsRegistryManifest, build_windows_registry_manifest,
    expected_windows_apply_dir_command, expected_windows_pack_dir_command,
};

#[derive(Debug, Clone)]
enum WindowsRegistryBackend {
    InMemory {
        classes_prefix: String,
        keys: Arc<Mutex<BTreeMap<String, BTreeMap<String, String>>>>,
    },
    #[cfg(windows)]
    HkcuRegistry { classes_prefix: String },
}

impl Default for WindowsRegistryBackend {
    fn default() -> Self {
        #[cfg(windows)]
        {
            Self::HkcuRegistry { classes_prefix: WINDOWS_DEFAULT_CLASSES_PREFIX.to_owned() }
        }
        #[cfg(not(windows))]
        {
            Self::InMemory {
                classes_prefix: WINDOWS_DEFAULT_CLASSES_PREFIX.to_owned(),
                keys: Arc::new(Mutex::new(BTreeMap::new())),
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowsShellImp {
    backend: WindowsRegistryBackend,
}

impl WindowsShellImp {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        Self {
            backend: WindowsRegistryBackend::InMemory {
                classes_prefix: WINDOWS_DEFAULT_CLASSES_PREFIX.to_owned(),
                keys: Arc::new(Mutex::new(BTreeMap::new())),
            },
        }
    }

    #[cfg(windows)]
    #[inline]
    pub fn with_hkcu_prefix(classes_prefix: impl Into<String>) -> Self {
        Self {
            backend: WindowsRegistryBackend::HkcuRegistry { classes_prefix: classes_prefix.into() },
        }
    }

    #[inline]
    pub fn classes_prefix(&self) -> &str {
        match &self.backend {
            WindowsRegistryBackend::InMemory { classes_prefix, .. } => classes_prefix,
            #[cfg(windows)]
            WindowsRegistryBackend::HkcuRegistry { classes_prefix } => classes_prefix,
        }
    }

    pub fn build_manifest(&self, config: &ShellMenuConfig) -> WindowsRegistryManifest {
        build_windows_registry_manifest(self.classes_prefix(), config)
    }

    fn read_command_key(&self, cmd_key_path: &str) -> Option<String> {
        match &self.backend {
            WindowsRegistryBackend::InMemory { keys, .. } => {
                let guard = keys.lock().ok()?;
                guard.get(cmd_key_path).and_then(|vals| vals.get("")).cloned()
            }
            #[cfg(windows)]
            WindowsRegistryBackend::HkcuRegistry { .. } => {
                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                let subkey = hkcu.open_subkey(cmd_key_path).ok()?;
                subkey.get_value::<String, _>("").ok()
            }
        }
    }

    fn write_manifest(
        &self,
        manifest: &WindowsRegistryManifest,
    ) -> Result<(), ShellIntegrationError> {
        let _ = self.delete_manifest_keys(manifest);
        match &self.backend {
            WindowsRegistryBackend::InMemory { keys, .. } => {
                let mut guard = keys
                    .lock()
                    .map_err(|e| ShellIntegrationError::RegistrationFailed(e.to_string()))?;
                for spec in &manifest.keys {
                    let entry = guard.entry(spec.relative_key_path.clone()).or_default();
                    entry.insert(String::new(), spec.default_value.clone());
                    for (k, v) in &spec.named_values {
                        entry.insert(k.clone(), v.clone());
                    }
                }
                Ok(())
            }
            #[cfg(windows)]
            WindowsRegistryBackend::HkcuRegistry { .. } => {
                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                for spec in &manifest.keys {
                    let (subkey, _) = hkcu
                        .create_subkey(&spec.relative_key_path)
                        .map_err(|e| ShellIntegrationError::RegistrationFailed(e.to_string()))?;
                    if !spec.default_value.is_empty() {
                        subkey.set_value("", &spec.default_value).map_err(|e| {
                            ShellIntegrationError::RegistrationFailed(e.to_string())
                        })?;
                    }
                    for (name, val) in &spec.named_values {
                        subkey.set_value(name, val).map_err(|e| {
                            ShellIntegrationError::RegistrationFailed(e.to_string())
                        })?;
                    }
                }
                Ok(())
            }
        }
    }

    fn delete_manifest_keys(
        &self,
        manifest: &WindowsRegistryManifest,
    ) -> Result<(), ShellIntegrationError> {
        match &self.backend {
            WindowsRegistryBackend::InMemory { keys, .. } => {
                let mut guard = keys
                    .lock()
                    .map_err(|e| ShellIntegrationError::UnregistrationFailed(e.to_string()))?;
                for root_key in &manifest.root_keys_to_delete {
                    let sub_prefix = format!("{root_key}\\");
                    guard.retain(|k, _| k != root_key && !k.starts_with(&sub_prefix));
                }
                Ok(())
            }
            #[cfg(windows)]
            WindowsRegistryBackend::HkcuRegistry { .. } => {
                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                for root_key in &manifest.root_keys_to_delete {
                    if let Err(err) = hkcu.delete_subkey_all(root_key)
                        && err.kind() != std::io::ErrorKind::NotFound
                    {
                        return Err(ShellIntegrationError::UnregistrationFailed(err.to_string()));
                    }
                }
                Ok(())
            }
        }
    }
}

impl ShellIntegrationPort for WindowsShellImp {
    fn status(&self) -> ShellRegistrationStatus {
        self.status_for_config(&ShellMenuConfig::default())
    }

    fn status_for_config(&self, config: &ShellMenuConfig) -> ShellRegistrationStatus {
        let prefix = self.classes_prefix().trim_end_matches('\\');
        let dir_cmd_key =
            format!("{prefix}\\Directory\\shell\\CodeContext\\shell\\01_pack\\command");
        let apply_cmd_key =
            format!("{prefix}\\Directory\\shell\\CodeContext\\shell\\02_apply\\command");
        let bg_cmd_key =
            format!("{prefix}\\Directory\\Background\\shell\\CodeContext\\shell\\01_pack\\command");

        let Some(dir_cmd) = self.read_command_key(&dir_cmd_key) else {
            return ShellRegistrationStatus::NotRegistered;
        };
        let Some(apply_cmd) = self.read_command_key(&apply_cmd_key) else {
            return ShellRegistrationStatus::StaleExecutablePath { registered_command: dir_cmd };
        };
        let Some(bg_cmd) = self.read_command_key(&bg_cmd_key) else {
            return ShellRegistrationStatus::StaleExecutablePath { registered_command: dir_cmd };
        };

        let expected_pack_cmd = expected_windows_pack_dir_command(config);
        let expected_apply_cmd = expected_windows_apply_dir_command(config);
        if dir_cmd == expected_pack_cmd
            && bg_cmd == expected_pack_cmd
            && apply_cmd == expected_apply_cmd
        {
            ShellRegistrationStatus::RegisteredHealthy
        } else {
            ShellRegistrationStatus::StaleExecutablePath { registered_command: dir_cmd }
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
        let manifest = self.build_manifest(config);
        self.write_manifest(&manifest)
    }

    fn unregister_context_menu(&self) -> Result<(), ShellIntegrationError> {
        let manifest = self.build_manifest(&ShellMenuConfig::default());
        self.delete_manifest_keys(&manifest)
    }
}
