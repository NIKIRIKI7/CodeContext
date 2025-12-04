"""
Main entry point for the CodeContext AI application.
"""
import sys
from src.ui.main_window import App


if __name__ == "__main__":
    try:
        app = App()
        app.mainloop()
    except KeyboardInterrupt:
        sys.exit()