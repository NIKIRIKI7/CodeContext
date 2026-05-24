"""
main.py — точка входа приложения.
"""
import sys
import argparse
import os
import time

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.di_container import DIContainer
from src.utils.async_runtime import AsyncRuntime
from src.utils.logger import app_logger

def main():
    app_logger.info("="*50)
    app_logger.info("🚀 CodeContext AI Started")
    app_logger.info(f"🔧 Arguments: {sys.argv}")

    parser = argparse.ArgumentParser(description="CodeContext AI Entry Point")
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode")
    parser.add_argument("--path", type=str, help="Target path for CLI mode")
    parser.add_argument(
        "--mode",
        type=str,
        choices=["default", "shallow", "deep"],
        default="default",
        help="Scan mode for files",
    )
    parser.add_argument("--install-context", action="store_true", help="Install Context Menu")
    parser.add_argument("--remove-context", action="store_true", help="Remove Context Menu")
    args = parser.parse_args()

    container = DIContainer()

    if args.install_context:
        print("Installing context menu...")
        success, msg = container.integration_service.install_context_menu()
        print(msg)
        if not success:
            time.sleep(3)
        sys.exit(0)

    if args.remove_context:
        print("Removing context menu...")
        success, msg = container.integration_service.remove_context_menu()
        print(msg)
        if not success:
            time.sleep(3)
        sys.exit(0)

    if args.cli:
        if not args.path:
            app_logger.error("CLI Mode Error: --path is required")
            print("Error: --path is required for CLI mode")
            sys.exit(1)

        container.cli_controller.run(args.path, args.mode)
        sys.exit(0)

    # GUI Mode
    app_logger.info("Starting GUI Mode...")
    AsyncRuntime.start()

    from src.ui.main_window import MainWindow
    app = MainWindow(container.store, container.main_controller)
    app.protocol("WM_DELETE_WINDOW", lambda: _on_close(app))
    app.mainloop()

def _on_close(app):
    """Корректное завершение: закрытие окна и остановка loop."""
    app_logger.info("Application shutting down...")
    try:
        app.on_closing()
    except Exception:
        pass
    AsyncRuntime.stop()
    sys.exit(0)

if __name__ == "__main__":
    main()