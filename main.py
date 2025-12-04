"""
CodeContext AI v3.5 - Entry Point.
Supports UI mode and CLI mode (for Context Menu).
"""
import sys
import argparse
from src.config import PRESETS

def main():
    parser = argparse.ArgumentParser(description="CodeContext AI")
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode")
    parser.add_argument("--path", type=str, help="Path to process (for CLI mode)")
    
    args = parser.parse_args()

    if args.cli and args.path:
        # CLI Mode (Fast load)
        from src.cli_handler import run_headless
        run_headless(args.path)
    else:
        # UI Mode (Lazy load customtkinter)
        from src.ui.main_window import ModernApp
        app = ModernApp()
        app.mainloop()

if __name__ == "__main__":
    main()