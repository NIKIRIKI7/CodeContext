use core::preset::models::resolution::{
    DEFAULT_GLOBAL_HOTKEY, GlobalConfig, KeymapAction, KeymapConfig,
};
use core::preset::ports::config_repo::ConfigPortError;
use core::system::ports::hotkey::{
    GlobalHotkeyPort as _, HotkeyPortError, HotkeyRegistrationStatus, HotkeyShortcut,
};
use core::system::ports::shell::{
    ShellIntegrationError, ShellIntegrationPort as _, ShellMenuConfig, ShellRegistrationStatus,
};
use core::system::ports::updater::{
    SelfInstallStatus, SemanticVersion, UpdateChannel, UpdateStage, UpdaterPort as _,
    UpdaterPortError,
};
use di::AppContainer;

use crate::locale::AppLocale;
use crate::theme::ThemeVariant;

const HOTKEY_CYCLE_PRESETS: &[&str] = &["Ctrl+Shift+C", "Ctrl+Alt+C", "Ctrl+Shift+P"];

#[derive(Debug, Clone)]
pub struct SettingsModalState {
    container: AppContainer,
    global_config: GlobalConfig,
    shell_status: ShellRegistrationStatus,
    hotkey_status: HotkeyRegistrationStatus,
    update_channel: UpdateChannel,
    update_stage: UpdateStage,
    self_install_status: SelfInstallStatus,
    locale: AppLocale,
    theme_variant: ThemeVariant,
    hotkeys_modal_open: bool,
    recording_action: Option<KeymapAction>,
    close_requested: bool,
}

impl SettingsModalState {
    pub fn new(container: AppContainer) -> Self {
        let global_config = container.preset_service().load_global_config();
        let shell_status = container.shell_integration().status();
        let current_hk_status = container.hotkey().status();
        let hotkey_status = if matches!(current_hk_status, HotkeyRegistrationStatus::NotRegistered)
        {
            let _ = container.hotkey().register_str(&global_config.global_hotkey);
            container.hotkey().status()
        } else {
            current_hk_status
        };
        let update_channel = UpdateChannel::parse(&global_config.update_channel);
        let update_stage = container.updater().stage();
        let self_install_status = container.updater().self_install_status();

        Self {
            container,
            global_config,
            shell_status,
            hotkey_status,
            update_channel,
            update_stage,
            self_install_status,
            locale: AppLocale::default(),
            theme_variant: ThemeVariant::default(),
            hotkeys_modal_open: false,
            recording_action: None,
            close_requested: false,
        }
    }

    #[inline]
    pub const fn is_hotkeys_modal_open(&self) -> bool {
        self.hotkeys_modal_open
    }

    #[inline]
    pub const fn open_hotkeys_modal(&mut self) {
        self.hotkeys_modal_open = true;
        self.recording_action = None;
    }

    #[inline]
    pub const fn close_hotkeys_modal(&mut self) {
        self.hotkeys_modal_open = false;
        self.recording_action = None;
    }

    #[inline]
    pub const fn recording_action(&self) -> Option<KeymapAction> {
        self.recording_action
    }

    #[inline]
    pub fn toggle_recording_action(&mut self, action: KeymapAction) {
        if self.recording_action == Some(action) {
            self.recording_action = None;
        } else {
            self.recording_action = Some(action);
        }
    }

    #[inline]
    pub const fn is_recording_hotkey(&self) -> bool {
        self.recording_action.is_some()
    }

    #[inline]
    pub fn toggle_recording_hotkey(&mut self) {
        self.toggle_recording_action(KeymapAction::GlobalHotkey);
    }

    #[inline]
    pub const fn cancel_recording_hotkey(&mut self) {
        self.recording_action = None;
    }

    #[inline]
    pub const fn set_locale(&mut self, locale: AppLocale) {
        self.locale = locale;
    }

    #[inline]
    pub const fn locale(&self) -> AppLocale {
        self.locale
    }

    #[inline]
    pub const fn set_theme_variant(&mut self, theme_variant: ThemeVariant) {
        self.theme_variant = theme_variant;
    }

    #[inline]
    pub const fn theme_variant(&self) -> ThemeVariant {
        self.theme_variant
    }

    #[inline]
    pub const fn request_close(&mut self) {
        self.hotkeys_modal_open = false;
        self.recording_action = None;
        self.close_requested = true;
    }

    #[inline]
    pub const fn take_close_request(&mut self) -> bool {
        let req = self.close_requested;
        self.close_requested = false;
        req
    }

    pub fn toggle_auto_read_clipboard(&mut self) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().toggle_auto_read_clipboard()?;
        self.global_config = updated;
        Ok(())
    }

    pub fn toggle_create_backups_by_default(&mut self) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().toggle_create_backups_by_default()?;
        self.global_config = updated;
        Ok(())
    }

    pub fn toggle_shell_context_menu(
        &mut self,
    ) -> Result<ShellRegistrationStatus, ShellIntegrationError> {
        let shell = self.container.shell_integration();
        if self.shell_status.is_registered() {
            shell.unregister_context_menu()?;
        } else {
            shell.register_context_menu()?;
        }
        self.shell_status = shell.status();
        Ok(self.shell_status.clone())
    }

    pub fn rebind_global_hotkey(
        &mut self,
        new_shortcut: &str,
    ) -> Result<HotkeyRegistrationStatus, HotkeyPortError> {
        let parsed = HotkeyShortcut::parse(new_shortcut)?;
        let canonical = parsed.to_canonical_string();
        let result = self.container.hotkey().rebind_str(&canonical);
        self.hotkey_status = self.container.hotkey().status();
        self.recording_action = None;

        if result.is_ok() {
            self.global_config.global_hotkey = canonical;
            let _ = self.container.preset_service().save_global_config(&self.global_config);
        }
        result
    }

    pub fn rebind_action_shortcut(
        &mut self,
        action: KeymapAction,
        new_shortcut: &str,
    ) -> Result<(), HotkeyPortError> {
        if action == KeymapAction::GlobalHotkey {
            let _ = self.rebind_global_hotkey(new_shortcut)?;
            return Ok(());
        }
        let parsed = HotkeyShortcut::parse(new_shortcut)?;
        let canonical = parsed.to_canonical_string();
        self.global_config.keymap.set_shortcut(action, canonical);
        self.recording_action = None;
        let _ = self.container.preset_service().save_global_config(&self.global_config);
        Ok(())
    }

    pub fn reset_all_shortcuts_to_default(&mut self) {
        self.recording_action = None;
        self.global_config.keymap = KeymapConfig::default();
        let _ = self.rebind_global_hotkey(DEFAULT_GLOBAL_HOTKEY);
        let _ = self.container.preset_service().save_global_config(&self.global_config);
    }

    pub fn cycle_hotkey_preset(&mut self) -> Result<HotkeyRegistrationStatus, HotkeyPortError> {
        let curr = self.global_config.global_hotkey.as_str();
        let pos =
            HOTKEY_CYCLE_PRESETS.iter().position(|p| p.eq_ignore_ascii_case(curr)).unwrap_or(0);
        let next_idx = (pos + 1) % HOTKEY_CYCLE_PRESETS.len();
        let next_preset = HOTKEY_CYCLE_PRESETS.get(next_idx).copied().unwrap_or("Ctrl+Shift+C");
        self.rebind_global_hotkey(next_preset)
    }

    #[inline]
    pub const fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }

    #[inline]
    pub const fn shell_status(&self) -> &ShellRegistrationStatus {
        &self.shell_status
    }

    #[inline]
    pub const fn hotkey_status(&self) -> &HotkeyRegistrationStatus {
        &self.hotkey_status
    }

    #[inline]
    pub const fn update_channel(&self) -> UpdateChannel {
        self.update_channel
    }

    #[inline]
    pub const fn update_stage(&self) -> &UpdateStage {
        &self.update_stage
    }

    #[inline]
    pub const fn self_install_status(&self) -> &SelfInstallStatus {
        &self.self_install_status
    }

    #[inline]
    pub fn current_version(&self) -> SemanticVersion {
        self.container.updater().current_version()
    }

    #[inline]
    pub const fn container(&self) -> &AppContainer {
        &self.container
    }

    pub fn sync_from_container(&mut self) {
        self.update_stage = self.container.updater().stage();
        self.self_install_status = self.container.updater().self_install_status();
        self.shell_status = self.container.shell_integration().status();
    }

    pub fn select_update_channel(&mut self, channel: UpdateChannel) -> Result<(), ConfigPortError> {
        if self.update_channel == channel {
            return Ok(());
        }
        self.update_channel = channel;
        let updated = self.container.preset_service().set_update_channel(channel.as_str())?;
        self.global_config = updated;
        self.update_stage = self.container.updater().switch_channel_cached(channel);
        Ok(())
    }

    pub fn check_for_updates(&mut self) -> Result<UpdateStage, UpdaterPortError> {
        let stage = self
            .container
            .updater()
            .check_for_updates(self.update_channel, &self.global_config.update_repo_url)?;
        self.update_stage = stage.clone();
        Ok(stage)
    }

    pub fn trigger_update_or_self_install(&mut self) -> Result<(), UpdaterPortError> {
        if let SelfInstallStatus::PortableMode { .. } = self.self_install_status {
            let status = self.container.updater().perform_self_install()?;
            if let SelfInstallStatus::InstalledCanonical { ref install_path } = status {
                let cfg = ShellMenuConfig::with_executable(install_path.clone());
                let _ = self.container.shell_integration().register_with_config(&cfg);
                self.shell_status = self.container.shell_integration().status_for_config(&cfg);
            }
            self.self_install_status = status;
            return Ok(());
        }

        match self.update_stage {
            UpdateStage::UpdateAvailable { .. } => {
                self.update_stage = self.container.updater().apply_staged_update()?;
            }
            UpdateStage::Idle
            | UpdateStage::UpToDate { .. }
            | UpdateStage::ReadyToRestart { .. }
            | UpdateStage::Failed { .. } => {
                self.update_stage = self
                    .container
                    .updater()
                    .check_for_updates(self.update_channel, &self.global_config.update_repo_url)?;
            }
        }
        Ok(())
    }
}
