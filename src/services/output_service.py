import pyperclip
from fpdf import FPDF
from ..utils.config import FONT_PATH


class OutputService:
    """Сервис для сохранения результатов (Буфер, Файл, PDF)"""

    @staticmethod
    def copy_to_clipboard(text: str):
        pyperclip.copy(text)

    @staticmethod
    def save_to_file(text: str, path: str):
        with open(path, 'w', encoding='utf-8') as f:
            f.write(text)

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
                print(Exception)
                pass

        if not font_loaded:
            pdf.set_font("Courier", size=10)

        text = text.encode('latin-1', 'replace').decode('latin-1')
        try:
            pdf.multi_cell(0, 5, text)
            pdf.output(path)
        except Exception as e:
            raise e