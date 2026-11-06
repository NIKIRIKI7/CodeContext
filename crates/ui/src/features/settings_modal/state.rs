use core::preset::models::resolution::{
    DEFAULT_GLOBAL_HOTKEY, EditionProfile, FeatureFlag, GlobalConfig, KeymapAction, KeymapConfig,
    TerminalCommandPreset, UiChromeSetting,
};
use core::preset::ports::config_repo::ConfigPortError;
use core::system::plugins::protocol::{PluginDescriptor, PluginHostPort as _};
use core::system::ports::hotkey::{
    GlobalHotkeyPort as _, HotkeyPortError, HotkeyRegistrationStatus, HotkeyShortcut,
};
use core::system::ports::shell::{
    ShellIntegrationError, ShellIntegrationPort as _, ShellMenuConfig, ShellRegistrationStatus,
};
use core::system::ports::updater::{
    SelfInstallStatus, SemanticVersion, SystemProbeReport, UpdateChannel, UpdateStage,
    UpdaterPort as _, UpdaterPortError,
};
use di::AppContainer;

use crate::locale::AppLocale;
use crate::shared::containers::virtual_text::VirtualTextBuffer;
use crate::theme::ThemeVariant;

const HOTKEY_CYCLE_PRESETS: &[&str] = &["Ctrl+K", "Ctrl+Shift+C", "Ctrl+Alt+C"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SetupWizardStep {
    #[default]
    Step1SystemProbe,
    Step2EditionSelect,
    Step3TargetAndOs,
    Step4Execute,
}

#[derive(Debug, Clone)]
pub struct SetupWizardState {
    pub is_open: bool,
    pub step: SetupWizardStep,
    pub register_shell: bool,
    pub add_to_path: bool,
    pub status_message: Option<String>,
}

impl Default for SetupWizardState {
    fn default() -> Self {
        Self {
            is_open: false,
            step: SetupWizardStep::Step1SystemProbe,
            register_shell: true,
            add_to_path: true,
            status_message: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SettingsCategory {
    #[default]
    Appearance,
    PanelsAndBar,
    Keymap,
    Safety,
    SystemAndUpdates,
    Maintenance,
}

impl SettingsCategory {
    pub const ALL: [Self; 5] =
        [Self::Appearance, Self::PanelsAndBar, Self::Keymap, Self::Safety, Self::SystemAndUpdates];
}

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
    active_category: SettingsCategory,
    json_editor_mode: bool,
    json_buffer: VirtualTextBuffer,
    editing_json_line: Option<(usize, String)>,
    json_error: Option<String>,
    wizard: SetupWizardState,
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

        let json_str = container.export_global_settings_json();
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
            active_category: SettingsCategory::default(),
            json_editor_mode: false,
            json_buffer: VirtualTextBuffer::new(json_str),
            editing_json_line: None,
            json_error: None,
            wizard: SetupWizardState::default(),
            hotkeys_modal_open: false,
            recording_action: None,
            close_requested: false,
        }
    }

    #[inline]
    pub const fn wizard(&self) -> &SetupWizardState {
        &self.wizard
    }

    pub fn open_setup_wizard(&mut self) {
        self.wizard.is_open = true;
        self.wizard.step = SetupWizardStep::Step1SystemProbe;
        self.wizard.status_message = None;
        self.json_editor_mode = false;
    }

    pub fn close_setup_wizard(&mut self) {
        self.wizard.is_open = false;
    }

    pub fn set_wizard_step(&mut self, step: SetupWizardStep) {
        self.wizard.step = step;
    }

    pub fn toggle_wizard_register_shell(&mut self) {
        self.wizard.register_shell = !self.wizard.register_shell;
    }

    pub fn toggle_wizard_add_path(&mut self) {
        self.wizard.add_to_path = !self.wizard.add_to_path;
    }

    pub fn execute_wizard_install(&mut self) -> Result<std::path::PathBuf, String> {
        let edition = self.global_config.edition;
        let res =
            self.container.perform_modular_setup(edition, None, self.wizard.register_shell)?;
        self.sync_from_container();
        self.wizard.step = SetupWizardStep::Step4Execute;
        self.wizard.status_message = Some(format!("Успешно установлено: {}", res.display()));
        Ok(res)
    }

    pub fn execute_wizard_uninstall(&mut self) -> Result<usize, String> {
        let _ = self.container.shell_integration().unregister_context_menu();
        let removed =
            self.container.updater().execute_uninstall(false).map_err(|e| e.to_string())?;
        self.sync_from_container();
        self.wizard.step = SetupWizardStep::Step4Execute;
        self.wizard.status_message =
            Some(format!("Деинсталляция завершена (удалено объектов: {removed})"));
        Ok(removed)
    }

    #[inline]
    pub const fn is_json_editor_mode(&self) -> bool {
        self.json_editor_mode
    }

    pub fn toggle_json_editor_mode(&mut self) {
        self.json_editor_mode = !self.json_editor_mode;
        self.editing_json_line = None;
        self.json_error = None;
        if self.json_editor_mode {
            self.refresh_json_buffer();
        }
    }

    pub fn refresh_json_buffer(&mut self) {
        let raw = self.container.export_global_settings_json();
        self.json_buffer = VirtualTextBuffer::new(raw);
    }

    #[inline]
    pub const fn json_buffer(&self) -> &VirtualTextBuffer {
        &self.json_buffer
    }

    #[inline]
    pub fn json_error(&self) -> Option<&str> {
        self.json_error.as_deref()
    }

    #[inline]
    pub fn editing_json_line(&self) -> Option<(usize, &str)> {
        self.editing_json_line.as_ref().map(|(idx, s)| (*idx, s.as_str()))
    }

    pub fn start_editing_json_line(&mut self, line_idx: usize) {
        let raw = self.json_buffer.raw_line(line_idx).to_owned();
        self.editing_json_line = Some((line_idx, raw));
    }

    pub fn cancel_editing_json_line(&mut self) {
        self.editing_json_line = None;
    }

    pub fn append_json_line_char(&mut self, ch: char) {
        if let Some((_, ref mut draft)) = self.editing_json_line {
            draft.push(ch);
        }
    }

    pub fn pop_json_line_char(&mut self) -> bool {
        if let Some((_, ref mut draft)) = self.editing_json_line {
            return draft.pop().is_some();
        }
        false
    }

    pub fn commit_json_line_edit(&mut self) {
        let Some((line_idx, new_line)) = self.editing_json_line.take() else {
            return;
        };
        let mut lines: Vec<&str> = self.json_buffer.as_str().lines().collect();
        if let Some(slot) = lines.get_mut(line_idx) {
            *slot = &new_line;
            let updated_json = lines.join("\n");
            self.json_buffer = VirtualTextBuffer::new(updated_json.as_str());
            match self.container.apply_global_settings_json(&updated_json) {
                Ok((new_cfg, _)) => {
                    self.global_config = new_cfg;
                    self.json_error = None;
                }
                Err(err) => {
                    self.json_error = Some(err);
                }
            }
        }
    }

    pub fn move_json_line_edit(&mut self, delta: isize) {
        let Some((curr_idx, _)) = self.editing_json_line else {
            return;
        };
        self.commit_json_line_edit();
        let total = self.json_buffer.line_count();
        if total == 0 {
            return;
        }
        let next_idx = if delta < 0 {
            curr_idx.saturating_sub(delta.unsigned_abs())
        } else {
            (curr_idx + delta.unsigned_abs()).min(total - 1)
        };
        self.start_editing_json_line(next_idx);
    }

    #[inline]
    pub const fn active_category(&self) -> SettingsCategory {
        self.active_category
    }

    #[inline]
    pub const fn set_active_category(&mut self, category: SettingsCategory) {
        self.active_category = category;
        self.recording_action = None;
        self.hotkeys_modal_open = matches!(category, SettingsCategory::Keymap);
    }

    #[inline]
    pub const fn is_hotkeys_modal_open(&self) -> bool {
        self.hotkeys_modal_open
    }

    #[inline]
    pub const fn open_hotkeys_modal(&mut self) {
        self.active_category = SettingsCategory::Keymap;
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

    pub fn toggle_show_bottom_bar(&mut self) -> Result<(), ConfigPortError> {
        self.toggle_ui_chrome(UiChromeSetting::BottomBar)
    }

    pub fn toggle_ui_chrome(&mut self, setting: UiChromeSetting) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().toggle_ui_chrome(setting)?;
        self.global_config = updated;
        if self.json_editor_mode {
            self.refresh_json_buffer();
        }
        Ok(())
    }

    pub fn toggle_feature_flag(&mut self, flag: FeatureFlag) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().toggle_feature_flag(flag)?;
        self.global_config = updated;
        if self.json_editor_mode {
            self.refresh_json_buffer();
        }
        Ok(())
    }

    pub fn select_edition_profile(
        &mut self,
        profile: EditionProfile,
    ) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().apply_edition_profile(profile)?;
        self.global_config = updated;
        if self.json_editor_mode {
            self.refresh_json_buffer();
        }
        Ok(())
    }

    #[inline]
    pub fn list_plugins(&self) -> Vec<PluginDescriptor> {
        self.container.list_plugins()
    }

    pub fn toggle_plugin(&mut self, plugin_id: &str) {
        let curr = self
            .container
            .list_plugins()
            .into_iter()
            .find(|p| p.manifest.id == plugin_id)
            .is_some_and(|p| p.enabled);
        let _ = self.container.plugin_host().set_plugin_enabled(plugin_id, !curr);
    }

    #[inline]
    pub fn system_probe_report(&self) -> SystemProbeReport {
        self.container.updater().probe_system()
    }

    pub fn add_terminal_preset(
        &mut self,
        preset: TerminalCommandPreset,
    ) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().add_terminal_preset(preset)?;
        self.global_config = updated;
        Ok(())
    }

    pub fn remove_terminal_preset(&mut self, index: usize) -> Result<(), ConfigPortError> {
        let updated = self.container.preset_service().remove_terminal_preset(index)?;
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
        let next_preset = HOTKEY_CYCLE_PRESETS.get(next_idx).copied().unwrap_or("Ctrl+K");
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
