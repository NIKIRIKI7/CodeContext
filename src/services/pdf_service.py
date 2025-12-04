"""PDF service for generating PDF documents."""
import os
import platform
from pathlib import Path
from fpdf import FPDF


class PdfService:
    """Service for creating PDF documents from text content."""
    
    @staticmethod
    def create_pdf(text_content: str, output_path: str, font_path: str = None):
        """
        Create a PDF file from text content.
        
        Args:
            text_content: Text content to include in PDF
            output_path: Path where PDF will be saved
            font_path: Optional path to custom font file
        """
        pdf = FPDF()
        pdf.add_page()
        pdf.set_auto_page_break(auto=True, margin=15)
        
        # Попытка найти системный шрифт Arial для поддержки UTF-8
        if not font_path:
            system = platform.system()
            if system == "Windows":
                font_path = os.path.join(os.environ["WINDIR"], "Fonts", "arial.ttf")
            elif system == "Linux":
                font_path = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
            elif system == "Darwin":
                font_path = "/Library/Fonts/Arial.ttf"

        try:
            if font_path and os.path.exists(font_path):
                pdf.add_font("CustomFont", "", font_path)
                pdf.set_font("CustomFont", size=10)
            else:
                pdf.set_font("Courier", size=10)  # Безопасный фоллбэк
        except:
             pdf.set_font("Courier", size=10)

        pdf.multi_cell(0, 5, text_content)
        pdf.output(output_path)