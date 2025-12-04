"""
PDF generation service for the Code Aggregator application.
"""
import os
from fpdf import FPDF
from src.config import FONT_PATH_WINDOWS


class PdfGenerator:
    """
    Responsible for creating PDF files from text content.
    """
    @staticmethod
    def create_pdf(text_content: str, output_path: str, logger_callback):
        """
        Create a PDF file from the given text content.
        
        Args:
            text_content: Text content to include in the PDF
            output_path: Path where the PDF should be saved
            logger_callback: Function to call with log messages
        """
        pdf = FPDF()
        pdf.add_page()
        
        # Setup fonts for Cyrillic support
        # FPDF2 requires a TTF font to be loaded for UTF-8 characters
        font_path = FONT_PATH_WINDOWS if os.path.exists(FONT_PATH_WINDOWS) else None
        
        try:
            if font_path:
                pdf.add_font("Arial", "", font_path)
                pdf.set_font("Arial", size=10)
            else:
                logger_callback("[WARN] Arial font not found. Cyrillic may not display correctly.")
                pdf.set_font("Helvetica", size=10)  # Standard font (no Cyrillic)

            # Set up automatic page breaks
            pdf.set_auto_page_break(auto=True, margin=15)
            
            # Pre-process text to avoid encoding issues in case font is not found
            if not font_path:
                text_content = text_content.encode('latin-1', 'replace').decode('latin-1')

            pdf.multi_cell(0, 5, text_content)
            
            pdf.output(output_path)
            logger_callback(f"[SUCCESS] PDF saved: {output_path}")
        except Exception as e:
            logger_callback(f"[ERROR] PDF generation error: {e}")