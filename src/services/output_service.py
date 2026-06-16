import os
import tempfile
import subprocess
import webbrowser
import pyperclip
from src.i18n import tr

def copy_to_clipboard(text: str):
    pyperclip.copy(text)

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
        else:
            # ponytail: stdlib webbrowser delegates to the native OS file/folder openers
            webbrowser.open(tmp_path)
    except Exception as e:
        raise RuntimeError(tr("output_service.editor.open_error", editor_cmd=editor_cmd, error=e))
