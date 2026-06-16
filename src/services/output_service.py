import os
import tempfile
import subprocess
import platform
from PySide6.QtGui import QGuiApplication
from src.i18n import tr

def copy_to_clipboard(text: str):
    # ponytail: Qt-буфер теряет данные при мгновенном выходе из CLI.
    # Возвращаем надежный pyperclip для работы без event loop.
    try:
        import pyperclip
        pyperclip.copy(text)
        return
    except ImportError:
        app = QGuiApplication.instance() or QGuiApplication(sys.argv)
        app.clipboard().setText(text)
        app.processEvents()

def save_to_file(text: str, path: str):
    with open(path, 'w', encoding='utf-8') as f:
        f.write(text)

def open_in_editor(text: str, editor_cmd: str):
    fd, tmp_path = tempfile.mkstemp(suffix=".md", prefix="codecontext_")
    with os.fdopen(fd, 'w', encoding='utf-8') as f:
        f.write(text)
    try:
        if editor_cmd:
            subprocess.Popen([editor_cmd, tmp_path])
        elif platform.system() == "Windows":
            os.startfile(tmp_path)
        elif platform.system() == "Darwin":
            subprocess.Popen(["open", tmp_path])
        else:
            subprocess.Popen(["xdg-open", tmp_path])
    except Exception as e:
        raise RuntimeError(tr("output_service.editor.open_error", editor_cmd=editor_cmd, error=e))
