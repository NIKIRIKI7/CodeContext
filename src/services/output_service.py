import os
import sys
import pyperclip
import platform
import tempfile
import subprocess
from fpdf import FPDF
from ..utils.config import FONT_PATH
from ..utils.logger import app_logger
from src.i18n import tr


class OutputService:
    """Сервис для сохранения результатов (Буфер, Файл, PDF)"""

    @staticmethod
    def copy_to_clipboard(text: str):
        try:
            pyperclip.copy(text)
        except Exception as e:
            if platform.system() == "Linux":
                raise RuntimeError(tr("output_service.clipboard.linux_install")) from e
            raise RuntimeError(tr("output_service.clipboard.copy_error", error=e)) from e

    @staticmethod
    def save_to_file(text: str, path: str):
        with open(path, 'w', encoding='utf-8') as f:
            f.write(text)

    @staticmethod
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

    @staticmethod
    def save_to_pdf(text: str, path: str):
        pdf = FPDF()
        pdf.add_page()
        pdf.set_auto_page_break(auto=True, margin=15)

        font_loaded = False
        if FONT_PATH:
            try:
                pdf.add_font("CustomFont", "", FONT_PATH)
                pdf.set_font("CustomFont", size=10)
                font_loaded = True
            except Exception:
                app_logger.warning(f"[Output] Custom font not loaded: {FONT_PATH}")

        if not font_loaded:
            pdf.set_font("Courier", size=10)

        text = text.encode('latin-1', 'replace').decode('latin-1')
        try:
            pdf.multi_cell(0, 5, text)
            pdf.output(path)
        except Exception as e:
            raise e