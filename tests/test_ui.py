import sys
import pytest
from unittest.mock import MagicMock
from PySide6.QtWidgets import QApplication


@pytest.fixture(scope="session")
def qapp():
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    yield app


def test_action_panel(qapp):
    from src.ui.components.action_panel import ActionPanel
    from src.store.state import AppSettings
    panel = ActionPanel(lambda x: None)
    settings = AppSettings(minify=True, output_format="xml")
    panel.update_ui(settings)
    assert panel.get_settings()['minify'] is True


def test_empty_state(qapp):
    from src.ui.components.empty_state import EmptyState
    panel = EmptyState(lambda x: None)
    panel.update_recent(["/fake/path"])


def test_file_tree(qapp):
    from src.ui.components.file_tree import FileTree
    tree = FileTree(lambda x, y: None, lambda x, y: None)
    tree.populate(["/a/b.py", "/a/c.js"], {"/a/b.py": {"tokens": 10, "category": "LIGHT"}}, {"/a/c.js"})
    assert tree.model.rowCount() > 0
    tree._filter_tree("b.py")
    tree.clear()


def test_folder_list(qapp):
    from src.ui.components.folder_list import FolderList
    lst = FolderList(lambda x: None, lambda x: None)
    lst.update_ui(["/test1"], ["/test1"])
    lst.update_ui([], [])


def test_log_panel(qapp):
    from src.ui.components.log_panel import LogPanel
    p = LogPanel()
    p.update_logs(["A", "B"])
    assert "A" in p.toPlainText()


def test_status_bar(qapp):
    from src.ui.components.status_bar import StatusBar
    b = StatusBar()
    b.update_ui("msg", 0.5, 100)
    assert b.lbl_status.text() == "msg"


def test_sidebar(qapp):
    from src.ui.components.sidebar import Sidebar
    from src.store.state import AppSettings
    ctrl = MagicMock()
    ctrl._store.state.settings.custom_presets = {}
    ctrl._store.state.settings.custom_prompt_presets = {}
    sidebar = Sidebar(ctrl, lambda: None)
    settings = AppSettings(extensions=".py")
    sidebar.update_ui(settings)
    assert sidebar.get_settings()['extensions'] == ".py"


def test_dialogs(qapp):
    from src.ui.dialogs import EditFolderDialog, JsonPatchDialog
    d = EditFolderDialog(None, "/a")
    d.entry.setText("/b")
    d._on_ok()
    assert d.get_input() == "/b"

    j = JsonPatchDialog(None)
    j.txt_patch.setPlainText("[]")
    assert j.get_json() == "[]"


def test_main_window(qapp):
    from src.ui.main_window import MainWindow
    from src.store.store import Store
    store = Store()
    ctrl = MagicMock()
    win = MainWindow(store, ctrl)
    store.dispatch("UI_ADD_LOG", "hi")
    store.dispatch("FOLDER_ADD", "/tmp")
    assert win.right_stack.currentIndex() == 1
    store.dispatch("FOLDER_CLEAR", None)
    assert win.right_stack.currentIndex() == 0
