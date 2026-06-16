import os
from PySide6.QtWidgets import (QApplication, QMainWindow, QWidget, QHBoxLayout, QVBoxLayout,
                               QSplitter, QFileDialog, QStackedWidget,
                               QLabel, QGraphicsOpacityEffect, QTabWidget)
from PySide6.QtCore import Signal, QObject, Qt, QPropertyAnimation, QTimer
from PySide6.QtGui import QKeySequence, QShortcut

from ..store.state import AppState
from ..controllers.main_controller import MainController
from .components.sidebar import Sidebar
from .components.folder_list import FolderList
from .components.action_panel import ActionPanel
from .components.log_panel import LogPanel
from .components.status_bar import StatusBar
from .components.file_tree import FileTree
from .components.empty_state import EmptyState
from .components.analytics_panel import AnalyticsPanel
from .dialogs import AdvancedPreviewDialog, InteractiveTourDialog, EditFolderDialog, UpdateDialog, CommandPaletteDialog
from .theme_manager import ThemeManager, theme_bus
from ..utils.config import PricingManager, get_app_version
from src.i18n import tr

class ToastNotification(QLabel):
    def __init__(self, parent):
        super().__init__(parent)
        self.setProperty("cssClass", "toast")
        self.setAlignment(Qt.AlignCenter)
        self.hide()
        self.opacity_effect = QGraphicsOpacityEffect(self)
        self.setGraphicsEffect(self.opacity_effect)
        self.animation = QPropertyAnimation(self.opacity_effect, b"opacity")
        self.timer = QTimer()
        self.timer.setSingleShot(True)
        self.timer.timeout.connect(self.fade_out)

    def show_message(self, message):
        self.setText(message)
        self.adjustSize()
        self.resize(self.width() + 40, self.height() + 10)
        parent_rect = self.parent().rect()
        x = (parent_rect.width() - self.width()) // 2
        y = parent_rect.height() - self.height() - 80
        self.move(x, y)
        self.opacity_effect.setOpacity(1.0)
        self.show()
        self.raise_()
        self.timer.start(2000)

    def fade_out(self):
        self.animation.setDuration(500)
        self.animation.setStartValue(1.0)
        self.animation.setEndValue(0.0)
        self.animation.finished.connect(self.hide)
        self.animation.start()

class MainWindow(QMainWindow):
    def __init__(self, state: AppState, controller: MainController):
        super().__init__()
        self.state = state
        self.controller = controller

        self.setWindowTitle(tr("main_window.title", default="CodeContext AI"))
        self.setAcceptDrops(True)
        self._apply_adaptive_size()

        self.state.changed.connect(self._on_state_changed)

        self._last_scanned_paths = []
        self._last_manual_exclusions = set()
        self._preview_dialog = None
        self._tour_dialog = None
        self._update_dialog = None
        self._chat_dialog = None
        self._command_palette = None
        self._ui_ready = False

        PricingManager.fetch_prices_background()
        self.controller.init_plugins(self)

        self._init_ui()
        self._ui_ready = True
        self.controller.load_initial_settings()

        self._update_theme_metrics()
        theme_bus.theme_changed.connect(self._update_theme_metrics)

    def _apply_adaptive_size(self):
        from PySide6.QtGui import QScreen
        screen = QApplication.primaryScreen()
        if screen:
            rect = screen.availableGeometry()
            w = min(int(rect.width() * 0.8), rect.width())
            h = min(int(rect.height() * 0.8), rect.height())
            self.resize(max(w, 1024), max(h, 700))
        else:
            self.resize(1200, 850)

    def _init_ui(self):
        self.central_widget = QWidget()
        self.central_widget.setObjectName("centralwidget")
        self.setCentralWidget(self.central_widget)

        self.main_layout = QHBoxLayout(self.central_widget)
        self.splitter = QSplitter(Qt.Horizontal)
        self.main_layout.addWidget(self.splitter)

        self.sidebar = Sidebar(self.controller, self._on_ui_settings_change)
        self.splitter.addWidget(self.sidebar)

        self.right_stack = QStackedWidget()
        self.empty_state = EmptyState(self.controller.add_folder)
        self.right_stack.addWidget(self.empty_state)

        right_panel = QWidget()
        self.right_layout = QVBoxLayout(right_panel)

        self.folder_list = FolderList(self._on_edit_folder, self.controller.remove_folder)
        self.file_tree = FileTree(self.controller.toggle_file_exclusion, self.controller.copy_file_with_dependencies)
        self.action_panel = ActionPanel(self._on_run, self.controller._plugin_api)
        self.log_panel = LogPanel()
        self.status_bar = StatusBar()

        self.tree_tabs = QTabWidget()
        self.tree_tabs.addTab(self.file_tree, tr("main_window.tab.file_tree", default="Files"))
        self.analytics_panel = AnalyticsPanel()
        self.tree_tabs.addTab(self.analytics_panel, tr("main_window.tab.analytics", default="Analytics"))

        self.right_layout.addWidget(self.folder_list, 1)
        self.right_layout.addWidget(self.tree_tabs, 4)
        self.right_layout.addWidget(self.action_panel, 0)
        self.right_layout.addWidget(self.log_panel, 1)
        self.right_layout.addWidget(self.status_bar, 0)

        self.right_stack.addWidget(right_panel)
        self.splitter.addWidget(self.right_stack)

        self.toast = ToastNotification(self.central_widget)

        QShortcut(QKeySequence("Ctrl+C"), self, activated=lambda: self._on_run('clipboard'))
        QShortcut(QKeySequence("Ctrl+Return"), self, activated=lambda: self._on_run('preview'))
        QShortcut(QKeySequence("Ctrl+F"), self, activated=self.file_tree.search.setFocus)
        QShortcut(QKeySequence("Ctrl+Shift+P"), self, activated=self._show_command_palette)

    def retranslate_ui(self):
        self.tree_tabs.setTabText(0, tr("main_window.tab.file_tree", default="Files"))
        self.tree_tabs.setTabText(1, tr("main_window.tab.analytics", default="Analytics"))
        self.setWindowTitle(tr("main_window.title", default="CodeContext AI"))
        self.sidebar.retranslate_ui()
        self.action_panel.retranslate_ui()
        self.empty_state.retranslate_ui()
        self.file_tree.retranslate_ui()
        self.analytics_panel.retranslate_ui()
        self.folder_list.retranslate_ui()
        self.status_bar.retranslate_ui()

    def _update_theme_metrics(self):
        m = ThemeManager.get_layout("main_margin", 16)
        s = ThemeManager.get_layout("main_spacing", 12)
        w = ThemeManager.get_layout("sidebar_width", 340)
        self.main_layout.setContentsMargins(m, m, m, m)
        self.right_layout.setContentsMargins(0, 0, 0, 0)
        self.right_layout.setSpacing(s)
        self.splitter.setSizes([w, self.width() - w])

    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()

    def dropEvent(self, event):
        for url in event.mimeData().urls():
            path = url.toLocalFile()
            if os.path.isdir(path):
                self.controller.add_folder(path)
            elif os.path.isfile(path):
                self.controller.add_folder(os.path.dirname(path))
        event.acceptProposedAction()

    def _show_command_palette(self):
        self.state.show_command_palette = True
        self.state.notify()

    def _close_command_palette(self):
        self.state.show_command_palette = False
        self.state.notify()

    def _on_state_changed(self, state):
        if not self._ui_ready: return

        if state.selected_folders:
            self.right_stack.setCurrentIndex(1)
        else:
            self.empty_state.update_recent(state.settings.recent_workspaces)
            self.right_stack.setCurrentIndex(0)

        self.sidebar.update_ui(state.settings)
        self.folder_list.update_ui(state.selected_folders, state.temp_folders)

        if state.scanned_files_paths:
            paths_changed = self._last_scanned_paths != state.scanned_files_paths
            exclusions_changed = self._last_manual_exclusions != state.manual_exclusions

            if paths_changed:
                self.file_tree.populate(state.scanned_files_paths, state.scanned_file_metadata, state.manual_exclusions)
                self.analytics_panel.populate(state.scanned_file_metadata, state.manual_exclusions)
                self._last_scanned_paths = list(state.scanned_files_paths)
                self._last_manual_exclusions = set(state.manual_exclusions)
            elif exclusions_changed:
                self.file_tree.update_exclusions(state.manual_exclusions)
                self.analytics_panel.populate(state.scanned_file_metadata, state.manual_exclusions)
                self._last_manual_exclusions = set(state.manual_exclusions)
        else:
            if self._last_scanned_paths:
                self.file_tree.clear()
                self.analytics_panel.table.setRowCount(0)
                self._last_scanned_paths = []
                self._last_manual_exclusions = set()

        self.action_panel.update_ui(state.settings)
        self.log_panel.update_logs(state.logs)

        tokens_display = state.total_tokens if state.final_output_text else state.selected_tokens
        model = state.settings.llm_model
        estimated_cost = tokens_display * PricingManager.get_price(model)

        self.status_bar.update_ui(state.status_message, state.progress, tokens_display, estimated_cost)

        if state.toast_message:
            self.toast.show_message(state.toast_message)
            self.controller.clear_toast()

        if getattr(state, 'show_command_palette', False):
            if not self._command_palette:
                commands = {
                    tr("main_window.command.copy_to_clipboard", default="Copy"): lambda: self._on_run('clipboard'),
                    tr("main_window.command.open_in_editor", default="Editor"): lambda: self._on_run('editor'),
                    tr("main_window.command.preview", default="Preview"): lambda: self._on_run('preview'),
                    tr("main_window.command.send_to_chat", default="Chat"): lambda: self._on_run('chat'),
                    tr("main_window.command.save_to_file", default="Save"): lambda: self._on_run('file'),
                    tr("main_window.command.toggle_minify", default="Minify"): lambda: self.action_panel.chk_minify.setChecked(not self.action_panel.chk_minify.isChecked()),
                    tr("main_window.command.toggle_skeleton", default="Skeleton"): lambda: self.action_panel.chk_skeleton.setChecked(not self.action_panel.chk_skeleton.isChecked()),
                    tr("main_window.command.toggle_mermaid", default="Mermaid"): lambda: self.sidebar.chk_mermaid.setChecked(not self.sidebar.chk_mermaid.isChecked()),
                    tr("main_window.command.clear_workspace", default="Clear"): self.controller.clear_folders,
                    tr("main_window.command.apply_json_patch", default="JSON Patch"): self.sidebar._open_patch_dialog,
                    tr("main_window.command.toggle_theme", default="Theme"): lambda: ThemeManager.apply_theme(mode="dark" if ThemeManager._current_mode == "light" else "light"),
                    tr("main_window.command.check_updates", default="Update"): lambda: self.controller.check_for_updates(get_app_version()),
                }
                for a_id, a_data in self.controller._plugin_api.ui.action_buttons.items():
                    commands[a_data["label"]] = a_data["callback"]

                self._command_palette = CommandPaletteDialog(self, commands, self._close_command_palette)
                parent_geo = self.geometry()
                x = parent_geo.x() + (parent_geo.width() - self._command_palette.width()) // 2
                y = parent_geo.y() + (parent_geo.height() - self._command_palette.height()) // 2 - 50
                self._command_palette.move(x, y)
                self._command_palette.show()
                self._command_palette.raise_()
                self._command_palette.search_input.setFocus()
        elif self._command_palette:
            self._command_palette.close()
            self._command_palette = None

        if state.show_chat:
            if not self._chat_dialog:
                from .dialogs import ChatDialog
                self._chat_dialog = ChatDialog(self, state, self.controller)
                self._chat_dialog.update_data(state)
                self._chat_dialog.show()
                self._chat_dialog.raise_()
        elif self._chat_dialog:
            self._chat_dialog.close()
            self._chat_dialog = None

        if state.show_preview:
            if not self._preview_dialog:
                self._preview_dialog = AdvancedPreviewDialog(self, state, self.controller.close_preview, self.controller)
                self._preview_dialog.update_data(state)
                self._preview_dialog.show()
                self._preview_dialog.raise_()
        elif self._preview_dialog:
            self._preview_dialog.close()
            self._preview_dialog = None

        if state.show_tour:
            if not self._tour_dialog:
                self._tour_dialog = InteractiveTourDialog(self, state.tour_steps, self.controller.close_tour)
                self._tour_dialog.show()
                self._tour_dialog.raise_()
        elif self._tour_dialog:
            self._tour_dialog.close()
            self._tour_dialog = None

        if state.show_update:
            if not self._update_dialog:
                self._update_dialog = UpdateDialog(self, state.update_info, self.controller.close_update_dialog, self.controller)
                self._update_dialog.show()
                self._update_dialog.raise_()
            else:
                self._update_dialog.update_data(state.update_info)
        elif self._update_dialog:
            self._update_dialog.close()
            self._update_dialog = None

    def _on_ui_settings_change(self):
        s_data = self.sidebar.get_settings()
        a_data = self.action_panel.get_settings()
        self.controller.update_settings({**s_data, **a_data})

    def _on_run(self, target):
        self._on_ui_settings_change()
        if target in ('file', 'pdf'):
            path, _ = QFileDialog.getSaveFileName(self, tr("main_window.save_file.title", default="Save File"), "", "All Files (*.*)")
            if path:
                self.controller.start_processing(target, path)
        else:
            self.controller.start_processing(target)

    def _on_edit_folder(self, path):
        dialog = EditFolderDialog(self, path)
        if dialog.exec():
            new_path = dialog.get_input()
            if new_path:
                self.controller.edit_folder(path, new_path)

    def closeEvent(self, event):
        self.controller.shutdown()
        self.controller.clear_folders()
        super().closeEvent(event)
