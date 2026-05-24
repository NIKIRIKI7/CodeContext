import os
from PySide6.QtWidgets import QMainWindow, QWidget, QHBoxLayout, QVBoxLayout, QSplitter, QFileDialog
from PySide6.QtCore import Signal, QObject, Qt

from ..store.store import Store
from ..controllers.main_controller import MainController
from .components.sidebar import Sidebar
from .components.folder_list import FolderList
from .components.action_panel import ActionPanel
from .components.log_panel import LogPanel
from .components.status_bar import StatusBar
from .components.file_tree import FileTree
from .dialogs import AdvancedPreviewDialog, InteractiveTourDialog, EditFolderDialog
from .theme_manager import ThemeManager, theme_bus


class UIBridge(QObject):
    state_changed = Signal(object)


class MainWindow(QMainWindow):
    def __init__(self, store: Store, controller: MainController):
        super().__init__()
        self.store = store
        self.controller = controller
        self.setWindowTitle("CodeContext AI")
        self.resize(1200, 850)
        self.setAcceptDrops(True)

        self.bridge = UIBridge()
        self.bridge.state_changed.connect(self._on_store_changed)
        self.unsubscribe = self.store.subscribe(self.bridge.state_changed.emit)

        self._preview_dialog = None
        self._tour_dialog = None

        self._init_ui()
        self._update_theme_metrics()
        theme_bus.theme_changed.connect(self._update_theme_metrics)

        self.controller.load_initial_settings()

    def _init_ui(self):
        self.central_widget = QWidget()
        self.central_widget.setObjectName("centralwidget")  # Явное указание для QSS (Фон)
        self.setCentralWidget(self.central_widget)

        self.main_layout = QHBoxLayout(self.central_widget)

        self.splitter = QSplitter(Qt.Horizontal)
        self.main_layout.addWidget(self.splitter)

        self.sidebar = Sidebar(self.controller, self._on_ui_settings_change)
        self.splitter.addWidget(self.sidebar)

        right_panel = QWidget()
        self.right_layout = QVBoxLayout(right_panel)

        self.folder_list = FolderList(self._on_edit_folder, self.controller.remove_folder)
        self.file_tree = FileTree(self.controller.toggle_file_exclusion, self.controller.copy_file_with_dependencies)
        self.action_panel = ActionPanel(self._on_run)
        self.log_panel = LogPanel()
        self.status_bar = StatusBar()

        self.right_layout.addWidget(self.folder_list, 1)
        self.right_layout.addWidget(self.file_tree, 4)
        self.right_layout.addWidget(self.action_panel, 0)
        self.right_layout.addWidget(self.log_panel, 1)
        self.right_layout.addWidget(self.status_bar, 0)

        self.splitter.addWidget(right_panel)

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

    def _on_store_changed(self, state):
        self.sidebar.update_ui(state.settings)
        self.folder_list.update_ui(state.selected_folders, state.temp_folders)

        if state.scanned_files_paths:
            self.file_tree.populate(state.scanned_files_paths, state.scanned_file_metadata, state.manual_exclusions)
        else:
            self.file_tree.clear()

        self.action_panel.update_ui(state.settings)
        self.log_panel.update_logs(state.logs)
        self.status_bar.update_ui(state.status_message, state.progress, state.total_tokens)

        if state.show_preview:
            if not self._preview_dialog:
                self._preview_dialog = AdvancedPreviewDialog(self, state, self.controller.close_preview)
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

    def _on_ui_settings_change(self):
        s_data = self.sidebar.get_settings()
        a_data = self.action_panel.get_settings()
        self.controller.update_settings({**s_data, **a_data})

    def _on_run(self, target):
        self._on_ui_settings_change()
        if target in ('file', 'pdf'):
            path, _ = QFileDialog.getSaveFileName(self, "Сохранить файл", "", "All Files (*.*)")
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
        self.controller.clear_folders()
        if self.unsubscribe:
            self.unsubscribe()
        super().closeEvent(event)