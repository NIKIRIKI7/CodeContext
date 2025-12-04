"""Unified PDF generation service."""
from fpdf import FPDF
from src.config import FONT_PATH
import logging

class PdfService:
    @staticmethod
    def create_pdf(text_content: str, output_path: str):
        pdf = FPDF()
        pdf.add_page()
        pdf.set_auto_page_break(auto=True, margin=15)
        
        font_loaded = False
        if FONT_PATH:
            try:
                pdf.add_font("CustomFont", "", FONT_PATH)
                pdf.set_font("CustomFont", size=10)
                font_loaded = True
            except Exception as e:
                logging.warning(f"Font loading failed: {e}")

        if not font_loaded:
            pdf.set_font("Courier", size=10)

        # Handle encoding issues by replacing unsupported chars
        safe_text = text_content.encode('latin-1', 'replace').decode('latin-1')
        
        # If we have a custom font, we can try using utf-8 directly via some hacks, 
        # but standard FPDF is tricky with Unicode. 
        # For full unicode support one might need fpdf2, assuming fpdf1 here for compatibility.
        # We will try to write as is if font loaded, else safe latin-1.
        
        try:
            if font_loaded:
                # FPDF with TTF supports more chars, but needs compatible calls
                pdf.multi_cell(0, 5, text_content)
            else:
                pdf.multi_cell(0, 5, safe_text)
            
            pdf.output(output_path)
        except Exception as e:
            # Fallback
            pdf.output(output_path)
            raise e