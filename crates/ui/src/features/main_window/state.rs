use core::packing::aggregates::project::Project;
use core::system::ports::hotkey::GlobalHotkeyPort as _;
use core::system::ports::ipc::{IpcCommand, SingleInstanceIpcPort as _, SystemActivationEvent};
use di::AppContainer;

use crate::features::change_applier::state::ChangeApplierState;
use crate::features::context_packer::state::ContextPackerState;
use crate::locale::AppLocale;
use crate::theme::ThemeVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceTab {
    #[default]
    ContextPacker,
    ChangeApplier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusBarSummary {
    pub files_count: usize,
    pub total_bytes: u64,
    pub backups_enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MainWindowState {
    active_tab: WorkspaceTab,
    settings_open: bool,
    project_meta_open: bool,
    locale: AppLocale,
    theme_variant: ThemeVariant,
}

impl MainWindowState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub const fn set_active_tab(&mut self, tab: WorkspaceTab) {
        self.active_tab = tab;
        self.settings_open = false;
        self.project_meta_open = false;
    }

    #[inline]
    pub const fn switch_to_tab(
        &mut self,
        tab: WorkspaceTab,
        auto_read_clipboard_on_apply: bool,
    ) -> bool {
        self.active_tab = tab;
        self.settings_open = false;
        self.project_meta_open = false;
        matches!(tab, WorkspaceTab::ChangeApplier) && auto_read_clipboard_on_apply
    }

    #[inline]
    pub const fn active_tab(&self) -> WorkspaceTab {
        self.active_tab
    }

    #[inline]
    pub const fn toggle_settings(&mut self) {
        self.settings_open = !self.settings_open;
        if self.settings_open {
            self.project_meta_open = false;
        }
    }

    #[inline]
    pub const fn set_settings_open(&mut self, open: bool) {
        self.settings_open = open;
    }

    #[inline]
    pub const fn is_settings_open(&self) -> bool {
        self.settings_open
    }

    #[inline]
    pub const fn toggle_project_meta(&mut self) {
        self.project_meta_open = !self.project_meta_open;
        if self.project_meta_open {
            self.settings_open = false;
        }
    }

    #[inline]
    pub const fn set_project_meta_open(&mut self, open: bool) {
        self.project_meta_open = open;
        if open {
            self.settings_open = false;
        }
    }

    #[inline]
    pub const fn is_project_meta_open(&self) -> bool {
        self.project_meta_open
    }

    #[inline]
    pub const fn toggle_locale(&mut self) {
        self.locale = self.locale.toggle();
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
    pub const fn toggle_theme(&mut self) {
        self.theme_variant = self.theme_variant.toggle();
    }

    #[inline]
    pub const fn set_theme_variant(&mut self, variant: ThemeVariant) {
        self.theme_variant = variant;
    }

    #[inline]
    pub const fn theme_variant(&self) -> ThemeVariant {
        self.theme_variant
    }

    pub fn handle_system_activation_event(
        &mut self,
        event: SystemActivationEvent,
        packer: &mut ContextPackerState,
        applier: &mut ChangeApplierState,
        auto_read_clipboard_on_apply: bool,
    ) -> bool {
        match event {
            SystemActivationEvent::OpenProjectAndFocus { project_root } => {
                let _ = packer.open_project(project_root.clone());
                applier.set_project_root(Some(project_root));
                true
            }
            SystemActivationEvent::ToggleOrFocusWindow => {
                if matches!(self.active_tab, WorkspaceTab::ChangeApplier)
                    && auto_read_clipboard_on_apply
                {
                    let _ = applier.load_response_from_clipboard_if_changed();
                }
                true
            }
        }
    }

    pub fn poll_system_events(
        &mut self,
        container: &AppContainer,
        packer: &mut ContextPackerState,
        applier: &mut ChangeApplierState,
        auto_read_clipboard_on_apply: bool,
    ) -> usize {
        let mut processed = 0_usize;

        while let Some(cmd) = container.ipc().try_recv_command() {
            match cmd {
                IpcCommand::OpenProjectAndFocus { project_root } => {
                    self.handle_system_activation_event(
                        SystemActivationEvent::OpenProjectAndFocus { project_root },
                        packer,
                        applier,
                        auto_read_clipboard_on_apply,
                    );
                    processed += 1;
                }
                IpcCommand::FocusWindow => {
                    self.handle_system_activation_event(
                        SystemActivationEvent::ToggleOrFocusWindow,
                        packer,
                        applier,
                        auto_read_clipboard_on_apply,
                    );
                    processed += 1;
                }
                IpcCommand::Ping => {}
            }
        }

        if container.hotkey().try_recv_hotkey_trigger() {
            self.handle_system_activation_event(
                SystemActivationEvent::ToggleOrFocusWindow,
                packer,
                applier,
                auto_read_clipboard_on_apply,
            );
            processed += 1;
        }

        processed
    }

    pub fn compute_status_bar_summary(
        &self,
        packer: &ContextPackerState,
        applier: &ChangeApplierState,
    ) -> StatusBarSummary {
        match self.active_tab {
            WorkspaceTab::ContextPacker => {
                let files_count = packer.project().map_or(0, Project::selected_files_count);
                let total_bytes = packer.pack_result().map_or_else(
                    || packer.project().map_or(0, Project::estimated_pack_capacity_bytes),
                    |res| res.total_bytes as u64,
                );
                StatusBarSummary {
                    files_count,
                    total_bytes,
                    backups_enabled: applier.backups_enabled(),
                }
            }
            WorkspaceTab::ChangeApplier => StatusBarSummary {
                files_count: applier.parsed_files().len(),
                total_bytes: applier.response_buffer().total_bytes() as u64,
                backups_enabled: applier.backups_enabled(),
            },
        }
    }
}
