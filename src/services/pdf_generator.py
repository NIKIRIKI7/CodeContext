"""
PDF generation service for the Code Aggregator application.
"""
import platform
import os
from fpdf import FPDF
from typing import Callable
from src.config import FONT_PATH


class PdfService:
    """Service for generating PDF with UTF-8 support."""

    @staticmethod
    def get_system_font_path() -> str:
        """Attempt to find system Arial font for Cyrillic support."""
        system = platform.system()
        if system == "Windows":
            path = os.path.join(os.environ["WINDIR"], "Fonts", "arial.ttf")
            if os.path.exists(path): return path
        elif system == "Darwin": # MacOS
            paths = ["/Library/Fonts/Arial.ttf", "/System/Library/Fonts/Helvetica.ttc"]
            for p in paths:
                if os.path.exists(p): return p
        elif system == "Linux":
            paths = ["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                     "/usr/share/fonts/TTF/Arial.ttf"]
            for p in paths:
                if os.path.exists(p): return p
        return FONT_PATH

    def create_pdf(self, text_content: str, output_path: str, log_callback: Callable):
        """
        Create a PDF file from the given text content.

        Args:
            text_content: Text content to include in the PDF
            output_path: Path where the PDF should be saved
            log_callback: Function to call with log messages
        """
        pdf = FPDF()
        pdf.add_page()
        pdf.set_auto_page_break(auto=True, margin=15)

        font_path = self.get_system_font_path()
        added_font = False

        if font_path:
            try:
                # Register font for UTF-8 support
                pdf.add_font("CustomFont", "", font_path)
                pdf.set_font("CustomFont", size=10)
                added_font = True
            except Exception as e:
                log_callback(f"[WARN] Failed to load font {font_path}: {e}")

        if not added_font:
            log_callback("[WARN] Using standard font. Cyrillic may not display correctly.")
            pdf.set_font("Courier", size=10) # Courier safer for code

        # Write text line by line or in chunks
        try:
            # Multi_cell handles text wrapping automatically
            pdf.multi_cell(0, 5, text_content)
            pdf.output(output_path)
            log_callback(f"[SUCCESS] PDF saved: {output_path}")
        except Exception as e:
            log_callback(f"[ERROR] PDF writing error: {e}")