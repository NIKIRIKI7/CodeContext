use core::packing::aggregates::project::Project;
use core::preset::models::resolution::TerminalTaskCategory;
use core::shared_kernel::execution::telemetry::LogFilterMode;
use core::system::ports::hotkey::GlobalHotkeyPort as _;
use core::system::ports::ipc::{IpcCommand, SingleInstanceIpcPort as _, SystemActivationEvent};
use core::system::ports::updater::{SemanticVersion, UpdateChannel};
use di::AppContainer;

use crate::features::change_applier::state::ChangeApplierState;
use crate::features::context_packer::state::ContextPackerState;
use crate::features::main_window::components::diagnostics_dock::{
    DEFAULT_DOCK_HEIGHT_PX, DockToolTab, MAX_DOCK_HEIGHT_PX, MIN_DOCK_HEIGHT_PX,
    TALL_DOCK_HEIGHT_PX, TerminalCommandEntry,
};
use crate::locale::AppLocale;
use crate::theme::ThemeVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceTab {
    #[default]
    ContextPacker,
    ChangeApplier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectsMenuState {
    #[default]
    Closed,
    OpenMain,
    OpenWithRecentSubmenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusBarSummary {
    pub files_count: usize,
    pub total_bytes: u64,
    pub backups_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct BottomDockState {
    pub is_open: bool,
    pub dock_height_px: f32,
    pub resizing_dock: Option<(f32, f32)>,
    pub active_tab: DockToolTab,
    pub log_filter_mode: LogFilterMode,
    pub task_category: Option<TerminalTaskCategory>,
    pub terminal_input: String,
    pub terminal_focused: bool,
    pub terminal_running: bool,
    pub terminal_history: Vec<TerminalCommandEntry>,
}

impl Default for BottomDockState {
    fn default() -> Self {
        Self {
            is_open: false,
            dock_height_px: DEFAULT_DOCK_HEIGHT_PX,
            resizing_dock: None,
            active_tab: DockToolTab::default(),
            log_filter_mode: LogFilterMode::default(),
            task_category: None,
            terminal_input: String::new(),
            terminal_focused: false,
            terminal_running: false,
            terminal_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct MainWindowState {
    active_tab: WorkspaceTab,
    settings_open: bool,
    project_meta_open: bool,
    command_palette: CommandPaletteState,
    bottom_dock: BottomDockState,
    projects_menu: ProjectsMenuState,
    update_modal_open: bool,
    available_update: Option<(SemanticVersion, UpdateChannel)>,
    dismissed_update_version: Option<SemanticVersion>,
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
        self.projects_menu = ProjectsMenuState::Closed;
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
        self.projects_menu = ProjectsMenuState::Closed;
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
            self.projects_menu = ProjectsMenuState::Closed;
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
    pub const fn command_palette(&self) -> &CommandPaletteState {
        &self.command_palette
    }

    #[inline]
    pub const fn command_palette_mut(&mut self) -> &mut CommandPaletteState {
        &mut self.command_palette
    }

    pub fn open_command_palette(&mut self, commands_mode: bool) {
        self.command_palette.is_open = true;
        self.command_palette.selected_index = 0;
        self.command_palette.query = if commands_mode { ">".to_owned() } else { String::new() };
        self.settings_open = false;
        self.project_meta_open = false;
        self.projects_menu = ProjectsMenuState::Closed;
    }

    pub fn close_command_palette(&mut self) {
        self.command_palette.is_open = false;
        self.command_palette.query.clear();
        self.command_palette.selected_index = 0;
    }

    #[inline]
    pub const fn toggle_logs_dock(&mut self) {
        self.bottom_dock.is_open = !self.bottom_dock.is_open;
        if !self.bottom_dock.is_open {
            self.bottom_dock.terminal_focused = false;
        }
    }

    #[inline]
    pub fn toggle_dock_tall_height(&mut self) {
        if (self.bottom_dock.dock_height_px - DEFAULT_DOCK_HEIGHT_PX).abs() < 20.0 {
            self.bottom_dock.dock_height_px = TALL_DOCK_HEIGHT_PX;
        } else {
            self.bottom_dock.dock_height_px = DEFAULT_DOCK_HEIGHT_PX;
        }
    }

    #[inline]
    pub const fn start_dock_resize(&mut self, mouse_y: f32) {
        self.bottom_dock.resizing_dock = Some((mouse_y, self.bottom_dock.dock_height_px));
    }

    #[inline]
    pub const fn stop_dock_resize(&mut self) {
        self.bottom_dock.resizing_dock = None;
    }

    #[inline]
    pub const fn update_dock_resize(&mut self, current_mouse_y: f32, max_allowed_height: f32) {
        if let Some((start_y, start_h)) = self.bottom_dock.resizing_dock {
            let delta = start_y - current_mouse_y;
            let raw = start_h + delta;
            let max_h = if max_allowed_height > MIN_DOCK_HEIGHT_PX {
                if max_allowed_height < MAX_DOCK_HEIGHT_PX {
                    max_allowed_height
                } else {
                    MAX_DOCK_HEIGHT_PX
                }
            } else {
                MIN_DOCK_HEIGHT_PX
            };
            self.bottom_dock.dock_height_px = if raw < MIN_DOCK_HEIGHT_PX {
                MIN_DOCK_HEIGHT_PX
            } else if raw > max_h {
                max_h
            } else {
                raw
            };
        }
    }

    #[inline]
    pub fn open_or_toggle_dock_tab(&mut self, tab: DockToolTab) {
        if self.bottom_dock.is_open && self.bottom_dock.active_tab == tab {
            self.bottom_dock.is_open = false;
            self.bottom_dock.terminal_focused = false;
        } else {
            self.bottom_dock.is_open = true;
            self.bottom_dock.active_tab = tab;
        }
    }

    #[inline]
    pub const fn set_dock_tab(&mut self, tab: DockToolTab) {
        self.bottom_dock.active_tab = tab;
        self.bottom_dock.is_open = true;
    }

    #[inline]
    pub const fn dock_active_tab(&self) -> DockToolTab {
        self.bottom_dock.active_tab
    }

    #[inline]
    pub const fn bottom_dock(&self) -> &BottomDockState {
        &self.bottom_dock
    }

    #[inline]
    pub const fn bottom_dock_mut(&mut self) -> &mut BottomDockState {
        &mut self.bottom_dock
    }

    #[inline]
    pub const fn set_logs_dock_open(&mut self, open: bool) {
        self.bottom_dock.is_open = open;
        if !open {
            self.bottom_dock.terminal_focused = false;
        }
    }

    #[inline]
    pub const fn is_logs_dock_open(&self) -> bool {
        self.bottom_dock.is_open
    }

    #[inline]
    pub const fn set_log_filter_mode(&mut self, mode: LogFilterMode) {
        self.bottom_dock.log_filter_mode = mode;
    }

    #[inline]
    pub const fn log_filter_mode(&self) -> LogFilterMode {
        self.bottom_dock.log_filter_mode
    }

    #[inline]
    pub const fn toggle_project_meta(&mut self) {
        self.project_meta_open = !self.project_meta_open;
        if self.project_meta_open {
            self.settings_open = false;
            self.projects_menu = ProjectsMenuState::Closed;
        }
    }

    #[inline]
    pub const fn set_project_meta_open(&mut self, open: bool) {
        self.project_meta_open = open;
        if open {
            self.settings_open = false;
            self.projects_menu = ProjectsMenuState::Closed;
        }
    }

    #[inline]
    pub const fn is_project_meta_open(&self) -> bool {
        self.project_meta_open
    }

    #[inline]
    pub const fn toggle_recent_projects(&mut self) {
        self.projects_menu = match self.projects_menu {
            ProjectsMenuState::Closed => {
                self.settings_open = false;
                self.project_meta_open = false;
                ProjectsMenuState::OpenMain
            }
            ProjectsMenuState::OpenMain | ProjectsMenuState::OpenWithRecentSubmenu => {
                ProjectsMenuState::Closed
            }
        };
    }

    #[inline]
    pub const fn set_recent_projects_open(&mut self, open: bool) {
        self.projects_menu =
            if open { ProjectsMenuState::OpenMain } else { ProjectsMenuState::Closed };
    }

    #[inline]
    pub fn set_recent_submenu_open(&mut self, submenu_open: bool) -> bool {
        if matches!(self.projects_menu, ProjectsMenuState::Closed) {
            return false;
        }
        let next = if submenu_open {
            ProjectsMenuState::OpenWithRecentSubmenu
        } else {
            ProjectsMenuState::OpenMain
        };
        if self.projects_menu == next {
            return false;
        }
        self.projects_menu = next;
        true
    }

    #[inline]
    pub const fn is_recent_projects_open(&self) -> bool {
        !matches!(self.projects_menu, ProjectsMenuState::Closed)
    }

    #[inline]
    pub const fn is_recent_submenu_open(&self) -> bool {
        matches!(self.projects_menu, ProjectsMenuState::OpenWithRecentSubmenu)
    }

    pub fn notify_update_available(&mut self, version: SemanticVersion, channel: UpdateChannel) {
        self.available_update = Some((version, channel));
        if self.dismissed_update_version != Some(version) {
            self.update_modal_open = true;
        }
    }

    #[inline]
    pub const fn dismiss_update_modal(&mut self) {
        if let Some((ver, _)) = self.available_update {
            self.dismissed_update_version = Some(ver);
        }
        self.update_modal_open = false;
    }

    #[inline]
    pub const fn close_update_modal(&mut self) {
        self.update_modal_open = false;
    }

    #[inline]
    pub const fn is_update_modal_open(&self) -> bool {
        self.update_modal_open
    }

    #[inline]
    pub const fn available_update(&self) -> Option<(SemanticVersion, UpdateChannel)> {
        self.available_update
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
